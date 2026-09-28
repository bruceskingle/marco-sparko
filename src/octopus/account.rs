use std::sync::Arc;

use crate::CacheManager;
use crate::data_set::{SingleRecordDataSet};

use super::graphql::account;
use super::RequestManager;

pub struct AccountManager {
    pub data_set: SingleRecordDataSet<account::viewer::Response>,
    pub default_account_id: String,
}

impl AccountManager {
    pub async fn new(config: &Arc<CacheManager>, request_manager: &Arc<RequestManager>)  -> anyhow::Result<Self> {
        let hash_key = format!("#Viewer");
        let data_set = SingleRecordDataSet::new(&hash_key, || account::viewer::Query::new(), config, request_manager).await?;
        let default_account_id = data_set.data.viewer_.accounts_.get(0).unwrap().number_.clone();
        Ok(Self {
            data_set,
            default_account_id,
        })
    }

    pub fn get_default_account_id(&self) -> &String {
        &self.default_account_id
    }
}

// pub struct Viewer {
//     pub viewer: account::viewer::Response,
//     pub default_account_id: String,
//     // hash_key: String,
// }

// impl Viewer {
//     async fn new(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>) -> anyhow::Result<Self> {
//         let hash_key = format!("#Viewer");

//         let opt_viewer: Option<account::viewer::Response> = cache_manager.read_one(&hash_key)?;

//         let viewer = if let Some(viewer) = opt_viewer {
//             viewer
//         }
//         else {
//             let query = account::viewer::Query::new();
//             let viewer = request_manager.call(&query).await?;

//             cache_manager.write_one(&hash_key, &viewer)?;

//             viewer
//         };

//         let default_account_id = viewer.viewer_.accounts_.get(0).unwrap().number_.clone();

//         Ok(Viewer {
//             default_account_id,
//             viewer,
//             // hash_key,
//         })
//     }
// }