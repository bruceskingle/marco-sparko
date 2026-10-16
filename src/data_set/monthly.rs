use std::{
    collections::BTreeMap,
    fs::{FileTimes, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    time::{Duration, SystemTime},
};

use serde::{Serialize, de::DeserializeOwned};
use sparko_graphql::{
    AuthenticatedRequestManager, GraphQLQuery, GraphQLResponse,
    types::{Date, DateTime},
};

use crate::{CacheConfig, octopus::token::OctopusTokenManager};
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
    
    pub async fn new<R, Q, IQP, CQP, RI, IN, I>(
            hash_key: &str,
            date: &Date,
            refresh_after: Duration,
            initial_query_provider: IQP,
            continuation_query_provider: CQP,
            response_iterator: RI,
            indexer: IN,
            config: &CacheConfig,
            request_manager: &AuthenticatedRequestManager<OctopusTokenManager>
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
    //         request_manager: &AuthenticatedRequestManager<OctopusTokenManager>
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
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&month_bucket.path)?;
        let _guard = file.lock()?;
        let modified = file.metadata()?.modified()?;
        let mut refresh =  SystemTime::now()
            .duration_since(modified)
            .map(|age| age > refresh_after)
            .unwrap_or(false); // If the file is modified in the future, do not refresh it.
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

                    file.seek(SeekFrom::Start(0))?;
                    file.set_len(0)?;
                    data.clear();
                    break;
                },
            }
        }

        if data.len() == 0 {
            refresh = true;
        }
        
        if refresh  {

            let last_index = if let Some(l) = &last_final_index {
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
            let mut response = request_manager.call(&query).await?;

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

                    response = request_manager.call(&query).await?;

                    
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
