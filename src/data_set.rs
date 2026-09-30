use std::{fs::OpenOptions, io::{BufReader, Seek, SeekFrom, Write}, sync::Arc};

use std::io::BufRead;
use serde::{Serialize, de::DeserializeOwned};
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
        let reader = BufReader::new(&file);
        let data = match serde_json::from_reader(reader) {
            Ok(data) => {
                data
            },
            Err(e) => {
                println!("Failed to read DataSet from {}: {:?}", path.display(), e);

                let data = request_manager.call(&query_provider()?).await?;

                let mut file = &file;
                file.seek(SeekFrom::Start(0))?;
                file.set_len(0)?;
                writeln!(file, "{}", serde_json::to_string(&data)?)?;

                data
            },
        };

        Ok(Self {
            data
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
            refresh: bool,
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
            IN: Fn(&V) -> (K, O),
            I: IntoIterator<Item = V>,
    {

        let mut data: OrderedMap<K, O, (DataSetAttributes, V)> = OrderedMap::new();
        let mut refresh = refresh;
        let mut path = config.dir_path.clone();
        path.push(hash_key);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let _guard = file.lock()?;

        let lines = BufReader::new(&file).lines();
        for line in lines.map_while(Result::ok) {
            if config.verbose 
            {
                println!("READ {}", line);
            }

            match serde_json::from_str(&line) {
                Ok(value) => {
                    
                    let (index, order) = indexer(&value);
                    data.insert(index, order, (DataSetAttributes { cached: true, }, value));
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
        
        if refresh  {

            let last_record = if let Some((_k, (_a,r))) = data.last() {
                Some(r)
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
                    let (index, order) = indexer(&value);
                    if ! data.contains_key(&index) {
                        writeln!(file, "{}", serde_json::to_string(&value)?)?;
                        data.insert(index, order, (DataSetAttributes { cached: false, }, value));
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
            data
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