use std::sync::Arc;

use crate::CacheManager;
use crate::data_set::{SingleRecordDataSet};
use sparko_graphql::GraphQLQueryBuilder;

use super::graphql::meter;
use super::RequestManager;

pub struct PropertyDataSet {
    pub data_set: SingleRecordDataSet<meter::account_properties_meters::Response>,
     pub meter_node_ids: Vec<String>,
}

impl PropertyDataSet {
    pub async fn new(account_number: &String, config: &Arc<CacheManager>, request_manager: &Arc<RequestManager>)  -> anyhow::Result<Self> {
        let hash_key = format!("{}#Properties", account_number);
        let query_provider = || {
            meter::account_properties_meters::Query::builder()
                .with_account_number(account_number.clone())
                .build()
        };
        let data_set = SingleRecordDataSet::new(&hash_key, query_provider, config, request_manager).await?;

        let mut meter_node_ids: Vec<String> = Vec::new();

        for property in &data_set.data.account_.properties_ {
            for point in &property.electricity_meter_points_ {
                for meter in &point.meters_ {
                    meter_node_ids.push(meter.node_id_.clone());
                }
            }
            for point in &property.gas_meter_points_ {
                for meter in &point.meters_ {
                    meter_node_ids.push(meter.node_id_.clone());
                }
            }
        }

        Ok(Self {
            data_set,
            meter_node_ids,
        })
    }
}