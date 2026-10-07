use std::{fs::{FileTimes, OpenOptions}, io::{BufReader, Seek, SeekFrom, Write}, sync::Arc, time::{Duration, SystemTime}};

use std::io::BufRead;
use indexmap::IndexMap;
use serde::{Serialize, de::DeserializeOwned};
use anyhow::anyhow;
use sparko_graphql::{AuthenticatedRequestManager, GraphQLQuery, GraphQLResponse};

use crate::{CacheManager, octopus::token::OctopusTokenManager, OrderedMap};

pub struct DataSetAttributes {
    pub cached: bool,
}

// pub struct DataSetConfig {
//     pub dir_path: PathBuf,
//     pub verbose: bool,
// }

// pub trait TDataSet {
//     fn save(&self);
//     fn type_name(&self) -> &str;
//     fn last_updated(&self) -> time::OffsetDateTime;
//     fn last_fetched(&self) -> time::OffsetDateTime;
// }

// pub trait TSingleRecordDataSet<T>: TDataSet {
//     fn record(&self) -> &T;
// }

// pub struct DataSet {
//     type_name: String,
//     last_updated: time::OffsetDateTime,
// }

// impl DataSet {
//     pub fn new(type_name: &str) -> DataSet {
//         let type_name = type_name.to_string();
//         let last_updated = time::OffsetDateTime::now_utc();
//         DataSet {
//             type_name,
//             last_updated
//         }
//     }
// }

// impl TDataSet for DataSet {
//     fn save(&self) {
//         todo!()
//     }

//     fn last_updated(&self) -> time::OffsetDateTime {
//         self.last_updated
//     }
    
//     fn type_name(&self) -> &str {
//         &self.type_name
//     }
// }



pub struct SingleRecordDataSet<R: GraphQLResponse>
{
    pub data: R,
}

impl<R: GraphQLResponse> SingleRecordDataSet<R>
{
    pub async fn new<Q, QP>(
            hash_key: &str,
            refresh_after: Duration,
            query_provider: QP,
            config: &Arc<CacheManager>,
            request_manager: &AuthenticatedRequestManager<OctopusTokenManager>
        ) -> anyhow::Result<Self>
        where
            Q: GraphQLQuery<R>,
            QP: FnOnce() -> Result<Q, sparko_graphql::Error>,
    {
        let mut path = config.dir_path.clone();
        path.push(hash_key);

        let file = OpenOptions::new()
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

        let mut data: Option<R> = None; // data is always initialized but the compiler can't see it so I need to use an Option.

        // Need to test refresh twice because a read error sets it to true.
        if !refresh {

            let reader = BufReader::new(&file);
            match serde_json::from_reader(reader) {
                Ok(value) => {
                    data = Some(value);
                },
                Err(e) => {
                    println!("Failed to read DataSet from {}: {:?}", path.display(), e);
                    refresh = true;
                },
            }
        }

        if refresh {

            let value = request_manager.call(&query_provider()?).await?;

            let mut file = &file;
            file.seek(SeekFrom::Start(0))?;
            file.set_len(0)?;
            writeln!(file, "{}", serde_json::to_string(&value)?)?;

            data = Some(value);
        }

        Ok(Self {
            data: data.unwrap()
        })
        
    }
}

/// A DataSet which is a list of records where the records returned from the API are in ascending order and new records can be
/// appended to the end of the list.
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
            config: &Arc<CacheManager>,
            request_manager: &AuthenticatedRequestManager<OctopusTokenManager>
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

        let mut data: OrderedMap<K, O, (DataSetAttributes, V)> = OrderedMap::new();
        let mut path = config.dir_path.clone();
        path.push(hash_key);

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
                    
                    let (index, order, is_final) = indexer(&value);

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
        
        if refresh  {
            // file.set_times(FileTimes::new().set_modified(SystemTime::now()))?;

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
                if let Some((_k, (r))) = data.get(&index) {
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
            let query = initial_query_provider(last_record)?;
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
                let cq = continuation_query_provider(&response);

                // Self::handle_records(response, &response_iterator, &indexer, &mut data, &mut file)?;

                for value in response_iterator(response) {
                    let (index, order, is_final) = indexer(&value);

                    // if let Some((a,x)) = data.get(&index);
                    
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




/// A DataSet which is a list of records where the records returned paginated from the API
pub struct ListDataSet<K, V>
{
    pub data: IndexMap<K, V> ,
    pub attributes: DataSetAttributes,
}

impl<K, V> ListDataSet<K, V>
where
    K: Eq + std::hash::Hash + Ord + Clone + Send + Sync,
    V: DeserializeOwned + Serialize
{
   pub async fn new<R, Q, IQP, CQP, RI, IN, I>(
            hash_key: &str,
            refresh_after: Option<Duration>,
            initial_query_provider: IQP,
            continuation_query_provider: CQP,
            response_iterator: RI,
            indexer: IN,
            config: &Arc<CacheManager>,
            request_manager: &AuthenticatedRequestManager<OctopusTokenManager>
        ) -> anyhow::Result<Self>
        where
            R: GraphQLResponse,
            Q: GraphQLQuery<R>,
            IQP: FnOnce() -> Result<Q, sparko_graphql::Error>,
            CQP: Fn(&R) -> Option<Result<Q, sparko_graphql::Error>>,
            RI: Fn(R) -> I,
            IN: Fn(&V) -> K,
            I: IntoIterator<Item = V>,
    {

        let mut attributes = DataSetAttributes { cached: true, };
        let mut data: IndexMap<K, V> = IndexMap::new();
        let mut path = config.dir_path.clone();
        path.push(hash_key);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let _guard = file.lock()?;
        let modified = file.metadata()?.modified()?;
        let mut refresh =  if let Some(refresh_after) = refresh_after {
            SystemTime::now()
                .duration_since(modified)
                .map(|age| age > refresh_after)
                .unwrap_or(false) // If the file is modified in the future, do not refresh it.
        } else {
            false
        };

        if !refresh {
            let lines = BufReader::new(&file).lines();
            for line in lines.map_while(Result::ok) {
                if config.verbose 
                {
                    println!("READ {}", line);
                }

                match serde_json::from_str(&line) {
                    Ok(value) => {
                        
                        let index = indexer(&value);
                        data.insert(index, value);
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
        }



        if data.len() == 0 {
            refresh = true;
        }
        
        if refresh  {
            let query = initial_query_provider()?;
            let mut response = request_manager.call(&query).await?;

            attributes.cached = false;

            loop {
                let cq = continuation_query_provider(&response);

                for value in response_iterator(response) {
                    let index = indexer(&value);
                    if ! data.contains_key(&index) {
                        writeln!(file, "{}", serde_json::to_string(&value)?)?;
                        data.insert(index, value);
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
            

        };

        Ok(Self {
            data,
            attributes,
        })
        
    }
}

// /// A DataSet which is a list of records where the records are returned from the API in a single call
// pub struct SimpleListDataSet<V>
// {
//     pub data: Vec<V>,
//     pub attributes: DataSetAttributes,
// }

// impl<V> SimpleListDataSet<V>
// where
//     V: DeserializeOwned + Serialize
// {
//    pub async fn new<R, Q, IQP, RI, IN, I>(
//             hash_key: &str,
//             refresh: bool,
//             initial_query_provider: IQP,
//             response_iterator: RI,
//             config: &Arc<CacheManager>,
//             request_manager: &AuthenticatedRequestManager<OctopusTokenManager>
//         ) -> anyhow::Result<Self>
//         where
//             R: GraphQLResponse,
//             Q: GraphQLQuery<R>,
//             IQP: FnOnce() -> Result<Q, sparko_graphql::Error>,
//             RI: Fn(R) -> I,
//             I: IntoIterator<Item = V>,
//     {

//         let mut data: Vec<V> = Vec::new();
//         let mut refresh = refresh;
//         let mut path = config.dir_path.clone();
//         path.push(hash_key);

//         let mut attributes = DataSetAttributes { cached: true, };
//         let mut file = OpenOptions::new()
//             .read(true)
//             .write(true)
//             .create(true)
//             .open(&path)?;
//         let _guard = file.lock()?;

//         let lines = BufReader::new(&file).lines();
//         for line in lines.map_while(Result::ok) {
//             if config.verbose 
//             {
//                 println!("READ {}", line);
//             }

//             match serde_json::from_str(&line) {
//                 Ok(value) => {
//                     data.push(value);
//                 },
//                 Err(e) => {
//                     println!("ERROR: failed to read record {:?}", e);
//                     refresh = true;

//                     file.seek(SeekFrom::Start(0))?;
//                     file.set_len(0)?;
//                     data.clear();
//                     break;
//                 },
//             }
//         }
        

        // if data.len() == 0 {
        //     refresh = true;
        // }
//         if refresh  {

//             attributes.cached = false;
//             let query = initial_query_provider()?;
//             let mut response = request_manager.call(&query).await?;

//                 for value in response_iterator(response) {
//                     writeln!(file, "{}", serde_json::to_string(&value)?)?;
//                     data.push(value);
//                 }
            

//         };

//         Ok(Self {
//             data,
//             attributes,
//         })
        
//     }

/// A DataSet which is a list of records where each record is returned from the API in a single call
pub struct MultiQueryDataSet<R>
{
    pub data: Vec<R>,
    pub attributes: DataSetAttributes,
}

impl<R> MultiQueryDataSet<R>
where
    R: GraphQLResponse,
{
   pub async fn new<Q, QI>(
            hash_key: &str,
            refresh_after: Duration,
            query_iterator: QI,
            config: &Arc<CacheManager>,
            request_manager: &AuthenticatedRequestManager<OctopusTokenManager>
        ) -> anyhow::Result<Self>
        where
            Q: GraphQLQuery<R>,
            QI: IntoIterator<Item = Result<Q, sparko_graphql::Error>>,
    {

        let mut data: Vec<R> = Vec::new();
        let mut path = config.dir_path.clone();
        path.push(hash_key);

        let mut attributes = DataSetAttributes { cached: true, };
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

        // Need to test refresh twice because a read error sets it to true.
        if !refresh {
            
            let lines = BufReader::new(&file).lines();
            for line in lines.map_while(Result::ok) {
                if config.verbose 
                {
                    println!("READ {}", line);
                }

                match serde_json::from_str(&line) {
                    Ok(value) => {
                        data.push(value);
                    },
                    Err(e) => {
                        println!("ERROR: failed to read record {:?}", e);
                        refresh = true;
                        break;
                    },
                }
            }
        }



        if data.len() == 0 {
            refresh = true;
        }
        
        if refresh  {
            file.seek(SeekFrom::Start(0))?;
            file.set_len(0)?;
            data.clear();
            attributes.cached = false;

            for query_result in query_iterator {
                match query_result {
                    Ok(query) => {
                        let mut response = request_manager.call(&query).await?;
                        writeln!(file, "{}", serde_json::to_string(&response)?)?;
                        data.push(response);
                    },
                    Err(err) => {
                        return Err(anyhow!(format!("Failed to get query {}", err)));
                    },
                }
            }
        }

        Ok(Self {
            data,
            attributes,
        })
        
    }




    // fn handle_records<R, RI, IN, I>(
    //     response: R,
    //     response_iterator: RI,
    //     indexer: IN,
    //     data: &mut OrderedMap<K, O, (DataSetAttributes, V)>,
    //     file: &mut std::fs::File
    // )   -> anyhow::Result<()>
    
    //     where
    //         R: GraphQLResponse,
    //         RI: FnOnce(R) -> I,
    //         IN: Fn(&V) -> (K, O),
    //         I: IntoIterator<Item = V>,
    // {
    //     for value in response_iterator(response) {
    //         let (index, order) = indexer(&value);
    //         if ! data.contains_key(&index) {
    //             writeln!(file, "{}", serde_json::to_string(&value)?)?;
    //             data.insert(index, order, (DataSetAttributes { cached: false, }, value));
    //         }
    //     }
    //     Ok(())
    // }
}


// impl<T: Serialize> TDataSet for ListDataSet<T> {
//     fn save(&self) {
//         todo!()
//     }

//     fn last_updated(&self) -> time::OffsetDateTime {
//         self.last_updated
//     }

//     fn last_fetched(&self) -> time::OffsetDateTime {
//         self.last_fetched
//     }
    
//     fn type_name(&self) -> &str {
//         &self.type_name
//     }
// }

// impl<T> TSingleRecordDataSet<T> for ListDataSet<T>
// where T: Serialize,
// {
//     // fn as_data_set(&self) -> &impl TDataSet {
//     //     &self.data_set
//     // }

//     fn record(&self) -> &T {
//         &self.record
//     }
// }