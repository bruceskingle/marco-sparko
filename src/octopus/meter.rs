use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use dioxus::prelude::*;
use indexmap::IndexMap;
use sparko_graphql::types::{Date, DateTime};
use sparko_graphql::AuthenticatedRequestManager;
use sparko_graphql::GraphQLQueryBuilder;
use time_tz::Tz;

use crate::data_set::{DataSetAttributes, Item, MonthlyDataSet, MonthlyFixedSlotDataSet, MultiQueryDataSet, SlotSource};
use crate::octopus::property::PropertyDataSet;
use crate::{CacheConfig, NULL};

use super::graphql::meter;
use super::RequestManager;
use super::{token::OctopusTokenManager};

pub enum MeterType {
    Gas,
    Electricity
}

impl Display for MeterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeterType::Gas => write!(f, "Gas"),
            MeterType::Electricity => write!(f, "Electricity"),
        }
    }
}

pub struct MeterManager {
    // pub account_number: String,
    pub cache_manager: Arc<CacheConfig>,
    pub request_manager: Arc<RequestManager>,
    // pub 
    // pub properties: HashMap<String, Arc<PropertyList>>,
    // pub agreements: IndexMap<String,MeterAgreementList>,
}

//   Rita the
impl MeterManager {
    pub fn new(cache_manager: &Arc<CacheConfig>, request_manager: &Arc<RequestManager>)  -> Self {
        Self {
            cache_manager: cache_manager.clone(),
            request_manager: request_manager.clone(),
        }
    }

    // pub async fn demand_handler(&self, _args: std::str::SplitWhitespace<'_>, account_number: &String) ->  anyhow::Result<()> {
    //     let properties = self.get_properties(account_number).await?;
    //     for property in &properties.data_set.data.account_.properties_ {
    //         for network in &property.smart_device_networks_ {
    //             for device in &network.smart_devices_ {
    //                 if let super::graphql::DeviceType::Esme =  device.type_ {
    //                     let mut cnt=5;
    //                     let ten_seconds = Duration::new(10, 0);


    //                     let now = DateTime::now_utc();
    //                     let now_mod_ten = now.replace_second(now.second()%10).unwrap();
    //                     let mut end_timestamp = now_mod_ten.unix_timestamp() + 10;

    //                     while cnt>0 {
    //                         cnt -= 1;

    //                         let start = DateTime::from_unix_timestamp(end_timestamp - 60)?;
    //                         let end = DateTime::from_unix_timestamp(end_timestamp)?;

    //                         let query = meter::get_current_demand::Query::builder()
    //                             .with_meter_device_id(device.device_id_.clone())
    //                             .with_start(start)
    //                             .with_end(end)
    //                             .with_grouping(crate::octopus::graphql::TelemetryGrouping::TenSeconds)
    //                             .build()?;
    //                         let demand = &self.request_manager.call(&query).await?;
                
    //                         if demand.smart_meter_telemetry_.is_empty() {
    //                             println!("NO RESULT");
    //                         }
    //                         else {
    //                             let result = demand.smart_meter_telemetry_.get(demand.smart_meter_telemetry_.len() - 1).unwrap();
    //                             println!("{} at {}", result.demand_, result.read_at_);
    //                         }
    //                         end_timestamp += 10;
    //                         sleep(ten_seconds).await;
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    pub async fn get_properties(&self, account_number: &String) -> anyhow::Result<PropertyDataSet>{
        // if let std::collections::hash_map::Entry::Vacant(entry) = self.properties.entry(account_number.clone()) {
        //     entry.insert(Arc::new(PropertyList::new(&self.cache_manager, &self.request_manager, account_number.clone()).await?));
        // }
        
        // Ok(self.properties.get(account_number).unwrap().clone())

        PropertyDataSet::new(account_number, &self.cache_manager, &self.request_manager).await
    }

    pub async fn get_line_items(&self, account_number: &String, meter_type: &MeterType, is_export: bool, start_date: &Date, end_date: &Date, billing_timezone: &time_tz::Tz) -> anyhow::Result<IndexMap<String, (Tariff, Vec<meter::electricity_agreement_line_items::LineItemType>)>>{
        let properties = self.get_properties(account_number).await?;
        // if let std::collections::hash_map::Entry::Vacant(entry) = self.properties.entry(account_number.clone()) {
        //     entry.insert(PropertyList::new(&self.cache_manager, &self.request_manager, account_number.clone()).await?);
        // }
        
        // let properties = self.properties.get(account_number).unwrap();
        let meter_agreements = //MeterAgreementList::new(&self.cache_manager, &self.request_manager, account_number.clone(), &properties.meter_node_ids).await?;
            MeterAgreementDataSet::new(&self.cache_manager, &self.request_manager, &account_number, &properties.meter_node_ids).await?;

        let start_date_time = start_date.at_midnight(billing_timezone);
        let end_date_time = end_date.at_next_midnight(billing_timezone);

        //println!("get_line_items {:?} - {:?}", start_date_time, end_date_time);

        let in_scope_agreements = meter_agreements.get_in_scope(meter_type, is_export, &start_date_time, &end_date_time);

        async fn get_line_items2(
            cache_manager: &CacheConfig, request_manager: &RequestManager,
            account_number: &String, meter_type: &MeterType, agreement_id: &String,
            start_date_time: &DateTime, end_date_time: &DateTime, billing_timezone: &time_tz::Tz) -> anyhow::Result<Vec<meter::electricity_agreement_line_items::LineItemType>> {
                let mut in_scope_items = Vec::new();
                let mut bucket_date = start_date_time.to_date();
                loop {
                    println!("Get bucket {:?}", bucket_date);
                    let line_items = AgreementLineItems::new(cache_manager, request_manager, account_number.clone(), meter_type, agreement_id.clone(), &bucket_date, billing_timezone).await?;

                    for (_cursor, (_, item)) in line_items.line_items {
                        println!("Candidate line item {:?}-{:?}", item.start_at_, item.end_at_);
                        if &item.start_at_ >= end_date_time {
                            // thats it
                            println!("Line item has date {:?} so we are done", item.start_at_);
                            return Ok(in_scope_items)
                        }
                        if &item.start_at_ >= start_date_time {
                            println!("Save {:?} - {:?}", item.start_at_, item.end_at_);
                            in_scope_items.push(item);
                        }
                    }

                    bucket_date = line_items.end_date;

                    println!("New bucket date {:?}", bucket_date);

                    if *bucket_date > end_date_time.date() {
                        println!("{:?} > date {:?} so we are done", bucket_date, end_date_time);
                        return Ok(in_scope_items)
                    }
                }
        }
        
        let mut item_map = IndexMap::new();

        for (agreement_id, tariff) in in_scope_agreements {
            // let agreement_id = agreement_id.to_string(); // Ugh!

            // tariff.print();
            let in_scope_items = get_line_items2(&self.cache_manager, &self.request_manager, account_number, meter_type, &agreement_id, &start_date_time, &end_date_time, billing_timezone).await?;

            if in_scope_items.is_empty() {
                //println!("Got no in scope items");
            }
            else {
                //println!("Got {} in scope items", in_scope_items.len());
                //println!("First {}", serde_json::to_string_pretty(in_scope_items.get(0).unwrap())?);
                //println!("Last {}", serde_json::to_string_pretty(in_scope_items.get(in_scope_items.len() - 1).unwrap())?);


                item_map.insert(agreement_id, (tariff, in_scope_items));
            }
        }

        Ok(item_map)
    }



    pub async fn get_consumption(
        &self,
        account_number: &String,
        meter_node_id: &String,
        start: &DateTime,
        end: &Option<DateTime>,
        tz: &'static Tz)
        -> anyhow::Result<Vec<MonthlyFixedSlotDataSet<ConsumptionSource>>>
    {

        let source = ConsumptionSource{
            meter_node_id: meter_node_id.clone(),
        };
        let hash_key = format!("{}#{}#ConsumptionRecords", account_number, meter_node_id);
        
        MonthlyFixedSlotDataSet::fetch(
            source,
            hash_key,
            tz,
            &self.cache_manager,
            &self.request_manager,
            start,
            end
        ).await

        // // if let std::collections::hash_map::Entry::Vacant(entry) = self.properties.entry(account_number.clone()) {
        // //     entry.insert(PropertyList::new(&self.cache_manager, &self.request_manager, account_number.clone()).await?);
        // // }
        
        
        // // async fn get_line_items2(
        // //     cache_manager: &CacheManager, request_manager: &RequestManager,
        // //     account_number: &String, meter_node_id: &String, start_date_time: &DateTime, end_date_time: &DateTime, billing_timezone: &time_tz::Tz) -> anyhow::Result<Vec<meter::meter_consumption::ConsumptionType>> {
        //         let mut in_scope_items = Vec::new();
        //         let mut bucket_date = start.to_date();
        //         let end = if let Some(to) = end {to} else {&DateTime::now_utc()};
        //         let end_date = end.date();

        //         loop {
        //             println!("Get bucket {:?}", bucket_date);
        //             let line_items =ConsumptionDataSet::new(&self.cache_manager, &self.request_manager, account_number.clone(), meter_node_id.clone(), &bucket_date, billing_timezone).await?;
        //             // let line_items = ConsumptionList::new(self.cache_manager, self.request_manager, account_number.clone(), meter_node_id.clone(), &bucket_date, billing_timezone).await?;

        //             for (key, item) in line_items.consumption{
        //                 //println!("Candidate line item {:?}-{:?}", item.start_at_, item.end_at_);
        //                 if &key >= end {
        //                     // thats it
        //                     //println!("Line item has date {:?} so we are done", item.start_at_);
        //                     return Ok(in_scope_items)
        //                 }
        //                 if &key >= start {
        //                     //println!("Save {:?} - {:?}", item.start_at_, item.end_at_);
        //                     in_scope_items.push(item);
        //                 }
        //             }

        //             bucket_date = line_items.end_date;

        //             println!("New bucket date {:?}", bucket_date);

        //             if *bucket_date > end_date {
        //                 //println!("{:?} > date {:?} so we are done", bucket_date, end_date_time);
        //                 return Ok(in_scope_items)
        //             }
        //         }
        // // }
        
        // // get_line_items2(&self.cache_manager, &self.request_manager, account_number, meter_node_id, &start_date_time, &end_date_time, billing_timezone).await
    }

    pub fn display_consumption(data: &Vec<MonthlyFixedSlotDataSet<ConsumptionSource>>, start: &DateTime, end: &Option<DateTime>) -> Element {
        // let items = Vec::new();

        // for (attr, consumption) in data {
        //     items.push(consumption.gui_summary_line(attr)?);
        // }
        let date_format = time::format_description::parse("[year]-[month]-[day]").unwrap();
        let time_format = time::format_description::parse("[hour]:[minute]").unwrap();

        let mut items = Vec::new();
        let mut last_date = Date::from_calendar_date(0, time::Month::January, 1)?;
        for data_set in data {
            for (key , item) in &data_set.data {
                let end_ok = if let Some(end) = end {
                    key < end
                }
                else {
                    true
                };
                
                if key >= start && end_ok {
                    
                    let row_class = if item.attributes.cached { "cached"} else {"fetched"};
                    let item_start_date = item.value.start_at_.to_date();
                    let (start_date, start_time) = if last_date == item_start_date {
                        ("".to_string(), item.value.start_at_.format(&time_format).unwrap())
                    } else {
                        (item.value.start_at_.format(&date_format).unwrap(), item.value.start_at_.format(&time_format).unwrap())
                    };
                    
                    let end_time = item.value.end_at_.format(&time_format).unwrap();
                    let amount = format!("{:.3}", item.value.value_);

                    items.push(rsx!{
                        tr { class: row_class,
                            td { "{start_date}" }
                            td { "{start_time}" }
                            td { "{end_time}" }
                            td { "{amount}" }
                        }
                    }
                    );
                }
                last_date = key.to_date();
            }
        };

        rsx! {
            table {
                tr { class: "header",
                    th { colspan: 2, "From" }
                    th { "To" }
                    th { "Value" }
                }
                for item in items {
                    {item}
                }
            }
        }
    }
}


#[derive(Debug)]
pub enum Tariff {
    Electricity(meter::meter_agreements::ElectricityTariffType),
    Gas(meter::meter_agreements::GasTariffType)
}

fn format_rate(rate: f64) -> String {
    format!("{:7.4}", rate)
}


fn gui_rate(rate: f64) -> Element {
    rsx!{
        td { class: "numeric", {format_rate(rate)} }
    }
}

fn gui_opt_rate(rate: Option<f64>) -> Element {
    if let Some(rate) = rate {
        gui_rate(rate)
    }
    else {
        rsx!{
            td { class: "numeric invalid", {NULL} }
        }
    }
}


fn format_opt_str<T: ToString>(str: &Option<T>) -> String {
    if let Some(str) = str {
        str.to_string()
    }
    else {
        "UNKNOWN".to_string()
    }
}

fn gui_opt_str<T: ToString>(str: &Option<T>) -> Element {
    if let Some(str) = str {
        rsx!{
            td { {str.to_string()} }
        }
    }
    else {
        rsx!{
            td { class: "invalid", {NULL} }
        }
    }
}

impl Tariff {
    pub fn standing_charge(&self) -> Option<f64> {
        match self {
            Tariff::Electricity(electricity_tariff_type) => {
                match electricity_tariff_type {
                    meter::meter_agreements::ElectricityTariffType::StandardTariff(tariff) => { tariff.pre_vat_standing_charge_},
                    meter::meter_agreements::ElectricityTariffType::DayNightTariff(tariff) => { tariff.pre_vat_standing_charge_},
                    meter::meter_agreements::ElectricityTariffType::ThreeRateTariff(tariff) => { tariff.pre_vat_standing_charge_},
                    meter::meter_agreements::ElectricityTariffType::HalfHourlyTariff(tariff) => { tariff.pre_vat_standing_charge_},
                    meter::meter_agreements::ElectricityTariffType::PrepayTariff(tariff) => { tariff.pre_vat_standing_charge_},
                    meter::meter_agreements::ElectricityTariffType::FourRateEvTariff(tariff) => {tariff.pre_vat_standing_charge_},
                }
            },
            Tariff::Gas(tariff) => { tariff.pre_vat_standing_charge_},
        }
    }

    pub fn gui_display(&self) -> Element {
        match self {
            Tariff::Electricity(electricity_tariff_type) => {
                match electricity_tariff_type {
                    meter::meter_agreements::ElectricityTariffType::StandardTariff(tariff) => {
                        rsx!{
                            div {
                                h3 { "Electricity Tariff: {tariff.display_name_}" }
                                table { class: "display",
                                    tr {
                                        th { class: "row-header", "Full Name" }
                                        td { {tariff.full_name_.as_str()} }
                                    }
                                    tr {
                                        th { class: "row-header", "Code" }
                                        td { {tariff.tariff_code_.as_str()} }
                                    }
                                    tr {
                                        th { class: "row-header", "Pre-VAT Standing Charge" }
                                        {gui_opt_rate(tariff.pre_vat_standing_charge_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Standing Charge" }
                                        {gui_opt_rate(tariff.standing_charge_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Pre-VAT Unit Rate" }
                                        {gui_opt_rate(tariff.pre_vat_unit_rate_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Unit Rate" }
                                        {gui_opt_rate(tariff.unit_rate_)}
                                    }
                                }
                            }
                        }
                    },
                    meter::meter_agreements::ElectricityTariffType::DayNightTariff(_tariff) => todo!(),
                    meter::meter_agreements::ElectricityTariffType::ThreeRateTariff(_tariff) => todo!(),
                    meter::meter_agreements::ElectricityTariffType::HalfHourlyTariff(tariff) => {
                        rsx!{
                            div {
                                h3 { "Electricity Tariff: {format_opt_str(&tariff.display_name_)}" }
                                table { class: "display",
                                    tr {
                                        th { class: "row-header", "Full Name" }
                                        {gui_opt_str(&tariff.full_name_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Display Name" }
                                        {gui_opt_str(&tariff.display_name_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Tariff Code" }
                                        {gui_opt_str(&tariff.tariff_code_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Product Code" }
                                        {gui_opt_str(&tariff.product_code_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Pre-VAT Standing Charge" }
                                        {gui_opt_rate(tariff.pre_vat_standing_charge_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Standing Charge" }
                                        {gui_opt_rate(tariff.standing_charge_)}
                                    }
                                
                                }
                            
                            // h4 { "Unit Rates" }
                            // table { class: "display",
                            //     tr {
                            //         th { "Valid From" }
                            //         th { "Valid To" }
                            //         th { "Pre-VAT Unit Rate" }
                            //         th { "Unit Rate" }
                            //     }
                            //     for rate in &tariff.unit_rates_ {
                            //         tr {
                            //             {gui_opt_str(&rate.valid_from_)}
                            //             {gui_opt_str(&rate.valid_to_)}
                            //             {gui_opt_rate(rate.pre_vat_value_)}
                            //             {gui_opt_rate(rate.value_)}
                            //         }
                            //     }
                            // }
                            }
                        }
                    },
                    meter::meter_agreements::ElectricityTariffType::PrepayTariff(_tariff) => todo!(),
                    meter::meter_agreements::ElectricityTariffType::FourRateEvTariff(_tariff) => todo!(),
                }
            },
            Tariff::Gas(gas_tariff_type) => {
                rsx!{
                    div {
                        h3 { "Gas Tariff: {gas_tariff_type.full_name_}" }
                        table { class: "display",
                            tr {
                                th { class: "row-header", "Code" }
                                td { {gas_tariff_type.tariff_code_.as_str()} }
                            }
                            tr {
                                th { class: "row-header", "Standing Charge" }
                                {gui_opt_rate(gas_tariff_type.standing_charge_)}
                            }
                            tr {
                                th { class: "row-header", "Pre-VAT Unit Rate" }
                                {gui_opt_rate(gas_tariff_type.pre_vat_unit_rate_)}
                            }
                            tr {
                                th { class: "row-header", "Unit Rate" }
                                {gui_opt_rate(gas_tariff_type.unit_rate_)}
                            }
                        }
                    }
                }
            },
        }
    }
}

pub struct MeterAgreementDataSet {
    pub import_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
    pub export_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
    pub gas_map: HashMap<String, Vec<meter::meter_agreements::GasAgreementType>>,
}

impl MeterAgreementDataSet {
    pub async fn new(config: &Arc<CacheConfig>, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: &str, meter_node_ids: &Vec<String>) -> anyhow::Result<Self> {
        let hash_key = format!("{}#MeterAgreements", account_number);
        let the_beginning: DateTime = DateTime::from_calendar_date(2000, time::Month::January, 1)?;
        let mut queries = Vec::new();

        for meter_node_id in meter_node_ids {
            let query = meter::meter_agreements::Query::builder()
                .with_meter_node_id(meter_node_id.clone())
                .with_valid_after(the_beginning.clone())
                .build();

            queries.push(query);
        }
        let data = MultiQueryDataSet::new(
            &hash_key,
            Duration::from_hours(24),
            queries,
            config,
            request_manager).await?;
        

        let mut export_electricity_map = HashMap::new();
        let mut import_electricity_map = HashMap::new();
        let mut gas_map = HashMap::new();

        for response in data.data {
            match response.node_ {
                meter::meter_agreements::Node::ElectricityMeterType(electricity_meter_type) => {
                    if electricity_meter_type.import_meter_.is_some() {
                        export_electricity_map.insert(electricity_meter_type.node_id_, electricity_meter_type.meter_point_.agreements_);
                    }
                    else {
                        import_electricity_map.insert(electricity_meter_type.node_id_, electricity_meter_type.meter_point_.agreements_);
                    }
                },
                meter::meter_agreements::Node::GasMeterType(gas_meter_type) => {
                    gas_map.insert(gas_meter_type.node_id_, gas_meter_type.meter_point_.agreements_);
                },
                _ => unreachable!()
            }
        }
        
        Ok(Self {
            export_electricity_map,
            import_electricity_map,
            gas_map,
        })
    }

    fn get_in_scope(self, meter_type: &MeterType, is_export: bool, start_date: &DateTime, end_date: &DateTime) -> Vec<(String, Tariff)> {
        let mut in_scope_agreements = Vec::new();

        match meter_type {
            MeterType::Gas => {
                for (_meter_node_id, agreement_vec) in self.gas_map {
                    for agreement in agreement_vec {
                        if &agreement.valid_from_ <= end_date {
                            if let Some(valid_to) = &agreement.valid_to_ {
                                if valid_to >= start_date {
                                    in_scope_agreements.push((agreement.id_.to_string(), Tariff::Gas(agreement.tariff_)));
                                }
                            }
                            else {
                                in_scope_agreements.push((agreement.id_.to_string(), Tariff::Gas(agreement.tariff_)));
                            }
                        }
                    }
                }
            },
            MeterType::Electricity => {
                for (_meter_node_id, agreement_vec) in if is_export {self.export_electricity_map} else {self.import_electricity_map} {
                    for agreement in agreement_vec {
                        if &agreement.valid_from_ <= end_date {
                            if let Some(valid_to) = &agreement.valid_to_ {
                                if valid_to >= start_date {
                                    in_scope_agreements.push((agreement.id_.to_string(), Tariff::Electricity(agreement.tariff_)));
                                }
                            }
                            else {
                                in_scope_agreements.push((agreement.id_.to_string(), Tariff::Electricity(agreement.tariff_)));
                            }
                        }
                    }
                }
            },
        }

        in_scope_agreements
    }
}

// pub struct MeterAgreementList {
//     pub _account_number: String,
//     pub import_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
//     pub export_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
//     pub gas_map: HashMap<String, Vec<meter::meter_agreements::GasAgreementType>>,
//     _hash_key: String,
// }

// impl MeterAgreementList {
//     async fn new(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: String, meter_node_ids: &Vec<String>) -> anyhow::Result<Self> {
//         let hash_key = format!("{}#MeterAgreements", account_number);
//             let the_beginning: DateTime = DateTime::from_calendar_date(2000, time::Month::January, 1)?;
//             let mut agreements = Vec::new();
    
//             cache_manager.read_vec(&hash_key, &mut agreements)?;
    
//             let cached_cnt = agreements.len();
    
//             if agreements.is_empty() {
//                 for meter_node_id in meter_node_ids {
//                     let query = meter::meter_agreements::Query::builder()
//                             .with_meter_node_id(meter_node_id.clone())
//                             .with_valid_after(the_beginning.clone())
//                             .build()?;
//                     let response = request_manager.call(&query).await?;

//                     agreements.push((meter_node_id.clone(), response));
//                 }
//                 cache_manager.write_vec(&hash_key, &agreements, cached_cnt)?;
//             }

//             let mut export_electricity_map = HashMap::new();
//             let mut import_electricity_map = HashMap::new();
//             let mut gas_map = HashMap::new();

//             for (meter_node_id, response) in agreements {
//                 match response.node_ {
//                     meter::meter_agreements::Node::ElectricityMeterType(electricity_meter_type) => {
//                         if electricity_meter_type.import_meter_.is_some() {
//                             export_electricity_map.insert(meter_node_id, electricity_meter_type.meter_point_.agreements_);
//                         }
//                         else {
//                             import_electricity_map.insert(meter_node_id, electricity_meter_type.meter_point_.agreements_);
//                         }
//                     },
//                     meter::meter_agreements::Node::GasMeterType(gas_meter_type) => {
//                         gas_map.insert(meter_node_id, gas_meter_type.meter_point_.agreements_);
//                     },
//                     _ => unreachable!()
//                 }
//             }
            
//             Ok(MeterAgreementList {
//                 _account_number: account_number,
//                 export_electricity_map,
//                 import_electricity_map,
//                 gas_map,
//                 _hash_key: hash_key,
//             })
//         }

//     fn get_in_scope(self, meter_type: &MeterType, is_export: bool, start_date: &DateTime, end_date: &DateTime) -> Vec<(String, Tariff)> {
//         let mut in_scope_agreements = Vec::new();

//         match meter_type {
//             MeterType::Gas => {
//                 for (_meter_node_id, agreement_vec) in self.gas_map {
//                     for agreement in agreement_vec {
//                         if &agreement.valid_from_ <= end_date {
//                             if let Some(valid_to) = &agreement.valid_to_ {
//                                 if valid_to >= start_date {
//                                     in_scope_agreements.push((agreement.id_.to_string(), Tariff::Gas(agreement.tariff_)));
//                                 }
//                             }
//                             else {
//                                 in_scope_agreements.push((agreement.id_.to_string(), Tariff::Gas(agreement.tariff_)));
//                             }
//                         }
//                     }
//                 }
//             },
//             MeterType::Electricity => {
//                 for (_meter_node_id, agreement_vec) in if is_export {self.export_electricity_map} else {self.import_electricity_map} {
//                     for agreement in agreement_vec {
//                         if &agreement.valid_from_ <= end_date {
//                             if let Some(valid_to) = &agreement.valid_to_ {
//                                 if valid_to >= start_date {
//                                     in_scope_agreements.push((agreement.id_.to_string(), Tariff::Electricity(agreement.tariff_)));
//                                 }
//                             }
//                             else {
//                                 in_scope_agreements.push((agreement.id_.to_string(), Tariff::Electricity(agreement.tariff_)));
//                             }
//                         }
//                     }
//                 }
//             },
//         }

//         in_scope_agreements
//     }
    
// }

// impl meter::meter_consumption::Node {
//     pub fn as_consumption_page(&self) -> &ForwardPageOf<meter::meter_consumption::ConsumptionType> {
//         match self {
//             meter::meter_consumption::Node::ElectricityMeterType(electricity_meter) => {
//                 &electricity_meter.consumption_
//             },
//             meter::meter_consumption::Node::GasMeterType(gas_meter) => {
//                 &gas_meter.consumption_
//             },
//             _ => {
//                 unreachable!()
//                 //return Some(Err(anyhow!("Unexpected response type")))
//             },
//         }
//     }
// }

impl meter::electricity_agreement_line_items::AgreementInterface {

    pub fn as_electricity_agreement(self) -> meter::electricity_agreement_line_items::ElectricityAgreementType {    
        match self {
            meter::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
                electricity_agreement_type
            },
            meter::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_abstract_agreement_interface) => unreachable!(),
        }
    }
    pub fn as_electricity_agreement_ref(&self) -> &meter::electricity_agreement_line_items::ElectricityAgreementType {    
        match self {
            meter::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
                electricity_agreement_type
            },
            meter::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_abstract_agreement_interface) => unreachable!(),
        }
    }

    // pub fn get_line_items(&self) -> &Vec<EdgeOf<meter::electricity_agreement_line_items::LineItemType>> {
    //     match self {
    //         meter::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
    //             &electricity_agreement_type.line_items_.edges
    //             //.into_iter().collect()
    //         },
    //         meter::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_abstract_agreement_interface) => unreachable!(),
    //     }
    // }


    // pub fn get_page_info(&self) -> &PageInfo {
    //     match self {
    //         meter::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
    //             &electricity_agreement_type.line_items_.page_info
    //             //.into_iter().collect()
    //         },
    //         meter::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_abstract_agreement_interface) => unreachable!(),
    //     }
    // }
}

impl meter::gas_agreement_line_items::AgreementInterface {
    pub fn as_gas_agreement(self) -> meter::gas_agreement_line_items::GasAgreementType {
        match self {
            meter::gas_agreement_line_items::AgreementInterface::ElectricityAgreementType(_electricity_agreement_type) => unreachable!(),
            meter::gas_agreement_line_items::AgreementInterface::GasAgreementType(abstract_agreement_interface) => {
                abstract_agreement_interface
            },
        }
    }
    
    pub fn as_gas_agreement_ref(&self) -> &meter::gas_agreement_line_items::GasAgreementType {
        match self {
            meter::gas_agreement_line_items::AgreementInterface::ElectricityAgreementType(_electricity_agreement_type) => unreachable!(),
            meter::gas_agreement_line_items::AgreementInterface::GasAgreementType(abstract_agreement_interface) => {
                abstract_agreement_interface
            },
        }
    }
    
    // pub fn get_line_items(&self) -> &Vec<EdgeOf<meter::gas_agreement_line_items::LineItemType>> {
    //     match self {
    //         meter::gas_agreement_line_items::AgreementInterface::ElectricityAgreementType(_electricity_agreement_type) => unreachable!(),
    //         meter::gas_agreement_line_items::AgreementInterface::GasAgreementType(abstract_agreement_interface) => {
    //             &abstract_agreement_interface.line_items_.edges
    //         },
    //     }
    // }


    // pub fn get_page_info(&self) -> &PageInfo {
    //     match self {
    //         meter::gas_agreement_line_items::AgreementInterface::ElectricityAgreementType(_electricity_agreement_type) => unreachable!(),
    //         meter::gas_agreement_line_items::AgreementInterface::GasAgreementType(abstract_agreement_interface) =>{
    //             &abstract_agreement_interface.line_items_.page_info
    //         },
    //     }
    // }
}

impl Into<meter::electricity_agreement_line_items::LineItemType> for meter::gas_agreement_line_items::LineItemType {
    fn into(self) -> meter::electricity_agreement_line_items::LineItemType {
        meter::electricity_agreement_line_items::LineItemType {
            start_at_: self.start_at_,
            end_at_: self.end_at_,
            net_amount_: self.net_amount_,
            number_of_units_: self.number_of_units_,
            settlement_unit_:self.settlement_unit_,
        }
    }
}

#[allow(dead_code)]
pub struct AgreementLineItems {
    pub account_number: String,
    pub agreement_id: String,
    start_date: Date,
    end_date: Date,
    pub line_items: BTreeMap<DateTime, (crate::data_set::DataSetAttributes, meter::electricity_agreement_line_items::LineItemType)>,
}

impl AgreementLineItems {
    async fn new(
        config: &CacheConfig,
        request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
        account_number: String,
        meter_type: &MeterType,
        agreement_id: String,
        date: &Date,
        billing_timezone: &time_tz::Tz
    ) -> anyhow::Result<Self> {
        let hash_key = format!("{}#{}#{}AgreementTransactions", account_number, agreement_id, meter_type);
        
        let (
            start_date,
            end_date,
            line_items): (Date, Date, std::collections::BTreeMap<DateTime, (crate::data_set::DataSetAttributes, meter::electricity_agreement_line_items::LineItemType)>
        ) = match meter_type {
            MeterType::Gas => {
                let initial_query_provider = |bucket_start: &Date, last_record: Option<&meter::gas_agreement_line_items::LineItemType>| {
                    let start_date = if let Some(last_record) = last_record {
                        last_record.start_at_.clone()
                    }
                    else {
                        bucket_start.at_midnight(billing_timezone)
                    };
                    meter::gas_agreement_line_items::Query::builder()
                        .with_agreement_id(agreement_id.clone())
                        .with_start_at(start_date)
                        .with_timezone(String::from("Europe/London"))
                        .with_item_type(super::graphql::LineItemTypeOptions::ConsumptionCharge)
                        .with_line_item_grouping(super::graphql::LineItemGroupingOptions::None)
                        .with_first(50)
                        .build()

                };
                let continuation_query_provider = |_bucket_start: &Date, last_response: &meter::gas_agreement_line_items::Response| {
                    
                    let agreement = last_response.gas_agreement_.as_gas_agreement_ref();
                    if let Some(end_cursor) = agreement.line_items_.page_info.end_cursor.clone() {
                        let start_date = agreement.line_items_.edges.last().unwrap().node.start_at_.clone();
                        Some(meter::gas_agreement_line_items::Query::builder()
                            .with_agreement_id(agreement_id.clone())
                            .with_start_at(start_date)
                            .with_timezone(String::from("Europe/London"))
                            .with_item_type(super::graphql::LineItemTypeOptions::ConsumptionCharge)
                            .with_line_item_grouping(super::graphql::LineItemGroupingOptions::None)
                            .with_first(50)
                            .with_after(end_cursor)
                            .build())
                    }
                    else {
                        None
                    }
                };

                let response_iterator = |response: meter::gas_agreement_line_items::Response| {
                    response.gas_agreement_.as_gas_agreement().line_items_.edges.into_iter().map(|edge| edge.node)
                };

                let indexer = |item: &meter::gas_agreement_line_items::LineItemType| {
                    (item.start_at_.clone(), true)
                };

                 let data_set = MonthlyDataSet::new(&hash_key,
                    date,
                    Duration::from_hours(24),
                    initial_query_provider,
                    continuation_query_provider,
                    response_iterator,
                    indexer,
                    config,
                    request_manager
                ).await?;

                let mut data = BTreeMap::new();
                for (k, (a,v)) in data_set.data {
                    data.insert(k, (a, v.into()));
                }

                (data_set.start_date, data_set.next_date, data)
            },
            MeterType::Electricity => {
                let initial_query_provider = |bucket_start: &Date, last_record: Option<&meter::electricity_agreement_line_items::LineItemType>| {
                    let start_date = if let Some(last_record) = last_record {
                        last_record.start_at_.clone()
                    }
                    else {
                        bucket_start.at_midnight(billing_timezone)
                    };
                
                    meter::electricity_agreement_line_items::Query::builder()
                        .with_agreement_id(agreement_id.clone())
                        .with_start_at(start_date)
                        .with_timezone(String::from("Europe/London"))
                        .with_item_type(super::graphql::LineItemTypeOptions::ConsumptionCharge)
                        .with_line_item_grouping(super::graphql::LineItemGroupingOptions::None)
                        .with_first(50)
                        .build()

                };
                let continuation_query_provider = |_bucket_start: &Date, last_response: &meter::electricity_agreement_line_items::Response| {
                    // let end_cursor = last_response.electricity_agreement_.get_line_items().last().unwrap().cursor;
                
                    let agreement = last_response.electricity_agreement_.as_electricity_agreement_ref();
                    if let Some(end_cursor) = agreement.line_items_.page_info.end_cursor.clone() {
                        let start_date = agreement.line_items_.edges.last().unwrap().node.start_at_.clone();
                        Some(meter::electricity_agreement_line_items::Query::builder()
                        .with_agreement_id(agreement_id.clone())
                        .with_start_at(start_date.clone())
                        .with_timezone(String::from("Europe/London"))
                        .with_item_type(super::graphql::LineItemTypeOptions::ConsumptionCharge)
                        .with_line_item_grouping(super::graphql::LineItemGroupingOptions::None)
                        .with_first(50)
                        .with_after(end_cursor)
                        .build())
                    }
                    else {
                        None
                    }
                };


                let response_iterator = |response: meter::electricity_agreement_line_items::Response| {
                    response.electricity_agreement_.as_electricity_agreement().line_items_.edges.into_iter().map(|edge| edge.node)
                };

                let indexer = |item: &meter::electricity_agreement_line_items::LineItemType| {
                    (item.start_at_.clone(), true)
                };

                 let data_set = MonthlyDataSet::new(&hash_key,
                    date,
                    Duration::from_hours(24),
                    initial_query_provider,
                    continuation_query_provider,
                    response_iterator,
                    indexer,
                    config,
                    request_manager
                ).await?;

                (data_set.start_date, data_set.next_date, data_set.data)
            },
        };

        Ok(Self {
            account_number,
            agreement_id,
            start_date,
            end_date,
            line_items,
        })
    }
}

pub struct ConsumptionDataSet {
    #[allow(dead_code)]
    pub account_number: String,
    #[allow(dead_code)]
    pub meter_node_id: String,
    #[allow(dead_code)]
    start_date: Date,
    end_date: Date,
    pub consumption: BTreeMap<DateTime, (crate::data_set::DataSetAttributes, meter::meter_consumption::ConsumptionType)>,
}

pub struct ConsumptionSource {
    pub meter_node_id:String,
}

impl SlotSource for ConsumptionSource {
    type Value = meter::meter_consumption::ConsumptionType;

    type Response = meter::meter_consumption::Response;

    type Query = meter::meter_consumption::Query;

    fn hash_key(&self, date: &Date) -> String {
        todo!()
    }

    fn initial_query(&self, start: &DateTime) -> std::prelude::v1::Result<Self::Query, sparko_graphql::Error> {
        meter::meter_consumption::Query::builder()
            .with_meter_id(self.meter_node_id.clone())
            .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
            .with_start_at(start.clone())
            .with_timezone(String::from("Europe/London"))
            .with_first(50)
            .build()
    }

    fn continuation_query(&self, start: &DateTime, last_response: &Self::Response) -> Option<std::prelude::v1::Result<Self::Query, sparko_graphql::Error>> {
        let page =  match &last_response.node_ {
            meter::meter_consumption::Node::ElectricityMeterType(electricity_meter) => {
                &electricity_meter.consumption_
            },
            meter::meter_consumption::Node::GasMeterType(gas_meter) => {
                &gas_meter.consumption_
            },
            _ => {
                unreachable!()
            },
        };

        if let Some(end_cursor) = &page.page_info.end_cursor {

            // let start_date = page.edges.last().unwrap().node.start_at_.clone();
            Some(meter::meter_consumption::Query::builder()
                .with_meter_id(self.meter_node_id.clone())
                .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
                .with_start_at(start.clone())
                .with_timezone(String::from("Europe/London"))
                .with_first(50)
                .with_after(end_cursor.clone())
                .build())
        }
        else {
            None
        }
    }

    fn items(&self, response: Self::Response) -> impl IntoIterator<Item = Self::Value> {
        let page =  match response.node_ {
            meter::meter_consumption::Node::ElectricityMeterType(electricity_meter) => {
                electricity_meter.consumption_
            },
            meter::meter_consumption::Node::GasMeterType(gas_meter) => {
                gas_meter.consumption_
            },
            _ => {
                unreachable!()
            },
        };

        page.edges.into_iter().map(|edge| edge.node)
    }

    fn index(&self, value: &Self::Value) -> (DateTime, DateTime, bool) {
        (value.start_at_.clone(), value.end_at_.clone(), true)
    }
    
    fn refresh_after(&self) -> Duration {
        Duration::from_hours(24)
    }
}

impl ConsumptionDataSet {
    async fn new(
        config: &CacheConfig,
        request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
        account_number: String,
        meter_node_id: String,
        date: &Date,
        billing_timezone: &time_tz::Tz
    ) -> anyhow::Result<Self> {
        let hash_key = format!("{}#{}#ConsumptionRecords", account_number, meter_node_id);


        let initial_query_provider = |bucket_start: &Date, _last_record: Option<&meter::meter_consumption::ConsumptionType>| {
            // let start_date = if let Some(last_record) = last_record {
            //     last_record.start_at_.clone()
            // }
            // else {
            //     bucket_start.at_midnight(billing_timezone)
            // };

            let start_date = bucket_start.at_midnight(billing_timezone);
            meter::meter_consumption::Query::builder()
                .with_meter_id(meter_node_id.clone())
                .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
                .with_start_at(start_date)
                .with_timezone(String::from("Europe/London"))
                .with_first(50)
                .build()
        };
        let continuation_query_provider = |bucket_start: &Date, last_response: &meter::meter_consumption::Response| {
            let page =  match &last_response.node_ {
                meter::meter_consumption::Node::ElectricityMeterType(electricity_meter) => {
                    &electricity_meter.consumption_
                },
                meter::meter_consumption::Node::GasMeterType(gas_meter) => {
                    &gas_meter.consumption_
                },
                _ => {
                    unreachable!()
                },
            };

            if let Some(end_cursor) = &page.page_info.end_cursor {

                let start_date = bucket_start.at_midnight(billing_timezone);
                // let start_date = page.edges.last().unwrap().node.start_at_.clone();
                Some(meter::meter_consumption::Query::builder()
                    .with_meter_id(meter_node_id.clone())
                    .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
                    .with_start_at(start_date)
                    .with_timezone(String::from("Europe/London"))
                    .with_first(50)
                    .with_after(end_cursor.clone())
                    .build())
            }
            else {
                None
            }
        };


        let response_iterator = |response: meter::meter_consumption::Response| {
            let page =  match response.node_ {
                meter::meter_consumption::Node::ElectricityMeterType(electricity_meter) => {
                    electricity_meter.consumption_
                },
                meter::meter_consumption::Node::GasMeterType(gas_meter) => {
                    gas_meter.consumption_
                },
                _ => {
                    unreachable!()
                },
            };

           page.edges.into_iter().map(|edge| edge.node)
        };

        let indexer = |item: &meter::meter_consumption::ConsumptionType| {
            (item.start_at_.clone(), true)
        };

        let data_set = MonthlyDataSet::new(&hash_key,
            date,
            Duration::from_hours(24),
            initial_query_provider,
            continuation_query_provider,
            response_iterator,
            indexer,
            config,
            request_manager
        ).await?;

        let start_date = data_set.start_date;
        let end_date = data_set.next_date;
        let consumption = data_set.data;

        Ok(Self {
            account_number,
            meter_node_id,
            start_date,
            end_date,
            consumption,
        })
    }
}




/*
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use dioxus::prelude::*;
use indexmap::IndexMap;
use sparko_graphql::types::{Date, DateTime};
use sparko_graphql::AuthenticatedRequestManager;
use sparko_graphql::GraphQLQueryBuilder;
use tokio::sync::Mutex;

use crate::data_set::{
    DataSetAttributes, MonthlyDataSet, MonthlyFixedSlotDataSet, MultiQueryDataSet,
};
use crate::octopus::property::PropertyDataSet;
use crate::{CacheConfig, NULL};

use super::graphql::meter;
use super::RequestManager;
use super::{token::OctopusTokenManager};

pub enum MeterType {
    Gas,
    Electricity
}

impl Display for MeterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeterType::Gas => write!(f, "Gas"),
            MeterType::Electricity => write!(f, "Electricity"),
        }
    }
}

pub struct MeterManager {
    // pub account_number: String,
    pub cache_manager: Arc<CacheConfig>,
    pub request_manager: Arc<RequestManager>,
    consumption: Mutex<HashMap<String, ConsumptionDataSet>>,
    // pub properties: HashMap<String, Arc<PropertyList>>,
    // pub agreements: IndexMap<String,MeterAgreementList>,
}

//   Rita the
impl MeterManager {
    pub fn new(cache_manager: &Arc<CacheConfig>, request_manager: &Arc<RequestManager>)  -> Self {
       Self {
            cache_manager: cache_manager.clone(),
            request_manager: request_manager.clone(),
            consumption: Mutex::new(HashMap::new()),
        }
    }

    // pub async fn demand_handler(&self, _args: std::str::SplitWhitespace<'_>, account_number: &String) ->  anyhow::Result<()> {
    //     let properties = self.get_properties(account_number).await?;
    //     for property in &properties.data_set.data.account_.properties_ {
    //         for network in &property.smart_device_networks_ {
    //             for device in &network.smart_devices_ {
    //                 if let super::graphql::DeviceType::Esme =  device.type_ {
    //                     let mut cnt=5;
    //                     let ten_seconds = Duration::new(10, 0);


    //                     let now = DateTime::now_utc();
    //                     let now_mod_ten = now.replace_second(now.second()%10).unwrap();
    //                     let mut end_timestamp = now_mod_ten.unix_timestamp() + 10;

    //                     while cnt>0 {
    //                         cnt -= 1;

    //                         let start = DateTime::from_unix_timestamp(end_timestamp - 60)?;
    //                         let end = DateTime::from_unix_timestamp(end_timestamp)?;

    //                         let query = meter::get_current_demand::Query::builder()
    //                             .with_meter_device_id(device.device_id_.clone())
    //                             .with_start(start)
    //                             .with_end(end)
    //                             .with_grouping(crate::octopus::graphql::TelemetryGrouping::TenSeconds)
    //                             .build()?;
    //                         let demand = &self.request_manager.call(&query).await?;
                
    //                         if demand.smart_meter_telemetry_.is_empty() {
    //                             println!("NO RESULT");
    //                         }
    //                         else {
    //                             let result = demand.smart_meter_telemetry_.get(demand.smart_meter_telemetry_.len() - 1).unwrap();
    //                             println!("{} at {}", result.demand_, result.read_at_);
    //                         }
    //                         end_timestamp += 10;
    //                         sleep(ten_seconds).await;
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    pub async fn get_properties(&self, account_number: &String) -> anyhow::Result<PropertyDataSet>{
        // if let std::collections::hash_map::Entry::Vacant(entry) = self.properties.entry(account_number.clone()) {
        //     entry.insert(Arc::new(PropertyList::new(&self.cache_manager, &self.request_manager, account_number.clone()).await?));
        // }
        
        // Ok(self.properties.get(account_number).unwrap().clone())

        PropertyDataSet::new(account_number, &self.cache_manager, &self.request_manager).await
    }

    pub async fn get_line_items(&self, account_number: &String, meter_type: &MeterType, is_export: bool, start_date: &Date, end_date: &Date, billing_timezone: &time_tz::Tz) -> anyhow::Result<IndexMap<String, (Tariff, Vec<meter::electricity_agreement_line_items::LineItemType>)>>{
        let properties = self.get_properties(account_number).await?;
        // if let std::collections::hash_map::Entry::Vacant(entry) = self.properties.entry(account_number.clone()) {
        //     entry.insert(PropertyList::new(&self.cache_manager, &self.request_manager, account_number.clone()).await?);
        // }
        
        // let properties = self.properties.get(account_number).unwrap();
        let meter_agreements = //MeterAgreementList::new(&self.cache_manager, &self.request_manager, account_number.clone(), &properties.meter_node_ids).await?;
            MeterAgreementDataSet::new(&self.cache_manager, &self.request_manager, &account_number, &properties.meter_node_ids).await?;

        let start_date_time = start_date.at_midnight(billing_timezone);
        let end_date_time = end_date.at_next_midnight(billing_timezone);

        //println!("get_line_items {:?} - {:?}", start_date_time, end_date_time);

        let in_scope_agreements = meter_agreements.get_in_scope(meter_type, is_export, &start_date_time, &end_date_time);

        async fn get_line_items2(
            cache_manager: &CacheConfig, request_manager: &RequestManager,
            account_number: &String, meter_type: &MeterType, agreement_id: &String,
            start_date_time: &DateTime, end_date_time: &DateTime, billing_timezone: &time_tz::Tz) -> anyhow::Result<Vec<meter::electricity_agreement_line_items::LineItemType>> {
                let mut in_scope_items = Vec::new();
                let mut bucket_date = start_date_time.to_date();
                loop {
                    println!("Get bucket {:?}", bucket_date);
                    let line_items = AgreementLineItems::new(cache_manager, request_manager, account_number.clone(), meter_type, agreement_id.clone(), &bucket_date, billing_timezone).await?;

                    for (_cursor, (_, item)) in line_items.line_items {
                        println!("Candidate line item {:?}-{:?}", item.start_at_, item.end_at_);
                        if &item.start_at_ >= end_date_time {
                            // thats it
                            println!("Line item has date {:?} so we are done", item.start_at_);
                            return Ok(in_scope_items)
                        }
                        if &item.start_at_ >= start_date_time {
                            println!("Save {:?} - {:?}", item.start_at_, item.end_at_);
                            in_scope_items.push(item);
                        }
                    }

                    bucket_date = line_items.end_date;

                    println!("New bucket date {:?}", bucket_date);

                    if *bucket_date > end_date_time.date() {
                        println!("{:?} > date {:?} so we are done", bucket_date, end_date_time);
                        return Ok(in_scope_items)
                    }
                }
        }
        
        let mut item_map = IndexMap::new();

        for (agreement_id, tariff) in in_scope_agreements {
            // let agreement_id = agreement_id.to_string(); // Ugh!

            // tariff.print();
            let in_scope_items = get_line_items2(&self.cache_manager, &self.request_manager, account_number, meter_type, &agreement_id, &start_date_time, &end_date_time, billing_timezone).await?;

            if in_scope_items.is_empty() {
                //println!("Got no in scope items");
            }
            else {
                //println!("Got {} in scope items", in_scope_items.len());
                //println!("First {}", serde_json::to_string_pretty(in_scope_items.get(0).unwrap())?);
                //println!("Last {}", serde_json::to_string_pretty(in_scope_items.get(in_scope_items.len() - 1).unwrap())?);


                item_map.insert(agreement_id, (tariff, in_scope_items));
            }
        }

        Ok(item_map)
    }



    pub async fn get_consumption(
        &self,
        account_number: &String,
        meter_node_id: &String,
        from: &DateTime,
        to: &Option<DateTime>,
        billing_timezone: &time_tz::Tz,
    ) -> anyhow::Result<Vec<(DataSetAttributes, meter::meter_consumption::ConsumptionType)>> {
        let end = to.clone().unwrap_or_else(DateTime::now_utc);
        let first_date = from.to_timezone(billing_timezone).to_date();
        let mut month = Date::from_calendar_date(first_date.year(), first_date.month(), 1)?;
        let mut datasets = self.consumption.lock().await;

        while month.at_midnight(billing_timezone) < end {
            let month_key = consumption_cache_key(account_number, meter_node_id, &month);
            let is_new = !datasets.contains_key(&month_key);
            if is_new {
                let dataset = ConsumptionDataSet::new(
                    &self.cache_manager,
                    &self.request_manager,
                    account_number.clone(),
                    meter_node_id.clone(),
                    &month,
                    billing_timezone,
                )
                .await?;
                datasets.insert(month_key.clone(), dataset);
            }

            let overflow = {
                let dataset = datasets.get_mut(&month_key).expect("dataset was inserted");
                if !is_new && !dataset.data.is_full(billing_timezone) {
                    dataset
                        .refresh(&self.request_manager, billing_timezone)
                        .await?;
                }
                dataset.data.take_overflow()
            };

            for (overflow_month, records) in overflow {
                let overflow_key =
                    consumption_cache_key(account_number, meter_node_id, &overflow_month);
                if !datasets.contains_key(&overflow_key) {
                    let dataset = ConsumptionDataSet::load(
                        &self.cache_manager,
                        account_number.clone(),
                        meter_node_id.clone(),
                        &overflow_month,
                        billing_timezone,
                    )?;
                    datasets.insert(overflow_key.clone(), dataset);
                }
                datasets
                    .get_mut(&overflow_key)
                    .expect("overflow dataset was inserted")
                    .data
                    .add_fetched(records);
            }

            month = next_month(&month)?;
        }

        for dataset in datasets.values_mut() {
            dataset.data.flush(&self.cache_manager)?;
        }

        let mut results = BTreeMap::new();
        let meter_cache_prefix = format!("{account_number}#{meter_node_id}#");
        for (cache_key, dataset) in datasets.iter() {
            if !cache_key.starts_with(&meter_cache_prefix) {
                continue;
            }
            for (start_at, (attributes, consumption)) in &dataset.data.data {
                if start_at >= from && start_at < &end {
                    results.insert(
                        start_at.clone(),
                        (
                            *attributes,
                            meter::meter_consumption::ConsumptionType {
                                value_: consumption.value_.clone(),
                                start_at_: consumption.start_at_.clone(),
                                end_at_: consumption.end_at_.clone(),
                            },
                        ),
                    );
                }
            }
        }

        Ok(results.into_values().collect())
    }

    pub fn gui_display(data: &Vec<(DataSetAttributes, meter::meter_consumption::ConsumptionType)>) -> Element {
        // let items = Vec::new();

        // for (attr, consumption) in data {
        //     items.push(consumption.gui_summary_line(attr)?);
        // }

        rsx! {
            table {
                {meter::meter_consumption::ConsumptionType::gui_summary_header()}
                for (attr , consumption) in data {
                    {consumption.gui_summary_line(attr)?}
                }
            }
        }
    }
}


impl meter::meter_consumption::ConsumptionType {

     pub fn gui_summary_header() -> Element{
        rsx!{
            tr { class: "header",
                th { "From" }
                th { "To" }
                th { "Value" }
            }
        }
    }

    pub fn gui_summary_line(&self, attributes: &DataSetAttributes) -> Element {
        let row_class = if attributes.cached { "cached"} else {"fetched"};

        rsx!{
            tr { class: row_class,
                td { "{self.start_at_}" }
                td { "{self.end_at_}" }
                td { "{self.value_}" }
            }
        }
    }
}


#[derive(Debug)]
pub enum Tariff {
    Electricity(meter::meter_agreements::ElectricityTariffType),
    Gas(meter::meter_agreements::GasTariffType)
}

fn format_rate(rate: f64) -> String {
    format!("{:7.4}", rate)
}


fn gui_rate(rate: f64) -> Element {
    rsx!{
        td { class: "numeric", {format_rate(rate)} }
    }
}

fn gui_opt_rate(rate: Option<f64>) -> Element {
    if let Some(rate) = rate {
        gui_rate(rate)
    }
    else {
        rsx!{
            td { class: "numeric invalid", {NULL} }
        }
    }
}


fn format_opt_str<T: ToString>(str: &Option<T>) -> String {
    if let Some(str) = str {
        str.to_string()
    }
    else {
        "UNKNOWN".to_string()
    }
}

fn gui_opt_str<T: ToString>(str: &Option<T>) -> Element {
    if let Some(str) = str {
        rsx!{
            td { {str.to_string()} }
        }
    }
    else {
        rsx!{
            td { class: "invalid", {NULL} }
        }
    }
}

impl Tariff {
    pub fn standing_charge(&self) -> Option<f64> {
        match self {
            Tariff::Electricity(electricity_tariff_type) => {
                match electricity_tariff_type {
                    meter::meter_agreements::ElectricityTariffType::StandardTariff(tariff) => { tariff.pre_vat_standing_charge_},
                    meter::meter_agreements::ElectricityTariffType::DayNightTariff(tariff) => { tariff.pre_vat_standing_charge_},
                    meter::meter_agreements::ElectricityTariffType::ThreeRateTariff(tariff) => { tariff.pre_vat_standing_charge_},
                    meter::meter_agreements::ElectricityTariffType::HalfHourlyTariff(tariff) => { tariff.pre_vat_standing_charge_},
                    meter::meter_agreements::ElectricityTariffType::PrepayTariff(tariff) => { tariff.pre_vat_standing_charge_},
                    meter::meter_agreements::ElectricityTariffType::FourRateEvTariff(tariff) => {tariff.pre_vat_standing_charge_},
                }
            },
            Tariff::Gas(tariff) => { tariff.pre_vat_standing_charge_},
        }
    }

    pub fn gui_display(&self) -> Element {
        match self {
            Tariff::Electricity(electricity_tariff_type) => {
                match electricity_tariff_type {
                    meter::meter_agreements::ElectricityTariffType::StandardTariff(tariff) => {
                        rsx!{
                            div {
                                h3 { "Electricity Tariff: {tariff.display_name_}" }
                                table { class: "display",
                                    tr {
                                        th { class: "row-header", "Full Name" }
                                        td { {tariff.full_name_.as_str()} }
                                    }
                                    tr {
                                        th { class: "row-header", "Code" }
                                        td { {tariff.tariff_code_.as_str()} }
                                    }
                                    tr {
                                        th { class: "row-header", "Pre-VAT Standing Charge" }
                                        {gui_opt_rate(tariff.pre_vat_standing_charge_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Standing Charge" }
                                        {gui_opt_rate(tariff.standing_charge_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Pre-VAT Unit Rate" }
                                        {gui_opt_rate(tariff.pre_vat_unit_rate_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Unit Rate" }
                                        {gui_opt_rate(tariff.unit_rate_)}
                                    }
                                }
                            }
                        }
                    },
                    meter::meter_agreements::ElectricityTariffType::DayNightTariff(_tariff) => todo!(),
                    meter::meter_agreements::ElectricityTariffType::ThreeRateTariff(_tariff) => todo!(),
                    meter::meter_agreements::ElectricityTariffType::HalfHourlyTariff(tariff) => {
                        rsx!{
                            div {
                                h3 { "Electricity Tariff: {format_opt_str(&tariff.display_name_)}" }
                                table { class: "display",
                                    tr {
                                        th { class: "row-header", "Full Name" }
                                        {gui_opt_str(&tariff.full_name_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Display Name" }
                                        {gui_opt_str(&tariff.display_name_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Tariff Code" }
                                        {gui_opt_str(&tariff.tariff_code_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Product Code" }
                                        {gui_opt_str(&tariff.product_code_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Pre-VAT Standing Charge" }
                                        {gui_opt_rate(tariff.pre_vat_standing_charge_)}
                                    }
                                    tr {
                                        th { class: "row-header", "Standing Charge" }
                                        {gui_opt_rate(tariff.standing_charge_)}
                                    }
                                
                                }
                            
                            // h4 { "Unit Rates" }
                            // table { class: "display",
                            //     tr {
                            //         th { "Valid From" }
                            //         th { "Valid To" }
                            //         th { "Pre-VAT Unit Rate" }
                            //         th { "Unit Rate" }
                            //     }
                            //     for rate in &tariff.unit_rates_ {
                            //         tr {
                            //             {gui_opt_str(&rate.valid_from_)}
                            //             {gui_opt_str(&rate.valid_to_)}
                            //             {gui_opt_rate(rate.pre_vat_value_)}
                            //             {gui_opt_rate(rate.value_)}
                            //         }
                            //     }
                            // }
                            }
                        }
                    },
                    meter::meter_agreements::ElectricityTariffType::PrepayTariff(_tariff) => todo!(),
                    meter::meter_agreements::ElectricityTariffType::FourRateEvTariff(_tariff) => todo!(),
                }
            },
            Tariff::Gas(gas_tariff_type) => {
                rsx!{
                    div {
                        h3 { "Gas Tariff: {gas_tariff_type.full_name_}" }
                        table { class: "display",
                            tr {
                                th { class: "row-header", "Code" }
                                td { {gas_tariff_type.tariff_code_.as_str()} }
                            }
                            tr {
                                th { class: "row-header", "Standing Charge" }
                                {gui_opt_rate(gas_tariff_type.standing_charge_)}
                            }
                            tr {
                                th { class: "row-header", "Pre-VAT Unit Rate" }
                                {gui_opt_rate(gas_tariff_type.pre_vat_unit_rate_)}
                            }
                            tr {
                                th { class: "row-header", "Unit Rate" }
                                {gui_opt_rate(gas_tariff_type.unit_rate_)}
                            }
                        }
                    }
                }
            },
        }
    }
}

pub struct MeterAgreementDataSet {
    pub import_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
    pub export_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
    pub gas_map: HashMap<String, Vec<meter::meter_agreements::GasAgreementType>>,
}

impl MeterAgreementDataSet {
    pub async fn new(config: &Arc<CacheConfig>, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: &str, meter_node_ids: &Vec<String>) -> anyhow::Result<Self> {
        let hash_key = format!("{}#MeterAgreements", account_number);
        let the_beginning: DateTime = DateTime::from_calendar_date(2000, time::Month::January, 1)?;
        let mut queries = Vec::new();

        for meter_node_id in meter_node_ids {
            let query = meter::meter_agreements::Query::builder()
                .with_meter_node_id(meter_node_id.clone())
                .with_valid_after(the_beginning.clone())
                .build();

            queries.push(query);
        }
        let data = MultiQueryDataSet::new(
            &hash_key,
            Duration::from_hours(24),
            queries,
            config,
            request_manager).await?;
        

        let mut export_electricity_map = HashMap::new();
        let mut import_electricity_map = HashMap::new();
        let mut gas_map = HashMap::new();

        for response in data.data {
            match response.node_ {
                meter::meter_agreements::Node::ElectricityMeterType(electricity_meter_type) => {
                    if electricity_meter_type.import_meter_.is_some() {
                        export_electricity_map.insert(electricity_meter_type.node_id_, electricity_meter_type.meter_point_.agreements_);
                    }
                    else {
                        import_electricity_map.insert(electricity_meter_type.node_id_, electricity_meter_type.meter_point_.agreements_);
                    }
                },
                meter::meter_agreements::Node::GasMeterType(gas_meter_type) => {
                    gas_map.insert(gas_meter_type.node_id_, gas_meter_type.meter_point_.agreements_);
                },
                _ => unreachable!()
            }
        }
        
        Ok(Self {
            export_electricity_map,
            import_electricity_map,
            gas_map,
        })
    }

    fn get_in_scope(self, meter_type: &MeterType, is_export: bool, start_date: &DateTime, end_date: &DateTime) -> Vec<(String, Tariff)> {
        let mut in_scope_agreements = Vec::new();

        match meter_type {
            MeterType::Gas => {
                for (_meter_node_id, agreement_vec) in self.gas_map {
                    for agreement in agreement_vec {
                        if &agreement.valid_from_ <= end_date {
                            if let Some(valid_to) = &agreement.valid_to_ {
                                if valid_to >= start_date {
                                    in_scope_agreements.push((agreement.id_.to_string(), Tariff::Gas(agreement.tariff_)));
                                }
                            }
                            else {
                                in_scope_agreements.push((agreement.id_.to_string(), Tariff::Gas(agreement.tariff_)));
                            }
                        }
                    }
                }
            },
            MeterType::Electricity => {
                for (_meter_node_id, agreement_vec) in if is_export {self.export_electricity_map} else {self.import_electricity_map} {
                    for agreement in agreement_vec {
                        if &agreement.valid_from_ <= end_date {
                            if let Some(valid_to) = &agreement.valid_to_ {
                                if valid_to >= start_date {
                                    in_scope_agreements.push((agreement.id_.to_string(), Tariff::Electricity(agreement.tariff_)));
                                }
                            }
                            else {
                                in_scope_agreements.push((agreement.id_.to_string(), Tariff::Electricity(agreement.tariff_)));
                            }
                        }
                    }
                }
            },
        }

        in_scope_agreements
    }
}

// pub struct MeterAgreementList {
//     pub _account_number: String,
//     pub import_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
//     pub export_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
//     pub gas_map: HashMap<String, Vec<meter::meter_agreements::GasAgreementType>>,
//     _hash_key: String,
// }

// impl MeterAgreementList {
//     async fn new(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: String, meter_node_ids: &Vec<String>) -> anyhow::Result<Self> {
//         let hash_key = format!("{}#MeterAgreements", account_number);
//             let the_beginning: DateTime = DateTime::from_calendar_date(2000, time::Month::January, 1)?;
//             let mut agreements = Vec::new();
    
//             cache_manager.read_vec(&hash_key, &mut agreements)?;
    
//             let cached_cnt = agreements.len();
    
//             if agreements.is_empty() {
//                 for meter_node_id in meter_node_ids {
//                     let query = meter::meter_agreements::Query::builder()
//                             .with_meter_node_id(meter_node_id.clone())
//                             .with_valid_after(the_beginning.clone())
//                             .build()?;
//                     let response = request_manager.call(&query).await?;

//                     agreements.push((meter_node_id.clone(), response));
//                 }
//                 cache_manager.write_vec(&hash_key, &agreements, cached_cnt)?;
//             }

//             let mut export_electricity_map = HashMap::new();
//             let mut import_electricity_map = HashMap::new();
//             let mut gas_map = HashMap::new();

//             for (meter_node_id, response) in agreements {
//                 match response.node_ {
//                     meter::meter_agreements::Node::ElectricityMeterType(electricity_meter_type) => {
//                         if electricity_meter_type.import_meter_.is_some() {
//                             export_electricity_map.insert(meter_node_id, electricity_meter_type.meter_point_.agreements_);
//                         }
//                         else {
//                             import_electricity_map.insert(meter_node_id, electricity_meter_type.meter_point_.agreements_);
//                         }
//                     },
//                     meter::meter_agreements::Node::GasMeterType(gas_meter_type) => {
//                         gas_map.insert(meter_node_id, gas_meter_type.meter_point_.agreements_);
//                     },
//                     _ => unreachable!()
//                 }
//             }
            
//             Ok(MeterAgreementList {
//                 _account_number: account_number,
//                 export_electricity_map,
//                 import_electricity_map,
//                 gas_map,
//                 _hash_key: hash_key,
//             })
//         }

//     fn get_in_scope(self, meter_type: &MeterType, is_export: bool, start_date: &DateTime, end_date: &DateTime) -> Vec<(String, Tariff)> {
//         let mut in_scope_agreements = Vec::new();

//         match meter_type {
//             MeterType::Gas => {
//                 for (_meter_node_id, agreement_vec) in self.gas_map {
//                     for agreement in agreement_vec {
//                         if &agreement.valid_from_ <= end_date {
//                             if let Some(valid_to) = &agreement.valid_to_ {
//                                 if valid_to >= start_date {
//                                     in_scope_agreements.push((agreement.id_.to_string(), Tariff::Gas(agreement.tariff_)));
//                                 }
//                             }
//                             else {
//                                 in_scope_agreements.push((agreement.id_.to_string(), Tariff::Gas(agreement.tariff_)));
//                             }
//                         }
//                     }
//                 }
//             },
//             MeterType::Electricity => {
//                 for (_meter_node_id, agreement_vec) in if is_export {self.export_electricity_map} else {self.import_electricity_map} {
//                     for agreement in agreement_vec {
//                         if &agreement.valid_from_ <= end_date {
//                             if let Some(valid_to) = &agreement.valid_to_ {
//                                 if valid_to >= start_date {
//                                     in_scope_agreements.push((agreement.id_.to_string(), Tariff::Electricity(agreement.tariff_)));
//                                 }
//                             }
//                             else {
//                                 in_scope_agreements.push((agreement.id_.to_string(), Tariff::Electricity(agreement.tariff_)));
//                             }
//                         }
//                     }
//                 }
//             },
//         }

//         in_scope_agreements
//     }
    
// }

// impl meter::meter_consumption::Node {
//     pub fn as_consumption_page(&self) -> &ForwardPageOf<meter::meter_consumption::ConsumptionType> {
//         match self {
//             meter::meter_consumption::Node::ElectricityMeterType(electricity_meter) => {
//                 &electricity_meter.consumption_
//             },
//             meter::meter_consumption::Node::GasMeterType(gas_meter) => {
//                 &gas_meter.consumption_
//             },
//             _ => {
//                 unreachable!()
//                 //return Some(Err(anyhow!("Unexpected response type")))
//             },
//         }
//     }
// }

impl meter::electricity_agreement_line_items::AgreementInterface {

    pub fn as_electricity_agreement(self) -> meter::electricity_agreement_line_items::ElectricityAgreementType {    
        match self {
            meter::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
                electricity_agreement_type
            },
            meter::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_abstract_agreement_interface) => unreachable!(),
        }
    }
    pub fn as_electricity_agreement_ref(&self) -> &meter::electricity_agreement_line_items::ElectricityAgreementType {    
        match self {
            meter::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
                electricity_agreement_type
            },
            meter::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_abstract_agreement_interface) => unreachable!(),
        }
    }

    // pub fn get_line_items(&self) -> &Vec<EdgeOf<meter::electricity_agreement_line_items::LineItemType>> {
    //     match self {
    //         meter::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
    //             &electricity_agreement_type.line_items_.edges
    //             //.into_iter().collect()
    //         },
    //         meter::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_abstract_agreement_interface) => unreachable!(),
    //     }
    // }


    // pub fn get_page_info(&self) -> &PageInfo {
    //     match self {
    //         meter::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
    //             &electricity_agreement_type.line_items_.page_info
    //             //.into_iter().collect()
    //         },
    //         meter::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_abstract_agreement_interface) => unreachable!(),
    //     }
    // }
}

impl meter::gas_agreement_line_items::AgreementInterface {
    pub fn as_gas_agreement(self) -> meter::gas_agreement_line_items::GasAgreementType {
        match self {
            meter::gas_agreement_line_items::AgreementInterface::ElectricityAgreementType(_electricity_agreement_type) => unreachable!(),
            meter::gas_agreement_line_items::AgreementInterface::GasAgreementType(abstract_agreement_interface) => {
                abstract_agreement_interface
            },
        }
    }
    
    pub fn as_gas_agreement_ref(&self) -> &meter::gas_agreement_line_items::GasAgreementType {
        match self {
            meter::gas_agreement_line_items::AgreementInterface::ElectricityAgreementType(_electricity_agreement_type) => unreachable!(),
            meter::gas_agreement_line_items::AgreementInterface::GasAgreementType(abstract_agreement_interface) => {
                abstract_agreement_interface
            },
        }
    }
    
    // pub fn get_line_items(&self) -> &Vec<EdgeOf<meter::gas_agreement_line_items::LineItemType>> {
    //     match self {
    //         meter::gas_agreement_line_items::AgreementInterface::ElectricityAgreementType(_electricity_agreement_type) => unreachable!(),
    //         meter::gas_agreement_line_items::AgreementInterface::GasAgreementType(abstract_agreement_interface) => {
    //             &abstract_agreement_interface.line_items_.edges
    //         },
    //     }
    // }


    // pub fn get_page_info(&self) -> &PageInfo {
    //     match self {
    //         meter::gas_agreement_line_items::AgreementInterface::ElectricityAgreementType(_electricity_agreement_type) => unreachable!(),
    //         meter::gas_agreement_line_items::AgreementInterface::GasAgreementType(abstract_agreement_interface) =>{
    //             &abstract_agreement_interface.line_items_.page_info
    //         },
    //     }
    // }
}

impl Into<meter::electricity_agreement_line_items::LineItemType> for meter::gas_agreement_line_items::LineItemType {
    fn into(self) -> meter::electricity_agreement_line_items::LineItemType {
        meter::electricity_agreement_line_items::LineItemType {
            start_at_: self.start_at_,
            end_at_: self.end_at_,
            net_amount_: self.net_amount_,
            number_of_units_: self.number_of_units_,
            settlement_unit_:self.settlement_unit_,
        }
    }
}

#[allow(dead_code)]
pub struct AgreementLineItems {
    pub account_number: String,
    pub agreement_id: String,
    start_date: Date,
    end_date: Date,
    pub line_items: BTreeMap<DateTime, (crate::data_set::DataSetAttributes, meter::electricity_agreement_line_items::LineItemType)>,
}

impl AgreementLineItems {
    async fn new(
        config: &CacheConfig,
        request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
        account_number: String,
        meter_type: &MeterType,
        agreement_id: String,
        date: &Date,
        billing_timezone: &time_tz::Tz
    ) -> anyhow::Result<Self> {
        let hash_key = format!("{}#{}#{}AgreementTransactions", account_number, agreement_id, meter_type);
        
        let (
            start_date,
            end_date,
            line_items): (Date, Date, std::collections::BTreeMap<DateTime, (crate::data_set::DataSetAttributes, meter::electricity_agreement_line_items::LineItemType)>
        ) = match meter_type {
            MeterType::Gas => {
                let initial_query_provider = |bucket_start: &Date, last_record: Option<&meter::gas_agreement_line_items::LineItemType>| {
                    let start_date = if let Some(last_record) = last_record {
                        last_record.start_at_.clone()
                    }
                    else {
                        bucket_start.at_midnight(billing_timezone)
                    };
                    meter::gas_agreement_line_items::Query::builder()
                        .with_agreement_id(agreement_id.clone())
                        .with_start_at(start_date)
                        .with_timezone(String::from("Europe/London"))
                        .with_item_type(super::graphql::LineItemTypeOptions::ConsumptionCharge)
                        .with_line_item_grouping(super::graphql::LineItemGroupingOptions::None)
                        .with_first(50)
                        .build()

                };
                let continuation_query_provider = |_bucket_start: &Date, last_response: &meter::gas_agreement_line_items::Response| {
                    
                    let agreement = last_response.gas_agreement_.as_gas_agreement_ref();
                    if let Some(end_cursor) = agreement.line_items_.page_info.end_cursor.clone() {
                        let start_date = agreement.line_items_.edges.last().unwrap().node.start_at_.clone();
                        Some(meter::gas_agreement_line_items::Query::builder()
                            .with_agreement_id(agreement_id.clone())
                            .with_start_at(start_date)
                            .with_timezone(String::from("Europe/London"))
                            .with_item_type(super::graphql::LineItemTypeOptions::ConsumptionCharge)
                            .with_line_item_grouping(super::graphql::LineItemGroupingOptions::None)
                            .with_first(50)
                            .with_after(end_cursor)
                            .build())
                    }
                    else {
                        None
                    }
                };

                let response_iterator = |response: meter::gas_agreement_line_items::Response| {
                    response.gas_agreement_.as_gas_agreement().line_items_.edges.into_iter().map(|edge| edge.node)
                };

                let indexer = |item: &meter::gas_agreement_line_items::LineItemType| {
                    (item.start_at_.clone(), true)
                };

                 let data_set = MonthlyDataSet::new(&hash_key,
                    date,
                    Duration::from_hours(24),
                    initial_query_provider,
                    continuation_query_provider,
                    response_iterator,
                    indexer,
                    config,
                    request_manager
                ).await?;

                let mut data = BTreeMap::new();
                for (k, (a,v)) in data_set.data {
                    data.insert(k, (a, v.into()));
                }

                (data_set.start_date, data_set.end_date, data)
            },
            MeterType::Electricity => {
                let initial_query_provider = |bucket_start: &Date, last_record: Option<&meter::electricity_agreement_line_items::LineItemType>| {
                    let start_date = if let Some(last_record) = last_record {
                        last_record.start_at_.clone()
                    }
                    else {
                        bucket_start.at_midnight(billing_timezone)
                    };
                
                    meter::electricity_agreement_line_items::Query::builder()
                        .with_agreement_id(agreement_id.clone())
                        .with_start_at(start_date)
                        .with_timezone(String::from("Europe/London"))
                        .with_item_type(super::graphql::LineItemTypeOptions::ConsumptionCharge)
                        .with_line_item_grouping(super::graphql::LineItemGroupingOptions::None)
                        .with_first(50)
                        .build()

                };
                let continuation_query_provider = |_bucket_start: &Date, last_response: &meter::electricity_agreement_line_items::Response| {
                    // let end_cursor = last_response.electricity_agreement_.get_line_items().last().unwrap().cursor;
                
                    let agreement = last_response.electricity_agreement_.as_electricity_agreement_ref();
                    if let Some(end_cursor) = agreement.line_items_.page_info.end_cursor.clone() {
                        let start_date = agreement.line_items_.edges.last().unwrap().node.start_at_.clone();
                        Some(meter::electricity_agreement_line_items::Query::builder()
                        .with_agreement_id(agreement_id.clone())
                        .with_start_at(start_date.clone())
                        .with_timezone(String::from("Europe/London"))
                        .with_item_type(super::graphql::LineItemTypeOptions::ConsumptionCharge)
                        .with_line_item_grouping(super::graphql::LineItemGroupingOptions::None)
                        .with_first(50)
                        .with_after(end_cursor)
                        .build())
                    }
                    else {
                        None
                    }
                };


                let response_iterator = |response: meter::electricity_agreement_line_items::Response| {
                    response.electricity_agreement_.as_electricity_agreement().line_items_.edges.into_iter().map(|edge| edge.node)
                };

                let indexer = |item: &meter::electricity_agreement_line_items::LineItemType| {
                    (item.start_at_.clone(), true)
                };

                 let data_set = MonthlyDataSet::new(&hash_key,
                    date,
                    Duration::from_hours(24),
                    initial_query_provider,
                    continuation_query_provider,
                    response_iterator,
                    indexer,
                    config,
                    request_manager
                ).await?;

                (data_set.start_date, data_set.end_date, data_set.data)
            },
        };

        Ok(Self {
            account_number,
            agreement_id,
            start_date,
            end_date,
            line_items,
        })
    }
}


fn consumption_cache_key(account_number: &str, meter_node_id: &str, month: &Date) -> String {
    format!(
        "{account_number}#{meter_node_id}#{}#ConsumptionRecords",
        month
    )
}

fn next_month(month: &Date) -> anyhow::Result<Date> {
    let (year, next_month) = if month.month() == time::Month::December {
        (month.year() + 1, time::Month::January)
    } else {
        (month.year(), month.month().next())
    };
    Ok(Date::from_calendar_date(year, next_month, 1)?)
}

pub struct ConsumptionDataSet {
    pub meter_node_id: String,
    pub data: MonthlyFixedSlotDataSet<meter::meter_consumption::ConsumptionType>,
}

impl ConsumptionDataSet {
    async fn new(
        config: &CacheConfig,
        request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
        account_number: String,
        meter_node_id: String,
        date: &Date,
        billing_timezone: &time_tz::Tz,
    ) -> anyhow::Result<Self> {
        let hash_key = format!("{account_number}#{meter_node_id}#ConsumptionRecords");
        let query_meter_id = meter_node_id.clone();
        let initial_query_provider =
            move |bucket_start: &Date,
                  _last_record: Option<&meter::meter_consumption::ConsumptionType>| {
                meter::meter_consumption::Query::builder()
                    .with_meter_id(query_meter_id.clone())
                    .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
                    .with_start_at(bucket_start.at_midnight(billing_timezone))
                    .with_timezone(String::from("Europe/London"))
                    .with_first(50)
                    .build()
            };
        let query_meter_id = meter_node_id.clone();
        let continuation_query_provider =
            move |bucket_start: &Date, last_response: &meter::meter_consumption::Response| {
                let page = match &last_response.node_ {
                    meter::meter_consumption::Node::ElectricityMeterType(electricity_meter) => {
                        &electricity_meter.consumption_
                    }
                    meter::meter_consumption::Node::GasMeterType(gas_meter) => {
                        &gas_meter.consumption_
                    }
                    _ => unreachable!(),
                };
                page.page_info.end_cursor.clone().map(|end_cursor| {
                    meter::meter_consumption::Query::builder()
                        .with_meter_id(query_meter_id.clone())
                        .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
                        .with_start_at(bucket_start.at_midnight(billing_timezone))
                        .with_timezone(String::from("Europe/London"))
                        .with_first(50)
                        .with_after(end_cursor)
                        .build()
                })
            };
        let response_iterator = |response: meter::meter_consumption::Response| {
            let page = match response.node_ {
                meter::meter_consumption::Node::ElectricityMeterType(electricity_meter) => {
                    electricity_meter.consumption_
                }
                meter::meter_consumption::Node::GasMeterType(gas_meter) => gas_meter.consumption_,
                _ => unreachable!(),
            };
            page.edges.into_iter().map(|edge| edge.node)
        };
        let indexer = |item: &meter::meter_consumption::ConsumptionType| item.start_at_.clone();
        let data = MonthlyFixedSlotDataSet::new(
            &hash_key,
            date,
            48,
            billing_timezone,
            initial_query_provider,
            continuation_query_provider,
            response_iterator,
            indexer,
            config,
            request_manager,
        )
        .await?;

        Ok(Self {
            meter_node_id,
            data,
        })
    }

    fn load(
        config: &CacheConfig,
        account_number: String,
        meter_node_id: String,
        date: &Date,
        billing_timezone: &time_tz::Tz,
    ) -> anyhow::Result<Self> {
        let hash_key = format!("{account_number}#{meter_node_id}#ConsumptionRecords");
        let indexer = |item: &meter::meter_consumption::ConsumptionType| item.start_at_.clone();
        let data =
            MonthlyFixedSlotDataSet::load(&hash_key, date, 48, billing_timezone, config, indexer)?;

        Ok(Self {
            meter_node_id,
            data,
        })
    }

    async fn refresh(
        &mut self,
        request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
        billing_timezone: &time_tz::Tz,
    ) -> anyhow::Result<()> {
        let meter_node_id = self.meter_node_id.clone();
        let initial_query_provider =
            |bucket_start: &Date,
             _last_record: Option<&meter::meter_consumption::ConsumptionType>| {
                meter::meter_consumption::Query::builder()
                    .with_meter_id(meter_node_id.clone())
                    .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
                    .with_start_at(bucket_start.at_midnight(billing_timezone))
                    .with_timezone(String::from("Europe/London"))
                    .with_first(50)
                    .build()
            };
        let continuation_query_provider =
            |bucket_start: &Date, last_response: &meter::meter_consumption::Response| {
                let page = match &last_response.node_ {
                    meter::meter_consumption::Node::ElectricityMeterType(electricity_meter) => {
                        &electricity_meter.consumption_
                    }
                    meter::meter_consumption::Node::GasMeterType(gas_meter) => {
                        &gas_meter.consumption_
                    }
                    _ => unreachable!(),
                };

                page.page_info.end_cursor.clone().map(|end_cursor| {
                    meter::meter_consumption::Query::builder()
                        .with_meter_id(meter_node_id.clone())
                        .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
                        .with_start_at(bucket_start.at_midnight(billing_timezone))
                        .with_timezone(String::from("Europe/London"))
                        .with_first(50)
                        .with_after(end_cursor)
                        .build()
                })
            };
        let response_iterator = |response: meter::meter_consumption::Response| {
            let page = match response.node_ {
                meter::meter_consumption::Node::ElectricityMeterType(electricity_meter) => {
                    electricity_meter.consumption_
                }
                meter::meter_consumption::Node::GasMeterType(gas_meter) => gas_meter.consumption_,
                _ => unreachable!(),
            };
            page.edges.into_iter().map(|edge| edge.node)
        };
        let indexer = |item: &meter::meter_consumption::ConsumptionType| item.start_at_.clone();

        self.data
            .refresh(
                48,
                billing_timezone,
                initial_query_provider,
                continuation_query_provider,
                response_iterator,
                indexer,
                request_manager,
            )
            .await?;
        Ok(())
    }
}


*/