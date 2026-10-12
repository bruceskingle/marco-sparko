use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    sync::Arc,
    time::{Duration, SystemTime},
};

use indexmap::IndexMap;
use serde::{Serialize, de::DeserializeOwned};
use sparko_graphql::{AuthenticatedRequestManager, GraphQLQuery, GraphQLResponse};

use crate::{CacheManager, octopus::token::OctopusTokenManager};
use super::DataSetAttributes;

/// A DataSet which is a list of records where the records returned paginated from the API
pub struct ListDataSet<K, V>
{
    pub data: IndexMap<K, V> ,
    pub _attributes: DataSetAttributes,
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
        let path = config.path_for_hash_key(hash_key);

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
            _attributes: attributes,
        })
        
    }
}
