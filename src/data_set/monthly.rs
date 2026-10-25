use std::{
    collections::BTreeMap,
    fs::{FileTimes, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Serialize, de::DeserializeOwned};
use sparko_graphql::{
    GraphQLCaller, GraphQLQuery, GraphQLResponse, ProgressMonitor,
    types::{Date, DateTime},
};

use crate::CacheConfig;
use super::DataSetAttributes;

/// A DataSet which is a monthly bucket containing a list of records where each record has a DateTime ordering and the records 
/// returned from the API are in ascending order
pub struct MonthlyDataSet<V>
{
    pub start_date: Date,
    pub next_date: Date,
    pub data: BTreeMap<DateTime, (DataSetAttributes, V)> ,
}

impl<V> MonthlyDataSet<V>
where
    V: DeserializeOwned + Serialize
{
    
    /// The month containing date, from the cache, fetched again if it is more than refresh_after old and not yet
    /// complete, or if force is set. A month is complete once it has been fetched more than refresh_after after it
    /// ended, so it has had time to get the data for its end, and then it is not fetched again unless forced. An empty
    /// month counts as fetched, but a new file is dated the epoch, so if the fetch fails it is fetched again next time.
    /// With force, the whole month is fetched rather than from the last final record.
    pub async fn new<R, Q, IQP, CQP, RI, IN, I>(
            hash_key: &str,
            date: &Date,
            refresh_after: Duration,
            force: bool,
            initial_query_provider: IQP,
            continuation_query_provider: CQP,
            response_iterator: RI,
            indexer: IN,
            config: &CacheConfig,
            request_manager: &impl GraphQLCaller,
            progress: &(impl ProgressMonitor + ?Sized),
        ) -> anyhow::Result<Self>
        where
            R: GraphQLResponse,
            Q: GraphQLQuery<R>,
            IQP: FnOnce(&Date, Option<&V>) -> Result<Q, sparko_graphql::Error>,
            CQP: Fn(&Date, &R) -> Option<Result<Q, sparko_graphql::Error>>,
            RI: Fn(R) -> I,
            IN: Fn(&V) -> (DateTime, bool),
            I: IntoIterator<Item = V>,
    {
        let month_bucket = config.path_for_month(hash_key, date)?;
        
    //     Self::init(
    //         path,
    //         refresh_after,
    //         initial_query_provider,
    //         continuation_query_provider,
    //         response_iterator,
    //         indexer,
    //         config,
    //         request_manager
    //     ).await
    // }

    // async fn init<R, Q, IQP, CQP, RI, IN, I>(
    //         path: PathBuf,
    //         refresh_after: Duration,
    //         initial_query_provider: IQP,
    //         continuation_query_provider: CQP,
    //         response_iterator: RI,
    //         indexer: IN,
    //         config: &Arc<CacheManager>,
    //         request_manager: &impl GraphQLCaller
    //     ) -> anyhow::Result<Self>
    //     where
    //         R: GraphQLResponse,
    //         Q: GraphQLQuery<R>,
    //         IQP: FnOnce(Option<&V>) -> Result<Q, sparko_graphql::Error>,
    //         CQP: Fn(&R) -> Option<Result<Q, sparko_graphql::Error>>,
    //         RI: Fn(R) -> I,
    //         IN: Fn(&V) -> (DateTime, bool),
    //         I: IntoIterator<Item = V>,
    // {
        let mut data: BTreeMap<DateTime, (DataSetAttributes, V)> = BTreeMap::new();
        let created = !month_bucket.path.exists();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&month_bucket.path)?;
        let _guard = file.lock()?;
        if created {
            // Nothing has been fetched for this month yet, so it is stale until a fetch of it succeeds.
            file.set_times(FileTimes::new().set_modified(UNIX_EPOCH))?;
        }
        let modified = file.metadata()?.modified()?;
        let stale = SystemTime::now()
            .duration_since(modified)
            .map(|age| age > refresh_after)
            .unwrap_or(false); // If the file is modified in the future, do not refresh it.
        // The end of the month, in UTC, which is close enough as the month is given refresh_after to complete.
        let month_end = UNIX_EPOCH + Duration::from_secs(
            month_bucket.next.midnight().assume_utc().unix_timestamp().max(0) as u64);
        let complete = modified > month_end + refresh_after;
        let mut refresh = force || (stale && !complete);
        let mut rewrite = false;

        let mut last_final_index = None;
        let mut first_non_final_index = None;
        let lines = BufReader::new(&file).lines();
        for line in lines.map_while(Result::ok) {
            if config.verbose 
            {
                println!("READ {}", line);
            }

            match serde_json::from_str(&line) {
                Ok(value) => {
                    
                    let (index, is_final) = indexer(&value);

                    if is_final {
                        if let Some(current_final_index) = &last_final_index {
                            if index > *current_final_index {
                                last_final_index = Some(index.clone());
                            }
                        }
                        else {
                            last_final_index = Some(index.clone());
                        }
                    }
                    else {
                        if let Some(current) = &first_non_final_index {
                            if index < *current {
                                first_non_final_index = Some(index.clone());
                            }
                        }
                        else {
                            first_non_final_index = Some(index.clone());
                        }
                    }
                    data.insert(index, (DataSetAttributes { cached: true, }, value));

                    // if is_final {
                    //     // last_record = Some(&value); //Some(&data.last().unwrap().1 .1);
                    //     if let Some((_k, (_a,r))) = data.last() {
                    //             last_record = Some(r)
                    //         }
                    // }
                },
                Err(e) => {
                    println!("ERROR: failed to read record {:?}", e);
                    refresh = true;

                    // Fetched again from the start, and stale until that succeeds.
                    file.seek(SeekFrom::Start(0))?;
                    file.set_len(0)?;
                    file.set_times(FileTimes::new().set_modified(UNIX_EPOCH))?;
                    data.clear();
                    last_final_index = None;
                    first_non_final_index = None;
                    break;
                },
            }
        }

        if refresh  {

            // With force everything is fetched again, in case what was cached is wrong, and replaces what was cached,
            // so records the server no longer has are dropped. The file is only rewritten if the fetch succeeds.
            if force {
                data.clear();
                rewrite = true;
            }
            let last_index = if force {
                None
            }
            else if let Some(l) = &last_final_index {
                if let Some(f) = &first_non_final_index {
                    if f < l {
                        first_non_final_index
                    }
                    else {
                        last_final_index
                    }
                }
                else {
                    last_final_index
                }
            }
            else {
                None
            };

            let last_record = if let Some(index) = last_index {
                if let Some((_k, r )) = data.get(&index) {
                    Some(r)
                }
                else {
                    None
                }

                // let x = Some(data.get(&index).unwrap().1);
                // Some(data.get(&index).unwrap().1 .1)
            }
            else {
                None
            };
            let query = initial_query_provider(&month_bucket.start, last_record)?;
            let mut response = request_manager.call3(&query, None, progress).await?;

            // Self::handle_records(response, response_iterator, indexer, &mut data, &mut file)?;

            // for value in response_iterator(response) {
            //         let (index, order) = indexer(&value);
            //     if ! data.contains_key(&index) {
            //         writeln!(file, "{}", serde_json::to_string(&value)?)?;
            //         data.insert(index, order, (DataSetAttributes { cached: false, }, value));
            //     }
            // }

            loop {
                let cq = continuation_query_provider(&month_bucket.start, &response);

                // Self::handle_records(response, &response_iterator, &indexer, &mut data, &mut file)?;

                for value in response_iterator(response) {
                    let (index, _is_final) = indexer(&value);

                    // if index.date() > *end_date {
                    //     cq = None;  // force a break from the outer loop
                    //     break;
                    // }

                    let date: time::Date = index.date();
                    if date >= *month_bucket.start && date < *month_bucket.next {

                        let update = if let Some((_attrs, current_value)) = data.get(&index) {
                            let v = serde_json::to_string(&value)?;
                            let c = serde_json::to_string(current_value)?;
                            println!("COMPARE \n{}\n{}\n={}", v, c, v!=c);
                            v != c
                        }
                        else {
                            true
                        };

                        if update {
                            data.insert(index, (DataSetAttributes { cached: false, }, value));
                            rewrite = true;
                        }
                    }
                }

                if let Some(continuation_query) = cq {
                    let query = continuation_query?;

                    response = request_manager.call3(&query, None, progress).await?;

                    
                }
                else {
                    break;
                }
            }
            

            file.set_times(FileTimes::new().set_modified(SystemTime::now()))?;
        };

        if rewrite {
            if config.verbose 
            {
                println!("REWRITING FILE {}", month_bucket.path.display());
            }
            file.seek(SeekFrom::Start(0))?;
            file.set_len(0)?;
            for (_k, (_a, v)) in &data {
                writeln!(file, "{}", serde_json::to_string(&v)?)?;
            }
        }

        Ok(Self {
            start_date: month_bucket.start,
            next_date: month_bucket.next,
            data
        })
        
    }
}
