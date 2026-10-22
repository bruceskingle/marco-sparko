use std::{
    collections::{BTreeMap, HashMap}, fs::{File, FileTimes, OpenOptions}, io::{BufRead, BufReader, Seek, SeekFrom, Write}, path::PathBuf, pin::pin, time::{Duration, SystemTime, UNIX_EPOCH},
};

use futures::future::{Either, select};

use serde::{Serialize, de::DeserializeOwned};
use sparko_graphql::{
    GraphQLCaller, GraphQLQuery, GraphQLResponse, ProgressMonitor,
    types::{Date, DateTime},
};
use time_tz::Tz;

use crate::CacheConfig;

use super::{CancelToken, DataSetAttributes};

/// The smallest number of consecutive final cached samples which are not fetched again, 12 hours of half hour samples.
/// Shorter runs are fetched again with the gaps around them, so that scattered samples do not split a fetch into many
/// small requests.
const MIN_CACHED_RUN: usize = 24;

/// The periods to fetch between start and end, as unix timestamps.
///
/// cached gives the start, end and finality of each cached sample, in order. Runs of at least MIN_CACHED_RUN
/// consecutive final samples are not fetched again, non-final samples may have changed so they always are. Each
/// period is also split at the month boundaries, so that no request covers more than one month.
fn fetch_windows(cached: &[(i64, i64, bool)], start: i64, end: i64, month_boundaries: &[i64]) -> Vec<(i64, i64)> {
    // The runs of consecutive final samples which are long enough to keep.
    let mut kept_runs = Vec::new();
    let mut run: Option<(i64, i64, usize)> = None;
    let mut close_run = |run: &mut Option<(i64, i64, usize)>| {
        if let Some((run_start, run_end, count)) = run.take() {
            if count >= MIN_CACHED_RUN {
                kept_runs.push((run_start, run_end));
            }
        }
    };
    for &(sample_start, sample_end, is_final) in cached {
        match run {
            Some((run_start, run_end, count)) if is_final && sample_start == run_end => {
                run = Some((run_start, sample_end, count + 1));
            },
            _ => {
                close_run(&mut run);
                if is_final {
                    run = Some((sample_start, sample_end, 1));
                }
            },
        }
    }
    close_run(&mut run);

    // The gaps between the kept runs.
    let mut gaps = Vec::new();
    let mut from = start;
    for (run_start, run_end) in kept_runs {
        if run_start >= end {
            break;
        }
        if run_start > from {
            gaps.push((from, run_start));
        }
        from = from.max(run_end);
    }
    if from < end {
        gaps.push((from, end));
    }

    // Split each gap at the month boundaries.
    gaps.into_iter()
        .flat_map(|(gap_start, gap_end)| {
            let mut windows = Vec::new();
            let mut from = gap_start;
            for &boundary in month_boundaries {
                if boundary > from && boundary < gap_end {
                    windows.push((from, boundary));
                    from = boundary;
                }
            }
            windows.push((from, gap_end));
            windows
        })
        .collect()
}

/// Whether the final samples in data cover the whole of the window from start to end with no gaps.
fn window_complete<V>(data: &BTreeMap<DateTime, Item<V>>, start: &DateTime, end: &DateTime) -> bool {
    let mut covered_to = *start;
    for (sample_start, item) in data.range(start..end) {
        if *sample_start != covered_to || !item.is_final {
            return false;
        }
        covered_to = item.end;
    }
    covered_to >= *end
}

/// The number of times a request which failed with a transient error is retried.
const RETRIES: u32 = 5;
/// The delay before the first retry, which doubles for each retry after it.
const FIRST_RETRY_DELAY: Duration = Duration::from_secs(2);

/// Whether a request which failed with this error may succeed if it is made again, e.g. after a network error or a
/// gateway timeout.
fn is_transient(error: &sparko_graphql::Error) -> bool {
    match error {
        sparko_graphql::Error::IOError(_) => true,
        sparko_graphql::Error::HttpError(status) => status.is_server_error() || status.as_u16() == 429,
        _ => false,
    }
}

/// Make a request, returning None without waiting for the response if the fetch is cancelled first.
/// A request which fails with a transient error is made again, with the same query and so the same cursor, after a
/// delay. Progress messages from the request are passed on to progress with the given prefix.
async fn call_unless_cancelled<Q: GraphQLQuery<R>, R: GraphQLResponse>(
    request_manager: &impl GraphQLCaller,
    query: &Q,
    cancel: &CancelToken,
    progress: &(impl ProgressMonitor + ?Sized),
    prefix: &str,
) -> anyhow::Result<Option<R>> {
    let request_progress = |message: String| progress.progress(format!("{prefix}: {message}"));
    let mut delay = FIRST_RETRY_DELAY;
    let mut retries = 0;
    loop {
        match select(pin!(request_manager.call3(query, None, &request_progress)), pin!(cancel.cancelled())).await {
            Either::Right(_) => return Ok(None),
            Either::Left((Ok(response), _)) => return Ok(Some(response)),
            Either::Left((Err(error), _)) if retries < RETRIES && is_transient(&error) => {
                retries += 1;
                request_progress(format!("request failed ({error}), retry {retries} of {RETRIES} in {}s", delay.as_secs()));
                if let Either::Right(_) = select(pin!(tokio::time::sleep(delay)), pin!(cancel.cancelled())).await {
                    return Ok(None);
                }
                delay *= 2;
            },
            Either::Left((Err(error), _)) => return Err(error.into()),
        }
    }
}

pub trait SlotSource {
    type Value: DeserializeOwned + Serialize + Clone;
    type Response: GraphQLResponse;
    type Query: GraphQLQuery<Self::Response>;

    fn hash_key(&self) -> &str;
    /// A query for the samples from start up to end. A source whose query has no end may return samples after it.
    fn initial_query(&mut self, start: &DateTime, end: &DateTime) -> anyhow::Result<Self::Query>;
    /// The query for the next page of the samples from start up to end, if there is one.
    fn continuation_query(&mut self, start: &DateTime, end: &DateTime, resp: &Self::Response) -> anyhow::Result<Option<Self::Query>>;
    fn items(&self, response: Self::Response) -> anyhow::Result<impl IntoIterator<Item = anyhow::Result<Self::Value>>>;
    fn index(&self, value: &Self::Value) -> anyhow::Result<(DateTime, DateTime, bool)>;
    fn refresh_after(&self) -> Duration;
}


pub struct Item<V> {
    pub end: DateTime,
    pub is_final: bool,
    pub attributes: DataSetAttributes,
    pub value: V,
}

/// A monthly dataset of fixed-interval records that may arrive out of order.
/// Each sample has a start and end DateTime, for contiguous samples A and B A.end == B.start.
pub struct MonthlyFixedSlotDataSet<S: SlotSource>
// where
//     V: DeserializeOwned + Serialize + Clone,
{
    /// The start date of this DataSet
    pub start_date: Date,
    /// The start date of the next monthly bucket, also the exclusive end date of this one.
    pub next_date: Date,
    /// The values indexed by the start DateTime of the sample.
    pub data: BTreeMap<DateTime, Item<<S as SlotSource>::Value>>,
    /// The end DateTime of the last final sample in this DataSet
    pub last_end: DateTime,
    /// True iff all samples for this bucket are. present and final.
    pub is_complete: bool,
}

impl<S: SlotSource> MonthlyFixedSlotDataSet<S> 
// where
//     V: DeserializeOwned + Serialize + Clone,
{
    fn new(
        start_date: Date,
        next_date: Date,
        tz: &'static Tz,
        data: BTreeMap<DateTime, Item<<S as SlotSource>::Value>>,
    ) -> Self {
        let mut last_end = start_date.at_midnight(tz);

        for (start, item) in &data {
            if &last_end != start {
                // there is a gap here
                break;
            }

            if item.is_final {
                last_end = item.end.clone();
            }
        }

        let is_complete = last_end == next_date.at_midnight(tz);

        Self {
            start_date,
            next_date,
            data,
            last_end,
            is_complete
        }
    }

    fn load(
        source: &S,
        config: &CacheConfig,
        file: &mut File,
        start_date: Date,
        next_date: Date,
        tz: &'static Tz,
    ) -> anyhow::Result<Self>
    {
        let mut data: BTreeMap<DateTime, Item<<S as SlotSource>::Value>> = BTreeMap::new();
        let lines = BufReader::new(&mut *file).lines();
        for line in lines.map_while(Result::ok) {
            if config.verbose 
            {
                println!("READ {}", line);
            }

            match serde_json::from_str(&line) {
                Ok(value) => {
                    
                    let (start, end, is_final) = source.index(&value)?;
                    let item = Item {
                        end,
                        is_final,
                        attributes: DataSetAttributes { cached: true, },
                        value,
                    };

                    data.insert(start, item);

                },
                Err(_) => {
                    file.seek(SeekFrom::Start(0))?;
                    file.set_len(0)?;
                    data.clear();
                    break;
                },
            }
        }

        Ok(Self::new(start_date, next_date, tz, data))
    }

    fn key_for(start: &Date) -> i32 {
        let key = start.year() * 12 + start.month() as i32 - 1;
        key
    }

    pub async fn fetch(
        mut source: S,
        tz: &'static Tz,
        config: &CacheConfig,
        request_manager: &impl GraphQLCaller,
        start: &DateTime,
        end: &DateTime,
        cancel: &CancelToken,
        progress: &(impl ProgressMonitor + ?Sized),
    ) -> anyhow::Result<Vec<MonthlyFixedSlotDataSet<S>>>
    {
        let mut result_keys = Vec::new();
        let mut result = Vec::new();
        let mut data_set_map = HashMap::new();

        {
            // scope this start_date
            let mut start_date = start.to_date();

            progress.progress(String::from("Loading cached data"));

            // end is exclusive so a bucket starting at end is not needed
            while start_date.at_midnight(tz) < *end {
                let open_data_set: OpenMonthlyFixedSlotDataSet<S> = OpenMonthlyFixedSlotDataSet::new(&source, &config, source.hash_key(), &start_date, tz)?;
                
                let key = Self::key_for(&start_date);
                result_keys.push(key);


                start_date = open_data_set.data_set.next_date.clone();
                data_set_map.insert(key, open_data_set);

            }
        }

        // At this point we have loaded all the required DataSets from local cache, if they are all complete or withing their refresh
        // duration then we can just return what we have.
        // This function returns the first missing start time unless all buckets are within their refresh duration
        fn all_up_to_date<S: SlotSource>(refresh_after: &Duration, result_keys: &Vec<i32>, data_set_map: &HashMap<i32, OpenMonthlyFixedSlotDataSet<S>>) -> anyhow::Result<Option<DateTime>> {
            let mut result = None;
            for key in result_keys {
                let ods = data_set_map.get(key).unwrap();
                let ds = &ods.data_set;
                if ! ds.is_complete {
                    if result.is_none() {
                        result = Some(ds.last_end);
                    }
                    if ods.is_stale(refresh_after)? {
                        return Ok(result);
                    }
                }
            }
            Ok(None)
        }

        let refresh_after = source.refresh_after();


        // The fetching is in a block of its own so that if it fails, the rows fetched before the failure are still
        // written to the cache below rather than being lost.
        let fetch_result: anyhow::Result<()> = async {
            if let Some(refresh_start) = all_up_to_date(&refresh_after, &result_keys, &data_set_map)? {
                if &refresh_start < end {
                    let refresh_start = if &refresh_start < start { start } else {&refresh_start};

                    // Fetch in windows which avoid the data we already have and cover at most one month each.
                    let cached: Vec<(i64, i64, bool)> = result_keys.iter()
                        .flat_map(|key| data_set_map[key].data_set.data.iter())
                        .map(|(sample_start, item)| (sample_start.unix_timestamp(), item.end.unix_timestamp(), item.is_final))
                        .collect();
                    let month_boundaries: Vec<i64> = result_keys.iter()
                        .map(|key| data_set_map[key].data_set.next_date.at_midnight(tz).unix_timestamp())
                        .collect();
                    let windows = fetch_windows(&cached, refresh_start.unix_timestamp(), end.unix_timestamp(), &month_boundaries);

                    // The number of requests made and of rows they returned, for progress messages.
                    let mut requests = 0;
                    let mut rows = 0;
                    let date_format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]").unwrap();

                    for (window_number, (window_start, window_end)) in windows.iter().enumerate() {
                        if cancel.is_cancelled() {
                            break;
                        }
                        let window_start = DateTime::from_unix_timestamp(*window_start)?.to_timezone(tz);
                        let window_end = DateTime::from_unix_timestamp(*window_end)?.to_timezone(tz);
                        let prefix = |requests: usize, rows: usize| format!(
                            "Period {} of {} from {}, request {requests}, {rows} rows loaded",
                            window_number + 1,
                            windows.len(),
                            window_start.format(&date_format).unwrap_or_default(),
                        );

                        requests += 1;
                        let query = source.initial_query(&window_start, &window_end)?;
                        let mut next_response = call_unless_cancelled(request_manager, &query, cancel, progress, &prefix(requests, rows)).await?;

                        while let Some(response) = next_response {
                            let cq = source.continuation_query(&window_start, &window_end, &response);

                            // Self::handle_records(response, &response_iterator, &indexer, &mut data, &mut file)?;

                            for value in source.items(response)? {
                                let value = value?;
                                rows += 1;
                                let (start, end, is_final) = source.index(&value)?;
                                let start_date = start.to_date();
                                let key = Self::key_for(&start_date);

                                let data_set = if let Some(data_set) = data_set_map.get_mut(&key) {
                                    data_set
                                }
                                else {
                                    data_set_map.insert(key, OpenMonthlyFixedSlotDataSet::new(&source, &config, source.hash_key(), &start_date, tz)?);
                                    data_set_map.get_mut(&key).unwrap()
                                };

                                let update = if let Some(item) = data_set.data_set.data.get(&start) {
                                    let v = serde_json::to_string(&value)?;
                                    let c = serde_json::to_string(&item.value)?;
                                    v != c
                                }
                                else {
                                    true
                                };

                                if update {
                                    data_set.data_set.data.insert(start, Item {
                                            end,
                                            is_final,
                                            attributes:DataSetAttributes { cached: false},
                                            value,
                                        }
                                    );
                                    data_set.is_dirty = true;
                                }
                            }

                            // A source whose query has no end keeps returning samples after the window, in no useful order,
                            // so stop once the window is complete rather than when a sample after the window arrives.
                            // Windows do not cross month boundaries, so the window is all in one bucket.
                            let window_key = Self::key_for(&window_start.to_date());
                            if data_set_map.get(&window_key)
                                .is_some_and(|data_set| window_complete(&data_set.data_set.data, &window_start, &window_end))
                            {
                                break;
                            }

                            if let Some(continuation_query) = cq? {
                                requests += 1;
                                next_response = call_unless_cancelled(request_manager, &continuation_query, cancel, progress, &prefix(requests, rows)).await?;
                            }
                            else {
                                break;
                            }

                        }
                    }
                }
            }
            Ok(())
        }.await;

        // If the fetch was cancelled or failed we keep what we have, but the cached buckets may be missing data so are
        // marked stale.
        let cancelled = cancel.is_cancelled() || fetch_result.is_err();
        let now = DateTime::now(tz);

        for key in result_keys {
            let ods = data_set_map.remove(&key).unwrap();

            let stale = cancelled ||
                start > &ods.data_set.start_date.at_midnight(tz) ||
                (&now > end && end < &ods.data_set.next_date.at_midnight(tz));


            result.push(ods.flush(&config, stale)?);
        }

        for (_key, ods) in data_set_map {
            ods.flush(&config, true)?;
        }

        // Report a failure now that what was fetched has been saved.
        fetch_result?;
        Ok(result)
    }
}

struct OpenMonthlyFixedSlotDataSet<S: SlotSource>
{
    file: File,   // File is exclusive locked
    path: PathBuf,
    is_dirty: bool,  // Cache needs updating
    data_set: MonthlyFixedSlotDataSet<S>
}

impl<S: SlotSource> OpenMonthlyFixedSlotDataSet<S>
{
    /// Create a DataSet and load any cached value leaving the cache file locked.
    fn new(
        source: &S,
        config: &CacheConfig,
        hash_key: &str,
        date: &Date,
        tz: &'static Tz,
    ) -> anyhow::Result<Self> 
    {
        let month_bucket = config.path_for_month(hash_key, date)?;

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&month_bucket.path)?;
        file.lock()?;

        let data_set: MonthlyFixedSlotDataSet<S> = MonthlyFixedSlotDataSet::load(source, config, &mut file, month_bucket.start, month_bucket.next, tz)?;
        let path = month_bucket.path;

        Ok(Self{
            file,
            path,
            is_dirty: false,
            data_set,
        })
    }

    fn is_stale(&self, refresh_after: &Duration) -> anyhow::Result<bool> {
        if self.data_set.data.len() == 0 {
            return Ok(true);
        }

        let modified = self.file.metadata()?.modified()?;
        Ok(SystemTime::now()
            .duration_since(modified)
            .map(|age| &age > refresh_after)
            .unwrap_or(false) // If the file is modified in the future, do not refresh it.
        )
    }
    
    fn flush(mut self, config: &CacheConfig, is_stale: bool) -> anyhow::Result<MonthlyFixedSlotDataSet<S>> {
        if self.is_dirty {
            // We added data so update the cache

            if config.verbose 
            {
                println!("REWRITING FILE {}", self.path.display());
            }
            self.file.seek(SeekFrom::Start(0))?;
            self.file.set_len(0)?;
            for (_k, item) in &self.data_set.data {
                writeln!(self.file, "{}", serde_json::to_string(&item.value)?)?;
            }

            if is_stale {
                // We fetched some data for this month but we don't know if more data might be available so we set the file modify date
                // to the epoch so that if it is read later it will be treated as stale.
                self.file.set_times(FileTimes::new().set_modified(UNIX_EPOCH))?;
            }
        }

        Ok(self.data_set)
    }
}
#[cfg(test)]
mod tests {
    use super::{MIN_CACHED_RUN, fetch_windows};

    const SLOT: i64 = 1800;
    const DAY: i64 = 48 * SLOT;

    /// count consecutive final samples starting at slot first.
    fn run(first: i64, count: i64) -> Vec<(i64, i64, bool)> {
        (first..first + count).map(|slot| (slot * SLOT, (slot + 1) * SLOT, true)).collect()
    }

    #[test]
    fn with_nothing_cached_the_whole_range_is_fetched_split_at_months() {
        let month = 30 * DAY;
        assert_eq!(
            fetch_windows(&[], 0, 2 * month + DAY, &[month, 2 * month, 3 * month]),
            [(0, month), (month, 2 * month), (2 * month, 2 * month + DAY)]
        );
    }

    #[test]
    fn cached_data_at_the_start_or_in_the_middle_is_not_fetched_again() {
        // Days 0 and 3 of 6 are cached.
        let mut cached = run(0, 48);
        cached.extend(run(3 * 48, 48));
        assert_eq!(fetch_windows(&cached, 0, 6 * DAY, &[]), [(DAY, 3 * DAY), (4 * DAY, 6 * DAY)]);
    }

    #[test]
    fn short_runs_of_cached_data_are_fetched_again() {
        let cached = run(48, MIN_CACHED_RUN as i64 - 1);
        assert_eq!(fetch_windows(&cached, 0, 4 * DAY, &[]), [(0, 4 * DAY)]);

        let cached = run(48, MIN_CACHED_RUN as i64);
        assert_eq!(
            fetch_windows(&cached, 0, 4 * DAY, &[]),
            [(0, DAY), (DAY + MIN_CACHED_RUN as i64 * SLOT, 4 * DAY)]
        );
    }

    #[test]
    fn non_final_samples_are_always_fetched_again() {
        // Slot 20 of a day is not final, which splits the day into a run of 20 slots, too short to keep, and a run of
        // 27, which is kept. So slots 0 to 20, including the non-final one, are fetched.
        let mut cached = run(0, 48);
        cached[20].2 = false;
        assert_eq!(fetch_windows(&cached, 0, DAY, &[]), [(0, 21 * SLOT)]);
    }

    #[test]
    fn a_run_which_starts_before_the_range_still_counts() {
        let cached = run(0, 48);
        assert_eq!(fetch_windows(&cached, DAY - 2 * SLOT, 2 * DAY, &[]), [(DAY, 2 * DAY)]);
    }
}

#[cfg(test)]
mod window_complete_tests {
    use std::collections::BTreeMap;

    use sparko_graphql::types::DateTime;

    use super::{Item, window_complete};
    use crate::data_set::DataSetAttributes;

    const SLOT: i64 = 1800;

    fn time(slot: i64) -> DateTime {
        DateTime::from_unix_timestamp(slot * SLOT).unwrap()
    }

    /// Samples for the given slots, final unless listed in not_final.
    fn samples(slots: &[i64], not_final: &[i64]) -> BTreeMap<DateTime, Item<()>> {
        slots.iter()
            .map(|slot| (time(*slot), Item {
                end: time(slot + 1),
                is_final: !not_final.contains(slot),
                attributes: DataSetAttributes { cached: false },
                value: (),
            }))
            .collect()
    }

    #[test]
    fn a_window_is_complete_only_when_every_slot_is_present_and_final() {
        let all: Vec<i64> = (0..10).collect();
        assert!(window_complete(&samples(&all, &[]), &time(2), &time(8)));

        // Samples outside the window do not matter, even if they arrived first.
        assert!(window_complete(&samples(&[2, 3, 4, 5, 6, 7, 40, 41], &[]), &time(2), &time(8)));

        assert!(!window_complete(&samples(&[2, 3, 5, 6, 7], &[]), &time(2), &time(8)));
        assert!(!window_complete(&samples(&[3, 4, 5, 6, 7], &[]), &time(2), &time(8)));
        assert!(!window_complete(&samples(&[2, 3, 4, 5, 6], &[]), &time(2), &time(8)));
        assert!(!window_complete(&samples(&all, &[4]), &time(2), &time(8)));
    }
}

#[cfg(test)]
mod retry_tests {
    use super::is_transient;

    #[test]
    fn network_errors_gateway_timeouts_and_rate_limits_are_retried_but_not_other_errors() {
        use sparko_graphql::Error;

        // sparko_graphql uses a different version of reqwest, so its status codes are made from numbers.
        let http = |code: u16| Error::HttpError(code.try_into().unwrap());
        assert!(is_transient(&http(504)));
        assert!(is_transient(&http(502)));
        assert!(is_transient(&http(429)));
        assert!(!is_transient(&http(400)));
        assert!(!is_transient(&http(401)));
        assert!(!is_transient(&Error::InternalError(String::from("bad"))));
    }
}
