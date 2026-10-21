use std::time::Duration;

use crate::CacheConfig;
use crate::data_set::{SingleRecordDataSet};

use super::graphql::account;
use sparko_graphql::GraphQLCaller;

pub struct ViewerDataSet {
    pub data_set: SingleRecordDataSet<account::viewer::Response>,
    pub default_account_id: String,
}

impl ViewerDataSet {
    pub async fn new(config: &CacheConfig, request_manager: &impl GraphQLCaller)  -> anyhow::Result<Self> {
        let hash_key = format!("#Viewer");
        let data_set = SingleRecordDataSet::new(
            &hash_key,
            Duration::from_hours(20 * 24), // 30 days
            || Ok(account::viewer::Query::new()),
            config,
            request_manager
        ).await?;

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