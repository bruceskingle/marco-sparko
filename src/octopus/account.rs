use std::sync::Arc;

use sparko_graphql::AuthenticatedRequestManager;

use crate::CacheManager;

use super::graphql::account;
use super::RequestManager;
use super::{token::OctopusTokenManager, Error};

pub struct AccountManager {
    pub cache_manager: Arc<CacheManager>,
    pub request_manager: Arc<RequestManager>,
    pub viewer: Viewer,
}

impl AccountManager {
    pub async fn new(cache_manager: &Arc<CacheManager>, request_manager: &Arc<RequestManager>)  -> Result<Self, Error> {
        let viewer = Viewer::new(cache_manager, request_manager).await?;

        Ok(Self {
            cache_manager: cache_manager.clone(),
            request_manager: request_manager.clone(),
            viewer,
        })
    }

    pub fn get_default_account_id(&self) -> &String {
        &self.viewer.default_account_id
    }
}

pub struct Viewer {
    pub viewer: account::viewer::Response,
    pub default_account_id: String,
    hash_key: String,
}

impl Viewer {
    async fn new(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>) -> Result<Self, Error> {
        let hash_key = format!("#Viewer");

        let opt_viewer: Option<account::viewer::Response> = cache_manager.read_one(&hash_key)?;

        let viewer = if let Some(viewer) = opt_viewer {
            viewer
        }
        else {
            let query = account::viewer::Query::new();
            let viewer = request_manager.call(&query).await?;

            cache_manager.write_one(&hash_key, &viewer);

            viewer
        };

        let default_account_id = viewer.viewer_.accounts_.get(0).unwrap().number_.clone();

        Ok(Viewer {
            default_account_id,
            viewer,
            hash_key,
        })
    }
}