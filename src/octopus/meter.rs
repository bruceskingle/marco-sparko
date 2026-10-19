use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

use dioxus::prelude::*;
use indexmap::IndexMap;
use rust_decimal::prelude::ToPrimitive;
use sparko_graphql::types::{Date, DateTime, PageOf};
use sparko_graphql::AuthenticatedRequestManager;
use sparko_graphql::GraphQLQueryBuilder;
use time_tz::Tz;
use time_tz::TimeZone;
use anyhow::anyhow;

use crate::data_set::{MonthlyDataSet, MonthlyFixedSlotDataSet, MultiQueryDataSet, SlotSource};
use crate::octopus::decimal::Decimal;
use crate::octopus::graphql::{ElectricityFiltersInput, ReadingFrequencyType, UtilityFiltersInput, UtilityFiltersInputBuilder};
use crate::octopus::meter_point::MeterPointDataSet;
use crate::octopus::property::PropertyDataSet;
use crate::{CacheConfig, NULL};

use super::consumption_graph::{ConsumptionGraph, GraphPoint, GraphStyle};
use super::stacked_bar_graph::{
    StackedBarGraph, StackedBarMode, StackedBarPeriod, StackedBarPoint,
};
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
    pub cache_config: Arc<CacheConfig>,
    pub request_manager: Arc<RequestManager>,
}

//   Rita the
impl MeterManager {
    pub fn new(cache_config: &Arc<CacheConfig>, request_manager: &Arc<RequestManager>)  -> Self {
        Self {
            cache_config: cache_config.clone(),
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
        PropertyDataSet::new(account_number, &self.cache_config, &self.request_manager).await
    }

    pub async fn get_meter_points(&self, account_number: &String) -> anyhow::Result<MeterPointDataSet>{
        MeterPointDataSet::new(account_number, &self.cache_config, &self.request_manager).await
    }

    pub async fn get_line_items(&self, account_number: &String, meter_type: &MeterType, is_export: bool, start_date: &Date, end_date: &Date, billing_timezone: &time_tz::Tz) -> anyhow::Result<IndexMap<String, (Tariff, Vec<meter::electricity_agreement_line_items::LineItemType>)>>{
        let properties = self.get_properties(account_number).await?;
        let meter_agreements = 
            MeterAgreementDataSet::new(&self.cache_config, &self.request_manager, &account_number, &properties.meter_node_ids).await?;

        let start_date_time = start_date.at_midnight(billing_timezone);
        let end_date_time = end_date.at_next_midnight(billing_timezone);

        //println!("get_line_items {:?} - {:?}", start_date_time, end_date_time);

        let in_scope_agreements = meter_agreements.get_in_scope(meter_type, is_export, &start_date_time, &end_date_time);

        async fn get_line_items2(
            cache_config: &CacheConfig, request_manager: &RequestManager,
            account_number: &String, meter_type: &MeterType, agreement_id: &String,
            start_date_time: &DateTime, end_date_time: &DateTime, billing_timezone: &time_tz::Tz) -> anyhow::Result<Vec<meter::electricity_agreement_line_items::LineItemType>> {
                let mut in_scope_items = Vec::new();
                let mut bucket_date = start_date_time.to_date();
                loop {
                    println!("Get bucket {:?}", bucket_date);
                    let line_items = AgreementLineItems::new(cache_config, request_manager, account_number.clone(), meter_type, agreement_id.clone(), &bucket_date, billing_timezone).await?;

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
            let in_scope_items = get_line_items2(&self.cache_config, &self.request_manager, account_number, meter_type, &agreement_id, &start_date_time, &end_date_time, billing_timezone).await?;

            if !in_scope_items.is_empty() {
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
            hash_key: format!("{}#{}#ConsumptionRecords", account_number, meter_node_id),
        };
        
        MonthlyFixedSlotDataSet::fetch(
            source,
            tz,
            &self.cache_config,
            &self.request_manager,
            start,
            end
        ).await
    }

    pub fn display_consumption(data: &Vec<MonthlyFixedSlotDataSet<ConsumptionSource>>, start: &DateTime, end: &Option<DateTime>) -> Element {
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

    pub fn display_consumption_graph(data: &Vec<MonthlyFixedSlotDataSet<ConsumptionSource>>, start: &DateTime, end: &Option<DateTime>, style: GraphStyle) -> Element {
        let date_format = time::format_description::parse("[year]-[month]-[day]").unwrap();
        let day_format = time::format_description::parse("[day] [month repr:short]").unwrap();
        let time_format = time::format_description::parse("[hour]:[minute]").unwrap();

        let values: Vec<_> = data.iter()
            .flat_map(|data_set| data_set.data.range(start..))
            .filter(|(key, _)| end.as_ref().is_none_or(|end| *key < end))
            .map(|(_, item)| &item.value)
            .collect();

        // Label each day, unless there is only one in which case label every 3 hours.
        let single_day = values.first().map(|v| v.start_at_.to_date()) == values.last().map(|v| v.start_at_.to_date());

        let mut points = Vec::with_capacity(values.len());
        let mut last_date = None;
        for value in values {
            let date = value.start_at_.to_date();
            let tick_label = if single_day {
                (value.start_at_.minute() == 0 && value.start_at_.hour() % 3 == 0)
                    .then(|| value.start_at_.format(&time_format).unwrap())
            } else {
                (last_date != Some(date)).then(|| value.start_at_.format(&day_format).unwrap())
            };
            last_date = Some(date);

            points.push(GraphPoint {
                start: value.start_at_.unix_timestamp(),
                end: value.end_at_.unix_timestamp(),
                value: value.value_.to_f64().unwrap_or_default(),
                tick_label,
                period_label: format!("{} {}–{}",
                    value.start_at_.format(&date_format).unwrap(),
                    value.start_at_.format(&time_format).unwrap(),
                    value.end_at_.format(&time_format).unwrap()),
            });
        }

        rsx! {
            ConsumptionGraph { points, style }
        }
    }





    pub async fn get_home_and_ev_consumption(
        &self,
        account_number: &String,
        mpan: &String,
        start: &DateTime,
        end: &Option<DateTime>,
        tz: &'static Tz)
        -> anyhow::Result<Vec<MonthlyFixedSlotDataSet<HomeAndEvConsumptionSource>>>
    {

        let end = if let Some(end) = end { end.clone() } else {DateTime::now(tz)};
        let source = HomeAndEvConsumptionSource{
            account_number: account_number.clone(),
            mpan: mpan.clone(),
            hash_key: format!("{}#{}#HomeAndEvConsumption", account_number, mpan),
            end_at: end,
            tz,
        };
        
        MonthlyFixedSlotDataSet::fetch(
            source,
            tz,
            &self.cache_config,
            &self.request_manager,
            start,
            &Some(end)
        ).await
    }

    pub fn display_home_and_ev_consumption(data: &Vec<MonthlyFixedSlotDataSet<HomeAndEvConsumptionSource>>, start: &DateTime, end: &Option<DateTime>) -> Element {
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
                    let item_start_date = item.value.start.to_date();
                    let (start_date, start_time) = if last_date == item_start_date {
                        ("".to_string(), item.value.start.format(&time_format).unwrap())
                    } else {
                        (item.value.start.format(&date_format).unwrap(), item.value.start.format(&time_format).unwrap())
                    };
                    
                    let end_time = item.value.end.format(&time_format).unwrap();
                    // let amount = format!("{:.3}", item.value.value_);

                    items.push(rsx!{
                        tr { class: row_class,
                            td { "{start_date}" }
                            td { "{start_time}" }
                            td { "{end_time}" }
                            td { "{item.value.home_standard.units}" }
                            td { "{item.value.home_standard.cost}" }
                            td { "{item.value.home_standard.rate}" }
                            td { "{item.value.home_low.units}" }
                            td { "{item.value.home_low.cost}" }
                            td { "{item.value.home_low.rate}" }
                            td { "{item.value.ev_standard.units}" }
                            td { "{item.value.ev_standard.cost}" }
                            td { "{item.value.ev_standard.rate}" }
                            td { "{item.value.ev_low.units}" }
                            td { "{item.value.ev_low.cost}" }
                            td { "{item.value.ev_low.rate}" }
                            td { "{item.value.standing_charge}" }
                        }
                    }
                    );
                }
                last_date = key.to_date();
            }
        };

        rsx! {
            table {
                tr {
                    td { colspan: 3 }
                    th { colspan: 3, class: "header", "Home Standard" }
                    th { colspan: 3, class: "header", "Home Low" }
                    th { colspan: 3, class: "header", "EV Standard" }
                    th { colspan: 3, class: "header", "EV Low" }
                }
                tr { class: "header",
                    th { colspan: 2, "From" }
                    th { "To" }
                    th { "Units" }
                    th { "Cost" }
                    th { "Rate" }
                    th { "Units" }
                    th { "Cost" }
                    th { "Rate" }
                    th { "Units" }
                    th { "Cost" }
                    th { "Rate" }
                    th { "Units" }
                    th { "Cost" }
                    th { "Rate" }
                    th { "Standing Charge" }
                }
                for item in items {
                    {item}
                }
            }
        }
    }

    pub fn display_home_and_ev_consumption_graph(data: &Vec<MonthlyFixedSlotDataSet<HomeAndEvConsumptionSource>>, start: &DateTime, end: &Option<DateTime>, tz: &'static Tz, mode: StackedBarMode, period: StackedBarPeriod) -> Element {
        let date_format = time::format_description::parse("[year]-[month]-[day]").unwrap();
        let day_format = time::format_description::parse("[day] [month repr:short]").unwrap();
        let time_format = time::format_description::parse("[hour]:[minute]").unwrap();
        let month_format = time::format_description::parse("[month repr:short] [year]").unwrap();

        let values: Vec<_> = data.iter()
            .flat_map(|data_set| data_set.data.range(start..))
            .filter(|(key, _)| end.as_ref().is_none_or(|end| *key < end))
            .map(|(_, item)| &item.value)
            .collect();

        let mut points = Vec::with_capacity(values.len());
        let single_day = period == StackedBarPeriod::Raw
            && values.first().map(|value| value.start.to_date())
                == values.last().map(|value| value.start.to_date());
        let mut last_date = None;
        let mut last_bucket_start = None;

        for value in values {
            let date = value.start.to_date();
            let (bucket_start, bucket_end) = match period {
                StackedBarPeriod::Raw => (value.start, value.end),
                StackedBarPeriod::Day => {
                    let bucket_start = date.at_midnight(tz);
                    (bucket_start, date.next_day().at_midnight(tz))
                }
                StackedBarPeriod::Week => {
                    let days_since_monday = date.weekday().number_days_from_monday() as i64;
                    let week_start = time::Date::from_calendar_date(
                        date.year(),
                        date.month(),
                        date.day(),
                    )
                    .unwrap()
                        - time::Duration::days(days_since_monday);
                    let week_start = Date::from_calendar_date(
                        week_start.year(),
                        week_start.month(),
                        week_start.day(),
                    )
                    .unwrap();
                    let week_end = week_start.next_day().next_day().next_day().next_day()
                        .next_day().next_day().next_day();
                    (week_start.at_midnight(tz), week_end.at_midnight(tz))
                }
                StackedBarPeriod::Month => {
                    let month_start = time::Date::from_calendar_date(
                        date.year(),
                        date.month(),
                        1,
                    )
                    .unwrap();
                    let next_month = if date.month() == time::Month::December {
                        time::Date::from_calendar_date(date.year() + 1, time::Month::January, 1)
                    } else {
                        time::Date::from_calendar_date(
                            date.year(),
                            time::Month::try_from(date.month() as u8 + 1).unwrap(),
                            1,
                        )
                    }
                    .unwrap();
                    let start_date = Date::from_calendar_date(
                        month_start.year(),
                        month_start.month(),
                        month_start.day(),
                    )
                    .unwrap();
                    let end_date = Date::from_calendar_date(
                        next_month.year(),
                        next_month.month(),
                        next_month.day(),
                    )
                    .unwrap();
                    (start_date.at_midnight(tz), end_date.at_midnight(tz))
                }
            };
            let tick_label = match period {
                StackedBarPeriod::Raw if single_day => {
                    (value.start.hour() % 3 == 0 && value.start.minute() == 0)
                        .then(|| value.start.format(&time_format).unwrap())
                }
                StackedBarPeriod::Raw => {
                    (last_date != Some(date)).then(|| date.format(&day_format).unwrap())
                }
                StackedBarPeriod::Day => {
                    (last_bucket_start != Some(bucket_start.unix_timestamp()))
                        .then(|| bucket_start.format(&day_format).unwrap())
                }
                StackedBarPeriod::Week => {
                    (last_bucket_start != Some(bucket_start.unix_timestamp())).then(|| {
                        format!("Week of {}", bucket_start.format(&day_format).unwrap())
                    })
                }
                StackedBarPeriod::Month => {
                    (last_bucket_start != Some(bucket_start.unix_timestamp()))
                        .then(|| bucket_start.format(&month_format).unwrap())
                }
            };
            let group_label = match period {
                StackedBarPeriod::Raw => format!(
                    "{} {}–{}",
                    value.start.format(&date_format).unwrap(),
                    value.start.format(&time_format).unwrap(),
                    value.end.format(&time_format).unwrap(),
                ),
                StackedBarPeriod::Day => bucket_start.format(&date_format).unwrap(),
                StackedBarPeriod::Week => format!(
                    "{} – {}",
                    bucket_start.format(&date_format).unwrap(),
                    bucket_end.to_date().previous_day().format(&date_format).unwrap(),
                ),
                StackedBarPeriod::Month => bucket_start.format(&month_format).unwrap(),
            };
            let point = StackedBarPoint {
                start: bucket_start.unix_timestamp(),
                end: bucket_end.unix_timestamp(),
                tick_label,
                period_label: group_label,
                home_standard_units: value.home_standard.units.to_f64().unwrap_or_default(),
                home_low_units: value.home_low.units.to_f64().unwrap_or_default(),
                ev_standard_units: value.ev_standard.units.to_f64().unwrap_or_default(),
                ev_low_units: value.ev_low.units.to_f64().unwrap_or_default(),
                home_standard_cost: value.home_standard.cost.to_f64().unwrap_or_default(),
                home_low_cost: value.home_low.cost.to_f64().unwrap_or_default(),
                ev_standard_cost: value.ev_standard.cost.to_f64().unwrap_or_default(),
                ev_low_cost: value.ev_low.cost.to_f64().unwrap_or_default(),
                standing_charge: value.standing_charge.to_f64().unwrap_or_default(),
            };

            if matches!(period, StackedBarPeriod::Raw) {
                points.push(point);
            } else if let Some(existing) = points.last_mut() {
                if existing.start == bucket_start.unix_timestamp() {
                    existing.add_values_from(&point);
                } else {
                    points.push(point);
                }
            } else {
                points.push(point);
            }
            last_date = Some(date);
            last_bucket_start = Some(bucket_start.unix_timestamp());
        }

        rsx! {
            StackedBarGraph { points, mode }
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

pub struct ConsumptionSource {
    pub meter_node_id:String,
    pub hash_key: String,
}

impl SlotSource for ConsumptionSource {
    type Value = meter::meter_consumption::ConsumptionType;

    type Response = meter::meter_consumption::Response;

    type Query = meter::meter_consumption::Query;

    fn hash_key(&self) -> &str {
        &self.hash_key
    }

    fn initial_query(&self, start: &DateTime) -> anyhow::Result<Self::Query> {
        Ok(meter::meter_consumption::Query::builder()
            .with_meter_id(self.meter_node_id.clone())
            .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
            .with_start_at(start.clone())
            .with_timezone(String::from("Europe/London"))
            .with_first(50)
            .build()?)
    }

    fn continuation_query(&self, start: &DateTime, last_response: &Self::Response) -> anyhow::Result<Option<Self::Query>> {
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

        Ok(if let Some(end_cursor) = &page.page_info.end_cursor {

            // let start_date = page.edges.last().unwrap().node.start_at_.clone();
            Some(meter::meter_consumption::Query::builder()
                .with_meter_id(self.meter_node_id.clone())
                .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
                .with_start_at(start.clone())
                .with_timezone(String::from("Europe/London"))
                .with_first(50)
                .with_after(end_cursor.clone())
                .build()?)
        }
        else {
            None
        })
    }

    fn items(&self, response: Self::Response) -> anyhow::Result<impl IntoIterator<Item = Self::Value>> {
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

        Ok(page.edges.into_iter().map(|edge| edge.node))
    }

    fn index(&self, value: &Self::Value) -> anyhow::Result<(DateTime, DateTime, bool)> {
       Ok((value.start_at_.clone(), value.end_at_.clone(), true))
    }
    
    fn refresh_after(&self) -> Duration {
        Duration::from_hours(24)
    }
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty, Eq, PartialEq, Clone)]
pub struct HomeAndEvReadingConsumption {
    units: Decimal,
    cost:  Decimal,
    rate: Decimal,
}

impl HomeAndEvReadingConsumption {
    fn from(stat: meter::home_and_ev_consumption_breakdown::StatisticOutput) -> Option<Self> {
        let estimated_money = stat.cost_incl_tax_?;
        Some(HomeAndEvReadingConsumption {
            units: stat.value_?,
            cost: estimated_money.estimated_amount_,
            rate: estimated_money.price_per_unit_?.amount_,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty, Eq, PartialEq, Clone)]
pub struct HomeAndEvReading {
    start:  DateTime,
    end: DateTime,
    home_standard: HomeAndEvReadingConsumption,
    ev_standard: HomeAndEvReadingConsumption,
    home_low: HomeAndEvReadingConsumption,
    ev_low: HomeAndEvReadingConsumption,
    standing_charge: Decimal,
}

impl HomeAndEvReading {
    fn from(node: meter::home_and_ev_consumption_breakdown::MeasurementInterface) -> Option<Self> {
        match node {
            meter::home_and_ev_consumption_breakdown::MeasurementInterface::MeasurementType(abstract_measurement_interface) => {
                println!("Unexpected abstract measurement {:?}", abstract_measurement_interface);
                None
            },
            meter::home_and_ev_consumption_breakdown::MeasurementInterface::IntervalMeasurementType(interval_measurement_type) => {
                if let Some(meta_data) = interval_measurement_type.measurement_interface_.meta_data_ {
                    let mut home_standard= None;
                    let mut ev_standard= None;
                    let mut home_low= None;
                    let mut ev_low= None;
                    let mut standing_charge = None;

                    for stat in meta_data.statistics_ {
                        match stat.type_.clone()? {
                            super::graphql::ReadingStatisticTypeEnum::StandingChargeCost => {
                                standing_charge = Some(stat.cost_incl_tax_?.estimated_amount_)
                            },
                            super::graphql::ReadingStatisticTypeEnum::ConsumptionCost => {
                                if let Some(label) = stat.label_.clone() {
                                    match label.as_str() {
                                        "EV Low Rate" => {
                                            ev_low = HomeAndEvReadingConsumption::from(stat);
                                        },
                                        "EV Standard Rate" => {
                                            ev_standard = HomeAndEvReadingConsumption::from(stat);
                                        },
                                        "Home Low Rate" => {
                                            home_low= HomeAndEvReadingConsumption::from(stat);
                                        },
                                        "Home Standard Rate" => {
                                            home_standard= HomeAndEvReadingConsumption::from(stat);
                                        },
                                        _ => {
                                            println!("Unknown meta data label {}", label);
                                            return None;
                                        }
                                    }
                                }
                                else {
                                    println!("No meta data label");
                                    return None;
                                }
                            },
                            _ => {
                                println!("Unexpected statistic type {:?}", stat.type_);
                                return None;
                            },
                        }
                    }

                    Some(HomeAndEvReading{
                        start: interval_measurement_type.start_at_,
                        end: interval_measurement_type.end_at_,
                        home_standard: home_standard?,
                        home_low: home_low?,
                        ev_standard: ev_standard?,
                        ev_low: ev_low?,
                        standing_charge: standing_charge?,
                    })
                }
                else {
                    None
                }
            },
        }
    }
}

pub struct HomeAndEvConsumptionSource {
    pub account_number: String,
    pub end_at: DateTime,
    pub tz: &'static Tz,
    pub mpan:String,
    pub hash_key: String,
}

impl HomeAndEvConsumptionSource {
    fn query_builder(&self) -> anyhow::Result<meter::home_and_ev_consumption_breakdown::QueryBuilder> {
        let mut utility_filters = Vec::new();
        let mut modelled_utility_filters = Vec::new();

        utility_filters.push(UtilityFiltersInput::builder()
            .with_electricity_filters(ElectricityFiltersInput::builder()
                .with_reading_frequency_type(ReadingFrequencyType::RawInterval)
                .with_reading_direction(super::graphql::ReadingDirectionType::Consumption)
                .with_market_supply_point_id(self.mpan.clone())
                .build()?)
            .build()?
        );

        modelled_utility_filters.push(UtilityFiltersInput::builder()
            .with_electricity_filters(ElectricityFiltersInput::builder()
                .with_reading_frequency_type(ReadingFrequencyType::RawInterval)
                .with_reading_direction(super::graphql::ReadingDirectionType::Consumption)
                .with_market_supply_point_id(self.mpan.clone())
                .with_modelled_sub_meter_readings(true)
                .build()?)
            .build()?
        );

        Ok(meter::home_and_ev_consumption_breakdown::Query::builder()
            .with_account_number(self.account_number.clone())
            .with_end_at(self.end_at)
            .with_timezone(self.tz.name().to_string())
            .with_utility_filters(utility_filters)
            .with_modelled_utility_filters(modelled_utility_filters)
            .with_first(50)
        )
    }

    fn get_measurements(last_response: &meter::home_and_ev_consumption_breakdown::Response) -> anyhow::Result<&PageOf<meter::home_and_ev_consumption_breakdown::MeasurementInterface>> {
            Ok(&last_response
                .account_.as_ref()
                .ok_or(anyhow!("No account in response"))?
                .properties_.first().ok_or(anyhow!("No property in response"))?
                // .electricity_meter_points_.first().ok_or("No meter points in response")?
                .measurements_

            )
        }
}

impl SlotSource for HomeAndEvConsumptionSource {
    type Value = HomeAndEvReading;

    type Response = meter::home_and_ev_consumption_breakdown::Response;

    type Query = meter::home_and_ev_consumption_breakdown::Query;

    fn hash_key(&self) -> &str {
        &self.hash_key
    }

    fn initial_query(&self, start: &DateTime) -> anyhow::Result<Self::Query> {
        Ok(self.query_builder()?
            .with_start_at(start.clone())
            .build()?
        )
    }

    fn continuation_query(&self, start: &DateTime, last_response: &Self::Response) -> anyhow::Result<Option<Self::Query>> {


        // fn get_cursor(last_response: &meter::home_and_ev_consumption_breakdown::Response) -> anyhow::Result<Option<String>> {
        //     Ok(last_response
        //         .account_.as_ref()
        //         .ok_or(anyhow!("No account in response"))?
        //         .properties_.first().ok_or(anyhow!("No property in response"))?
        //         // .electricity_meter_points_.first().ok_or("No meter points in response")?
        //         .measurements_.as_ref().ok_or(anyhow!("No measurements in response"))?
        //         .page_info_
        //         .end_cursor
        //         .as_ref()
        //         .cloned()
        //     )
        // }

        Ok(if let Some(end_cursor) = Self::get_measurements(&last_response)?
            .page_info
                .end_cursor
                .as_ref()
                .cloned()
        {
            Some(self.query_builder()?
                .with_start_at(start.clone())
                .with_cursor(end_cursor)
                .build()?)
        }
        else {
            None
        })
    }

    fn items(&self, response: Self::Response) -> anyhow::Result<impl IntoIterator<Item = Self::Value>> {
        // let measurements = response
        //         .account_
        //         .ok_or(anyhow!("No account in response"))?
        //         .properties_.first().ok_or(anyhow!("No property in response"))?
        //         // .electricity_meter_points_.first().ok_or("No meter points in response")?
        //         .measurements_;

        Ok(
            response
                .account_
                .ok_or(anyhow!("No account in response"))?
                .properties_.into_iter().next().ok_or(anyhow!("No property in response"))?
                // .electricity_meter_points_.first().ok_or("No meter points in response")?
                .measurements_
                .edges
                .into_iter()
                .filter_map(|edge| {
                    HomeAndEvReading::from(edge.node)
                    // match edge.node {
                        
                    //     meter::home_and_ev_consumption_breakdown::MeasurementInterface::MeasurementType(abstract_measurement_interface) => {
                    //         println!("Unexpected abstract measurement {:?}", abstract_measurement_interface);
                    //         None
                    //     },
                    //     meter::home_and_ev_consumption_breakdown::MeasurementInterface::IntervalMeasurementType(interval_measurement_type) => {
                    //         if let Some(meta_data) = interval_measurement_type.measurement_interface_.meta_data_ {
                    //             for stat in meta_data.statistics_ {
                    //                 stat.label_
                    //             }
                    //         }
                    //         Some(interval_measurement_type)
                    //     },
                    // }
                })
        )
    }

    fn index(&self, value: &Self::Value) -> anyhow::Result<(DateTime, DateTime, bool)> {
        Ok((value.start.clone(), value.end.clone(), true))
    }
    
    fn refresh_after(&self) -> Duration {
        Duration::from_hours(24)
    }
}