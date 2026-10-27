use std::{
    collections::HashSet,
    fs::{FileTimes, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    sync::Arc,
    time::{Duration, SystemTime},
};

use serde::{Serialize, de::DeserializeOwned};
use sparko_graphql::{GraphQLCaller, GraphQLQuery, GraphQLResponse};

use crate::{CacheConfig, OrderedMap};
use super::DataSetAttributes;

/// A DataSet which is a list of records where the records returned from the API are in ascending order and new records can be
/// appended to the end of the list.
///
/// Each record is final or not, see the indexer, e.g. a closed bill or an open one, which may still change. A refresh
/// fetches from the earliest record which is not final, or if they all are, from the latest, see fetch_from. With
/// drop_missing_provisional, a record which is not final and is no longer returned is removed, e.g. an open bill which
/// the server replaced with another, see missing_provisional.
pub struct OrderedListDataSet<K, O, V>
{
    pub data: OrderedMap<K, O, (DataSetAttributes, V)> ,
}

impl<K, O, V> OrderedListDataSet<K, O, V>
where
    K: Eq + std::hash::Hash + Ord + Clone + Send + Sync,
    O: Ord + Clone,
    V: DeserializeOwned + Serialize
{
    pub async fn new<R, Q, IQP, CQP, RI, IN, I>(
            hash_key: &str,
            refresh_after: Duration,
            initial_query_provider: IQP,
            continuation_query_provider: CQP,
            response_iterator: RI,
            indexer: IN,
            drop_missing_provisional: bool,
            config: &Arc<CacheConfig>,
            request_manager: &impl GraphQLCaller
        ) -> anyhow::Result<Self>
        where
            R: GraphQLResponse,
            Q: GraphQLQuery<R>,
            IQP: FnOnce(Option<&V>) -> Result<Q, sparko_graphql::Error>,
            CQP: Fn(&R) -> Option<Result<Q, sparko_graphql::Error>>,
            RI: Fn(R) -> I,
            IN: Fn(&V) -> (K, O, bool),
            I: IntoIterator<Item = V>,
    {
        let path = config.path_for_hash_key(hash_key);
    //     Self::init(
    //         config.path_for_hash_key(hash_key),
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
    //         IN: Fn(&V) -> (K, O, bool),
    //         I: IntoIterator<Item = V>,
    // {
        let mut data: OrderedMap<K, O, (DataSetAttributes, V)> = OrderedMap::new();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let _guard = file.lock()?;
        let modified = file.metadata()?.modified()?;
        let mut refresh =  SystemTime::now()
            .duration_since(modified)
            .map(|age| age > refresh_after)
            .unwrap_or(false); // If the file is modified in the future, do not refresh it.
        let mut rewrite = false;

        let lines = BufReader::new(&file).lines();
        for line in lines.map_while(Result::ok) {
            if config.verbose 
            {
                println!("READ {}", line);
            }

            match serde_json::from_str(&line) {
                Ok(value) => {
                    
                    let (index, order, _is_final) = indexer(&value);
                    data.insert(index, order, (DataSetAttributes { cached: true, }, value));

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
        
        // An offline profile only has what is cached.
        if refresh && !config.offline {

            let from = fetch_from(data.iter().map(|(_, (_, value))| indexer(value)));
            let last_record = from.as_ref().and_then(|(index, _)| data.get(index)).map(|(_, value)| value);
            let query = initial_query_provider(last_record)?;
            let mut response = request_manager.call(&query).await?;
            // The records returned, for drop_missing_provisional.
            let mut returned: HashSet<K> = HashSet::new();

            // Self::handle_records(response, response_iterator, indexer, &mut data, &mut file)?;

            // for value in response_iterator(response) {
            //         let (index, order) = indexer(&value);
            //     if ! data.contains_key(&index) {
            //         writeln!(file, "{}", serde_json::to_string(&value)?)?;
            //         data.insert(index, order, (DataSetAttributes { cached: false, }, value));
            //     }
            // }

            loop {
                let cq = continuation_query_provider(&response);

                // Self::handle_records(response, &response_iterator, &indexer, &mut data, &mut file)?;

                for value in response_iterator(response) {
                    let (index, order, _is_final) = indexer(&value);
                    returned.insert(index.clone());

                    // if let Some((a,x)) = data.get(&index);
                    
                    let update = if let Some((_attrs, current_value)) = data.get(&index) {
                        let v = serde_json::to_string(&value)?;
                        let c = serde_json::to_string(current_value)?;
                        if config.verbose {
                            println!("COMPARE \n{}\n{}\n={}", v, c, v!=c);
                        }
                        v != c
                    }
                    else {
                        true
                    };

                    if update {
                        // writeln!(file, "{}", serde_json::to_string(&value)?)?;
                        data.insert(index, order, (DataSetAttributes { cached: false, }, value));
                        rewrite = true;
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
            

            if drop_missing_provisional {
                let from_order = from.as_ref().map(|(_, order)| order);
                let missing = missing_provisional(data.iter().map(|(_, (_, value))| indexer(value)), from_order, &returned);
                if !missing.is_empty() {
                    crate::log_info!("Removed {} records which are no longer returned from {}", missing.len(), path.display());
                    for index in &missing {
                        data.remove(index);
                    }
                    rewrite = true;
                }
            }

            file.set_times(FileTimes::new().set_modified(SystemTime::now()))?;
        };

        if rewrite {
            if config.verbose 
            {
                println!("REWRITING FILE {}", path.display());
            }
            file.seek(SeekFrom::Start(0))?;
            file.set_len(0)?;
            for (_k, (_a, v)) in &data {
                writeln!(file, "{}", serde_json::to_string(&v)?)?;
            }
        }

        Ok(Self {
            data
        })
        
    }
}

/// The record to fetch from, as its key and order, from each record's key, order and whether it is final: the earliest
/// record which is not final, as it may have changed, if it is before the latest final one, otherwise the latest final
/// one, as new records come after it. None, to fetch them all, if none are final.
fn fetch_from<K, O: Ord>(records: impl Iterator<Item = (K, O, bool)>) -> Option<(K, O)> {
    let mut latest_final: Option<(K, O)> = None;
    let mut earliest_provisional: Option<(K, O)> = None;
    for (key, order, is_final) in records {
        if is_final {
            if latest_final.as_ref().is_none_or(|(_, latest)| order > *latest) {
                latest_final = Some((key, order));
            }
        }
        else if earliest_provisional.as_ref().is_none_or(|(_, earliest)| order < *earliest) {
            earliest_provisional = Some((key, order));
        }
    }
    match (latest_final, earliest_provisional) {
        (Some(latest), Some(earliest)) if earliest.1 < latest.1 => Some(earliest),
        (latest, _) => latest,
    }
}

/// The keys of the records, from each one's key, order and whether it is final, which are not final, from the order
/// fetched from, or all of them if from is None, and were not returned.
fn missing_provisional<K: Eq + std::hash::Hash, O: Ord>(records: impl Iterator<Item = (K, O, bool)>, from: Option<&O>,
    returned: &HashSet<K>) -> Vec<K>
{
    records
        .filter(|(key, order, is_final)| !is_final && from.is_none_or(|from| order >= from) && !returned.contains(key))
        .map(|(key, _, _)| key)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{fetch_from, missing_provisional};

    /// Bills as (id, from date, closed), as they were cached when an open statement was replaced, see BillDataSet.
    fn bills() -> Vec<(&'static str, &'static str, bool)> {
        vec![
            ("64400356", "2021-05-04", true),
            ("464800541", "2026-09-25", true),
            ("465467620", "2026-09-28", false),
            ("473394964", "2026-09-28", true),
            ("474269026", "2026-10-25", false),
        ]
    }

    #[test]
    fn a_refresh_is_from_the_earliest_record_which_is_not_final_by_order_not_key() {
        // The open statement from 28 Sep, rather than the 2021 bill, whose id is the largest as text.
        let from = fetch_from(bills().into_iter());
        assert_eq!(from.map(|(_, order)| order), Some("2026-09-28"));
        // With them all final, from the latest.
        let closed = bills().into_iter().map(|(id, date, _)| (id, date, true));
        assert_eq!(fetch_from(closed), Some(("474269026", "2026-10-25")));
        // With none final, all of them.
        let open = bills().into_iter().map(|(id, date, _)| (id, date, false));
        assert_eq!(fetch_from(open), None);
    }

    #[test]
    fn records_which_are_not_final_and_are_no_longer_returned_are_missing() {
        let returned: HashSet<&str> = ["473394964", "474269026"].into_iter().collect();
        assert_eq!(missing_provisional(bills().into_iter(), Some(&"2026-09-28"), &returned), ["465467620"]);
        // Final records, and those before the fetch, are never missing.
        assert!(missing_provisional(bills().into_iter(), Some(&"2026-10-25"), &returned).is_empty());
    }
}
