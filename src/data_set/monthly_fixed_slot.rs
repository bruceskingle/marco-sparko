use std::{
    collections::{BTreeMap, HashMap}, fs::{File, FileTimes, OpenOptions}, io::{BufRead, BufReader, Seek, SeekFrom, Write}, path::PathBuf, pin::pin, time::{Duration, SystemTime, UNIX_EPOCH},
};

use futures::future::{Either, select};

use serde::{Serialize, de::DeserializeOwned};
use sparko_graphql::{
    AuthenticatedRequestManager, GraphQLQuery, GraphQLResponse,
    types::{Date, DateTime},
};
use time_tz::Tz;

use crate::{CacheConfig, octopus::token::OctopusTokenManager};

use super::{CancelToken, DataSetAttributes};

/// Make a request, returning None without waiting for the response if the fetch is cancelled first.
async fn call_unless_cancelled<Q: GraphQLQuery<R>, R: GraphQLResponse>(
    request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
    query: &Q,
    cancel: &CancelToken,
) -> anyhow::Result<Option<R>> {
    match select(pin!(request_manager.call(query)), pin!(cancel.cancelled())).await {
        Either::Left((response, _)) => Ok(Some(response?)),
        Either::Right(_) => Ok(None),
    }
}

pub trait SlotSource {
    type Value: DeserializeOwned + Serialize + Clone;
    type Response: GraphQLResponse;
    type Query: GraphQLQuery<Self::Response>;

    fn hash_key(&self) -> &str;
    fn initial_query(&mut self, start: &DateTime) -> anyhow::Result<Self::Query>;
    fn continuation_query(&mut self, start: &DateTime, resp: &Self::Response) -> anyhow::Result<Option<Self::Query>>;
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
                Err(e) => {
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
        request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
        start: &DateTime,
        end: &DateTime,
        cancel: &CancelToken,
    ) -> anyhow::Result<Vec<MonthlyFixedSlotDataSet<S>>>
    {
        let mut result_keys = Vec::new();
        let mut result = Vec::new();
        let mut data_set_map = HashMap::new();

        {
            // scope this start_date
            let mut start_date = start.to_date();

            // end is exclusive so a bucket starting at end is not needed
            while start_date.at_midnight(tz) < *end {
                let open_data_set: OpenMonthlyFixedSlotDataSet<S> = OpenMonthlyFixedSlotDataSet::new(&source, &config, source.hash_key(), &start_date, tz)?;
                
                let key = Self::key_for(&start_date);
                result_keys.push(key);


                start_date = open_data_set.data_set.next_date.clone();
                data_set_map.insert(key, open_data_set);

            }
        }

        fn all_complete<S: SlotSource>(result_keys: &Vec<i32>, data_set_map: &HashMap<i32, OpenMonthlyFixedSlotDataSet<S>>) -> bool {
            for key in result_keys {
                let ds = &data_set_map.get(key).unwrap().data_set;
                if ! ds.is_complete {
                    return false;
                }
            }
            true
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


        if let Some(refresh_start) = all_up_to_date(&refresh_after, &result_keys, &data_set_map)? {
            if &refresh_start < end {
                let refresh_start = if &refresh_start < start { start } else {&refresh_start};
                let query = source.initial_query(refresh_start)?;
                let mut next_response = call_unless_cancelled(request_manager, &query, cancel).await?;

                while let Some(response) = next_response {
                    let cq = source.continuation_query(&refresh_start, &response);

                    // Self::handle_records(response, &response_iterator, &indexer, &mut data, &mut file)?;

                    for value in source.items(response)? {
                        let value = value?;
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

                    if all_complete(&result_keys, &data_set_map) {
                        break;
                    }

                    if let Some(continuation_query) = cq? {
                        next_response = call_unless_cancelled(request_manager, &continuation_query, cancel).await?;
                    }
                    else {
                        break;
                    }

                }
            }
        }

        // If the fetch was cancelled we return what we have, but the cached buckets may be missing data so are marked stale.
        let cancelled = cancel.is_cancelled();
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