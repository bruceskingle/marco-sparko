use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::LdExternalLink};
use indexmap::IndexMap;
use sparko_graphql::types::Date;
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

fn gui_opt_date(date: &Option<Date>) -> String {
    if let Some(date) = date {
        date.to_string()
    }
    else {
        "".to_string()
    }
}

pub type Property = meter::account_properties_meters::PropertyType;

impl Property {
    pub fn gui_summary_header() -> Element{
        rsx!{
            tr { class: "header",
                th { "ID" }
                th { "Address" }
                th { "Post Code" }
                th { "Coordinates" }
                th { "Occupancy Periods" }
            }
        }
    }


    fn format_period(period: &meter::account_properties_meters::OccupancyPeriodType) -> String {
        format!("{}-{}",
            period.effective_from_,
            if let Some(effective_to) = &period.effective_to_ {
                effective_to.to_string()
            } else {
                "".to_string()
            }
        )
    }

    pub fn gui_summary_line(&self) -> Element {
        // let row_class = if attributes.cached { "cached"} else {"fetched"};
        let id = self.id_.clone();
        let mut it = self.occupancy_periods_.iter();
        let period = if let Some(period_data) = it.next() {
            Self::format_period(period_data)
        } else {
            String::from("No occupancy periods")
        };

        let mut additional_periods = Vec::new();
        for period_data in it {
            let period_str = Self::format_period(period_data);
            additional_periods.push(rsx!{
                tr {
                    td { colspan: 4, "" }
                    td { "{period_str}" }
                }
            });
        }

        let coordinates = format!("{:.3} {:.3}", self.coordinates_.latitude_, self.coordinates_.longitude_);
        let map_url = format!("https://www.google.com/maps/search/?api=1&query={},{}", self.coordinates_.latitude_, self.coordinates_.longitude_);
        
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
                            let new_path = vec![String::from("properties"), id.clone()];
                            path_signal.set(new_path);
                        },
                        "{self.id_}"
                    }
                }
                td { "{self.address_}" }
                td { "{self.postcode_}" }
                td {
                    a { href: "{map_url}", target: "_blank",
                        "{coordinates}"
                        Icon {
                            class: "inline-icon",
                            icon: LdExternalLink,
                            width: 12,
                            height: 12,
                            title: "Opens in a new window",
                        }
                    }
                }

                td { "{period}" }
            }
            for item in additional_periods {
                {item}
            }
        }
    }



    pub fn gui_display(&self) -> Element {
        // let row_class = if attributes.cached { "cached"} else {"fetched"};
        let id = self.id_.clone();
        let mut it = self.occupancy_periods_.iter();
        let period = if let Some(period_data) = it.next() {
            Self::format_period(period_data)
        } else {
            String::from("No occupancy periods")
        };

        let mut additional_periods = Vec::new();
        for period_data in it {
            let period_str = Self::format_period(period_data);
            additional_periods.push(rsx!{
                tr {
                    td { "" }
                    td { "{period_str}" }
                }
            });
        }

        let mut meters = Vec::new();

        for meter_point in &self.electricity_meter_points_ {


            meters.push(rsx!{
                h2 { "Electricity Meter Point" }
                table { class: "display",
                    tr {
                        th { class: "row-header", "ID" }
                        td { "{meter_point.id_}" }
                    }
                    tr {
                        th { class: "row-header", "MPAN" }
                        td { "{meter_point.mpan_}" }
                    }
                    tr {
                        th { class: "row-header", "Status" }
                        td { "{meter_point.status_}" }
                    }
                    tr {
                        th { class: "row-header", "Supply End Date" }
                        td { "{gui_opt_date(&meter_point.supply_end_date_)}" }
                    }
                }
            });
            
            for meter in &meter_point.meters_ {
                let direction = if meter.import_meter_.is_none() { "Import" } else { "Export" };

                meters.push(rsx!{
                    h3 { "Electricity Meter" }
                    table { class: "display",
                        tr {
                            th { class: "row-header", "Node ID" }
                            td { "{meter.node_id_}" }
                        }
                        tr {
                            th { class: "row-header", "Serial Number" }
                            td { "{meter.serial_number_}" }
                        }
                        tr {
                            th { class: "row-header", "Consumption Units" }
                            td { "{meter.consumption_units_}" }
                        }
                        tr {
                            th { class: "row-header", "Has And Allows HH Readings" }
                            td { "{&meter.has_and_allows_hh_readings_}" }
                        }
                        tr {
                            th { class: "row-header", "Direction" }
                            td { "{direction}" }
                        }
                    }
                });
            }
        }

        let coordinates = format!("{:.3} {:.3}", self.coordinates_.latitude_, self.coordinates_.longitude_);
        let map_url = format!("https://www.google.com/maps/search/?api=1&query={},{}", self.coordinates_.latitude_, self.coordinates_.longitude_);
        
        rsx!{
            h1 { "Property Details" }
            table { class: "display",
                tr {
                    th { class: "row-header", "ID" }
                    td { "{self.id_}" }
                }
                tr {
                    th { class: "row-header", "Address" }
                    td { "{self.address_}" }
                }
                tr {
                    th { class: "row-header", "Post Code" }
                    td { "{self.postcode_}" }
                }
                tr {
                    th { class: "row-header", "Coordinates" }
                    td {
                        a { href: "{map_url}", target: "_blank",
                            "{coordinates}"
                            Icon {
                                class: "inline-icon",
                                icon: LdExternalLink,
                                width: 12,
                                height: 12,
                                title: "Opens in a new window",
                            }
                        }
                    }
                }
                tr {
                    th { class: "row-header", "Occupancy Periods" }
                    td { "{period}" }
                }
                for item in additional_periods {
                    {item}
                }
            }
            for item in meters {
                {item}
            }
        }
    }
}

pub struct PropertyDataSet {
    pub property_map: IndexMap<String, Arc<meter::account_properties_meters::PropertyType>>,
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
        let mut property_map = IndexMap::new();

        for property in data_set.data.account_.properties_ {
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
            property_map.insert(property.id_.clone(), Arc::new(property));
        }

        Ok(Self {
            property_map,
            meter_node_ids,
        })
    }
}