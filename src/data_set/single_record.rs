use std::{
    fs::OpenOptions,
    io::{BufReader, Seek, SeekFrom, Write},
    time::{Duration, SystemTime},
};

use sparko_graphql::{GraphQLCaller, GraphQLQuery, GraphQLResponse};

use crate::CacheConfig;

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
            config: &CacheConfig,
            request_manager: &impl GraphQLCaller
        ) -> anyhow::Result<Self>
        where
            Q: GraphQLQuery<R>,
            QP: FnOnce() -> Result<Q, sparko_graphql::Error>,
    {
        let path = config.path_for_hash_key(hash_key);
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
                    crate::log_warning!("Failed to read the cached {}, fetching it again: {e:?}", path.display());
                    refresh = true;
                },
            }
        }

        // An offline profile only has what is cached.
        if refresh && !config.offline {

            let value = request_manager.call(&query_provider()?).await?;

            let mut file = &file;
            file.seek(SeekFrom::Start(0))?;
            file.set_len(0)?;
            writeln!(file, "{}", serde_json::to_string(&value)?)?;

            data = Some(value);
        }

        Ok(Self {
            data: data.ok_or_else(|| anyhow::anyhow!("{} is not in this offline profile's data", path.display()))?
        })
        
    }
}
