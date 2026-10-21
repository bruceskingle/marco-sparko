use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    time::{Duration, SystemTime},
};

use anyhow::anyhow;
use sparko_graphql::{GraphQLCaller, GraphQLQuery, GraphQLResponse};

use crate::CacheConfig;
use super::DataSetAttributes;

/// A DataSet which is a list of records where each record is returned from the API in a single call
pub struct MultiQueryDataSet<R>
{
    pub data: Vec<R>,
    pub _attributes: DataSetAttributes,
}

impl<R> MultiQueryDataSet<R>
where
    R: GraphQLResponse,
{
   pub async fn new<Q, QI>(
            hash_key: &str,
            refresh_after: Duration,
            query_iterator: QI,
            config: CacheConfig,
            request_manager: &impl GraphQLCaller
        ) -> anyhow::Result<Self>
        where
            Q: GraphQLQuery<R>,
            QI: IntoIterator<Item = Result<Q, sparko_graphql::Error>>,
    {

        let mut data: Vec<R> = Vec::new();
        let path = config.path_for_hash_key(hash_key);

        let mut _attributes = DataSetAttributes { cached: true, };
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
            _attributes.cached = false;

            for query_result in query_iterator {
                match query_result {
                    Ok(query) => {
                        let response = request_manager.call(&query).await?;
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
            _attributes,
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
