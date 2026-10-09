use dioxus::prelude::*;
use std::sync::Arc;
use std::time::Duration;

use crate::CacheManager;
use crate::data_set::{DataSetAttributes, SingleRecordDataSet};
use sparko_graphql::GraphQLQueryBuilder;

use super::graphql::meter;
use super::RequestManager;


/*
        #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
        pub struct PropertyType {
            #[serde(rename = "id")]
            pub id_: String, // T1
            #[serde(rename = "address")]
            pub address_: String, // T1
            #[serde(rename = "postcode")]
            pub postcode_: String, // T1
            #[serde(rename = "occupancyPeriods")]
            pub occupancy_periods_: Vec<OccupancyPeriodType>, // T1
            #[serde(rename = "coordinates")]
            pub coordinates_: CoordinatesType, // T1
            #[serde(rename = "electricityMeterPoints")]
            pub electricity_meter_points_: Vec<ElectricityMeterPointType>, // T1
            #[serde(rename = "gasMeterPoints")]
            pub gas_meter_points_: Vec<GasMeterPointType>, // T1
            #[serde(rename = "smartDeviceNetworks")]
            pub smart_device_networks_: Vec<SmartMeterDeviceNetworkType>, // T1
        }
*/

pub type Property = meter::account_properties_meters::PropertyType;

impl Property {
    pub fn gui_summary_header() -> Element{
        rsx!{
            tr { class: "header",
                th { "ID" }
                th { "Address" }
                th { "Post Code" }
                th { "Occupancy Periods" }
                th { "Coordinates" }
            }
        }
    }

    pub fn gui_summary_line(&self) -> Element {
        // let row_class = if attributes.cached { "cached"} else {"fetched"};
    let id = self.id_.clone();
        
        rsx!{
            tr {
                td {
                    div {
                        class: "link",
                        // onclick: |_| {path_signal.set(vec!(String::from("bills"), abstract_bill.id_.clone()))},
                        onclick: move |_| {
                            // println!("Id={}", id);
                            // let id = abstract_bill.id_.clone();
                            // nav_callback(id);
                            let mut path_signal = use_context::<Signal<Vec<String>>>();
                            let new_path = vec![String::from("bills"), id.clone()];
                            path_signal.set(new_path);
                        },
                        "{self.id_}"
                    }
                }
                td { "{self.address_}" }
                td { "{self.postcode_}" }
                td { "{self.occupancy_periods_.len()}" }
                td { "{self.coordinates_.latitude_}, {self.coordinates_.longitude_}" }
            }
        }
    }
}

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
        let data_set = SingleRecordDataSet::new(
            &hash_key,
            Duration::from_hours(20 * 24), // 30 days
            query_provider,
            config,
            request_manager
        ).await?;

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