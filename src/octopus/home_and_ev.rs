use std::cell::Cell;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use dioxus::prelude::*;
use display_json::DisplayAsJsonPretty;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use sparko_graphql::types::{Currency, Date, DateTime, ForwardPageOf};
use sparko_graphql::{GraphQLCaller, GraphQLQueryBuilder};
use time_tz::{TimeZone, Tz};

use crate::CacheConfig;
use crate::data_set::{CancelToken, MonthlyFixedSlotDataSet, SlotSource};
use crate::octopus::decimal::Decimal;
use crate::octopus::graphql::{ElectricityFiltersInput, ReadingFrequencyType, UtilityFiltersInput};

use super::graphql::meter;
use super::stacked_bar_graph::{StackedBarGraph, StackedBarMode, StackedBarPeriod, StackedBarPoint};

/// Home and EV consumption readings for a meter over a period, cached in monthly buckets.
pub struct HomeAndEvDataSet {
    pub data: Vec<MonthlyFixedSlotDataSet<HomeAndEvConsumptionSource>>,
}

impl HomeAndEvDataSet {
    pub async fn new(
        account_number: &String,
        mpan: &String,
        start: &DateTime,
        end: &DateTime,
        tz: &'static Tz,
        on_progress: Box<dyn Fn(usize)>,
        cancel: &CancelToken,
        cache_config: &Arc<CacheConfig>,
        request_manager: &impl GraphQLCaller)
        -> anyhow::Result<Self>
    {

        let source = HomeAndEvConsumptionSource{
            account_number: account_number.clone(),
            mpan: mpan.clone(),
            hash_key: format!("{}#{}#HomeAndEvConsumption", account_number, mpan),
            end_at: end.clone(),
            tz,
            rows_loaded: Cell::new(0),
            on_progress,
        };
        
        let data = MonthlyFixedSlotDataSet::fetch(
            source,
            tz,
            cache_config,
            request_manager,
            start,
            end,
            cancel,
        ).await?;

        Ok(Self { data })
    }

    pub fn display(&self, start: &DateTime, end: &DateTime) -> Element {
        let date_format = time::format_description::parse("[year]-[month]-[day]").unwrap();
        let time_format = time::format_description::parse("[hour]:[minute]").unwrap();

        let mut items = Vec::new();
        let mut last_date = Date::from_calendar_date(0, time::Month::January, 1)?;
        for data_set in &self.data {
            for (key , item) in &data_set.data {
                let end_ok = key < end;
                

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

                    items.push(item.value.summary_line(row_class, &start_date, &start_time, &end_time));
                    last_date = key.to_date();
                }
            }
        };

        rsx! {
            table {
                {HomeAndEvReading::summary_headers()}
                for item in items {
                    {item}
                }
            }
        }
    }

    pub fn display_graph(&self, start: &DateTime, end: &DateTime, tz: &'static Tz, mode: StackedBarMode, period: StackedBarPeriod) -> Element {
        let date_format = time::format_description::parse("[year]-[month]-[day]").unwrap();
        let day_format = time::format_description::parse("[day] [month repr:short]").unwrap();
        let time_format = time::format_description::parse("[hour]:[minute]").unwrap();
        let month_format = time::format_description::parse("[month repr:short] [year]").unwrap();

        let values: Vec<_> = self.data.iter()
            .flat_map(|data_set| data_set.data.range(start..))
            .filter(|(key, _)| *key < end)
            .map(|(_, item)| &item.value)
            .collect();

        let mut points = Vec::with_capacity(values.len());
        let single_day = matches!(period, StackedBarPeriod::Raw | StackedBarPeriod::Rate)
            && values.first().map(|value| value.start.to_date())
                == values.last().map(|value| value.start.to_date());
        let mut last_date = None;
        let mut last_bucket_start = None;

        // For Rate, the rates and start of the run of readings in the last point, and the date of the last run start.
        let mut last_rates = None;
        let mut run_start = None;
        let mut last_run_date = None;
        let range_label = |from: &DateTime, to: &DateTime| {
            if from.to_date() == to.to_date() || *to == from.to_date().next_day().at_midnight(tz) {
                format!("{} {}–{}", from.format(&date_format).unwrap(), from.format(&time_format).unwrap(), to.format(&time_format).unwrap())
            }
            else {
                format!("{} {} – {} {}", from.format(&date_format).unwrap(), from.format(&time_format).unwrap(),
                    to.format(&date_format).unwrap(), to.format(&time_format).unwrap())
            }
        };

        for value in values {
            let date = value.start.to_date();
            let (bucket_start, bucket_end) = match period {
                StackedBarPeriod::Raw | StackedBarPeriod::Rate => (value.start, value.end),
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
                // Set below when a new run starts.
                StackedBarPeriod::Rate => None,
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
                StackedBarPeriod::Rate => range_label(&value.start, &value.end),
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
                currency: value.currency(),
                unit: value.unit.clone(),
            };

            if matches!(period, StackedBarPeriod::Raw) {
                points.push(point);
            } else if period == StackedBarPeriod::Rate {
                // Extend the last run if this reading follows on from it with the same rates, otherwise start a new run.
                let rates = value.rates();
                match points.last_mut() {
                    Some(existing) if last_rates == Some(rates) && existing.end == point.start => {
                        existing.end = point.end;
                        existing.add_values_from(&point);
                        if let Some(run_start) = &run_start {
                            existing.period_label = range_label(run_start, &value.end);
                        }
                    },
                    _ => {
                        let mut point = point;
                        point.tick_label = if single_day {
                            Some(value.start.format(&time_format).unwrap())
                        }
                        else {
                            (last_run_date != Some(date)).then(|| date.format(&day_format).unwrap())
                        };
                        run_start = Some(value.start);
                        last_run_date = Some(date);
                        points.push(point);
                    },
                }
                last_rates = Some(rates);
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
            StackedBarGraph { points, mode, per_hour: period == StackedBarPeriod::Rate }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty, Eq, PartialEq, Clone)]
pub struct HomeAndEvReadingConsumption {
    units: Decimal,
    cost:  Decimal,
    rate: Decimal,
    ccy: sparko_graphql::types::Currency,
}

impl HomeAndEvReadingConsumption {
    fn from(stat: meter::home_and_ev_consumption_breakdown::StatisticOutput) -> Self {


        let ccy = if let Some(ex_cost) = &stat.cost_excl_tax_ {
            ex_cost.cost_currency_
        }
        else {
            if let Some(inc_cost) = &stat.cost_incl_tax_ {
                inc_cost.cost_currency_
            }
            else {
                sparko_graphql::types::Currency::NONE
            }
        };

        let cost;
        let rate;

        if let Some(estimated_money) = stat.cost_incl_tax_ {
            cost = estimated_money.estimated_amount_;
            if let Some(price_per_unit) = estimated_money.price_per_unit_ {
                rate = price_per_unit.amount_;
            }
            else {
                rate = Decimal::ZERO;
            }
        }
        else {
            cost = Decimal::ZERO;
            rate = Decimal::ZERO;
        }

        let units = if let Some(units) = stat.value_ {
            units
        }
        else {
            Decimal::ZERO
        };

        HomeAndEvReadingConsumption {
            units,
            cost,
            rate,
            ccy,
        }
    }

    fn from_tou(stat: meter::home_and_ev_consumption_breakdown::StatisticOutput) -> anyhow::Result<Self> {
        let ccy = if let Some(ex_cost) = &stat.cost_excl_tax_ {
            ex_cost.cost_currency_
        }
        else {
            if let Some(inc_cost) = &stat.cost_incl_tax_ {
                inc_cost.cost_currency_
            }
            else {
                sparko_graphql::types::Currency::NONE
            }
        };

        let excl = stat.cost_excl_tax_.as_ref()
            .ok_or_else(|| anyhow!("TouBucketCost with no costExclTax"))?;
        let price_excl = excl.price_per_unit_.as_ref()
            .ok_or_else(|| anyhow!("TouBucketCost with no pricePerUnit"))?
            .amount_;
        let cost_excl = excl.estimated_amount_;
        let cost_incl = stat.cost_incl_tax_.as_ref()
            .map(|c| c.estimated_amount_)
            .unwrap_or(cost_excl);

        let units = if price_excl.is_zero() {
            Decimal::ZERO
        } else {
            (cost_excl / price_excl).normalize()
        };

        // Express the rate incl. tax, to match your cost column and the new format.
        // A zero-cost slot gives no tax ratio, so fall back to the excl price.
        // Units are 0 there, so it only affects the rate shown.
        let rate = if cost_excl.is_zero() {
            price_excl
        } else {
            (price_excl * cost_incl / cost_excl).round_dp(4)
        };

        Ok(HomeAndEvReadingConsumption { units, cost: cost_incl, rate, ccy})
    }

    fn zero() -> Self {
        let cost = Decimal::ZERO;
        let rate = Decimal::ZERO;
        let units = Decimal::ZERO;

        HomeAndEvReadingConsumption {
            units,
            cost,
            rate,
            ccy: Currency::NONE,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty, Eq, PartialEq, Clone)]
pub struct HomeAndEvReading {
    start:  DateTime,
    end: DateTime,
    is_final: bool,
    home_standard: HomeAndEvReadingConsumption,
    ev_standard: HomeAndEvReadingConsumption,
    home_low: HomeAndEvReadingConsumption,
    ev_low: HomeAndEvReadingConsumption,
    standing_charge: Decimal,
    unit: String,
}

impl HomeAndEvReading {
    const FINAL_DELAY: Duration = Duration::from_hours(3 * 24);

    /// The unit rates of the four consumption types, readings with the same rates are grouped by StackedBarPeriod::Rate.
    fn rates(&self) -> [Decimal; 4] {
        [self.home_standard.rate, self.home_low.rate, self.ev_standard.rate, self.ev_low.rate]
    }

    /// The currency of the costs, a consumption with no cost (e.g. zero()) has no currency so the first real one is used.
    fn currency(&self) -> Currency {
        [&self.home_standard, &self.home_low, &self.ev_standard, &self.ev_low]
            .into_iter()
            .map(|consumption| consumption.ccy)
            .find(|ccy| *ccy != Currency::NONE)
            .unwrap_or(Currency::NONE)
    }

    fn summary_headers() -> Element {
        rsx! {
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
        }
    }

    
    fn summary_line(&self, row_class: &str, start_date: &str, start_time: &str, end_time: &str) -> Element {
        rsx!{
            tr { class: row_class,
                td { "{start_date}" }
                td { "{start_time}" }
                td { "{end_time}" }
                td { "{self.home_standard.units}" }
                td { "{self.home_standard.cost}" }
                td { "{self.home_standard.rate}" }
                td { "{self.home_low.units}" }
                td { "{self.home_low.cost}" }
                td { "{self.home_low.rate}" }
                td { "{self.ev_standard.units}" }
                td { "{self.ev_standard.cost}" }
                td { "{self.ev_standard.rate}" }
                td { "{self.ev_low.units}" }
                td { "{self.ev_low.cost}" }
                td { "{self.ev_low.rate}" }
                td { "{self.standing_charge}" }
            }
        }
    }
    fn from(node: meter::home_and_ev_consumption_breakdown::MeasurementInterface, query_time: &DateTime) -> anyhow::Result<Self> {
        match node {
            meter::home_and_ev_consumption_breakdown::MeasurementInterface::MeasurementType(abstract_measurement_interface) => {

                Err(anyhow!(format!("Unexpected abstract measurement {:?}", abstract_measurement_interface)))
            },
            meter::home_and_ev_consumption_breakdown::MeasurementInterface::IntervalMeasurementType(interval_measurement_type) => {
                if let Some(meta_data) = interval_measurement_type.measurement_interface_.meta_data_ {
                    let mut home_standard= None;
                    let mut ev_standard= None;
                    let mut home_low= None;
                    let mut ev_low= None;
                    let mut standing_charge = None;
                    let unit = interval_measurement_type.measurement_interface_.unit_;

                    for stat in meta_data.statistics_ {
                        if let Some(type_) = &stat.type_ {
                            match type_.clone() {
                                super::graphql::ReadingStatisticTypeEnum::StandingChargeCost => {
                                    if let Some(v) = stat.cost_incl_tax_ {
                                        standing_charge = Some(v.estimated_amount_)
                                    }
                                    else {
                                        return Err(anyhow!("Null standing charge"));
                                    }
                                },
                                super::graphql::ReadingStatisticTypeEnum::ConsumptionCost => {
                                    if let Some(label) = stat.label_.clone() {
                                        match label.as_str() {
                                            "EV Low Rate" => {
                                                ev_low = Some(HomeAndEvReadingConsumption::from(stat));
                                            },
                                            "EV Standard Rate" => {
                                                ev_standard = Some(HomeAndEvReadingConsumption::from(stat));
                                            },
                                            "Home Low Rate" => {
                                                home_low= Some(HomeAndEvReadingConsumption::from(stat));
                                            },
                                            "Home Standard Rate" => {
                                                home_standard= Some(HomeAndEvReadingConsumption::from(stat));
                                            },
                                            _ => {

                                                return Err(anyhow!(format!("Unknown meta data label {}", label)));
                                            }
                                        }
                                    }
                                    else {

                                        return Err(anyhow!("No meta data label"));
                                    }
                                },
                                super::graphql::ReadingStatisticTypeEnum::TouBucketCost => {

                                let label = stat.label_.clone()
                                    .ok_or_else(|| anyhow!("No meta data label"))?;
                                let consumption = HomeAndEvReadingConsumption::from_tou(stat)?;

                                // Old format: one whole-house line per slot, no EV split, so it all goes to Home.
                                let slot = match label.as_str() {
                                    "OFF_PEAK" => &mut home_low,
                                    "STANDARD_RATE" => &mut home_standard,
                                    _ => {

                                        return Err(anyhow!("Unknown TOU bucket label {}", label));
                                    }
                                };
                                if slot.is_some() {
                                    return Err(anyhow!("Duplicate {} bucket in slot {}", label, interval_measurement_type.start_at_));
                                }
                                *slot = Some(consumption);
                                },
                                _ => {

                                    return Err(anyhow!(format!("Unexpected statistic type {:?}", type_)));
                                },
                            }
                        }
                        else {
                            return Err(anyhow!("No meta type"));
                        }
                    }

                    let home_standard = if let Some(v) = home_standard { v } else { HomeAndEvReadingConsumption::zero()};
                    let home_low = if let Some(v) = home_low { v } else { HomeAndEvReadingConsumption::zero()};
                    let ev_standard = if let Some(v) = ev_standard { v } else { HomeAndEvReadingConsumption::zero()};
                    let ev_low = if let Some(v) = ev_low { v } else { HomeAndEvReadingConsumption::zero()};
                    let standing_charge = if let Some(v) = standing_charge { v } else { Decimal::ZERO };

                    let is_final = **query_time - *interval_measurement_type.end_at_ > HomeAndEvReading::FINAL_DELAY;
                    Ok(HomeAndEvReading{
                        start: interval_measurement_type.start_at_,
                        end: interval_measurement_type.end_at_,
                        is_final,
                        home_standard,
                        home_low,
                        ev_standard,
                        ev_low,
                        standing_charge,
                        unit
                    })
                }
                else {
                    Err(anyhow!(format!("No meta data")))
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
    /// Count of rows fetched from the API so far
    rows_loaded: Cell<usize>,
    /// Called with the updated row count as each row is fetched, for progress display
    on_progress: Box<dyn Fn(usize)>,
}

impl HomeAndEvConsumptionSource {
    fn query_builder(&self) -> anyhow::Result<meter::home_and_ev_consumption_breakdown::QueryBuilder> {
        let mut utility_filters = Vec::new();

        utility_filters.push(UtilityFiltersInput::builder()
            .with_electricity_filters(ElectricityFiltersInput::builder()
                .with_reading_frequency_type(ReadingFrequencyType::RawInterval)
                .with_reading_direction(super::graphql::ReadingDirectionType::Consumption)
                .with_market_supply_point_id(self.mpan.clone())
                .build()?)
            .build()?
        );

        // modelled_utility_filters.push(UtilityFiltersInput::builder()
        //     .with_electricity_filters(ElectricityFiltersInput::builder()
        //         .with_reading_frequency_type(ReadingFrequencyType::RawInterval)
        //         .with_reading_direction(super::graphql::ReadingDirectionType::Consumption)
        //         .with_market_supply_point_id(self.mpan.clone())
        //         .with_modelled_sub_meter_readings(true)
        //         .build()?)
        //     .build()?
        // );

        Ok(meter::home_and_ev_consumption_breakdown::Query::builder()
            .with_account_number(self.account_number.clone())
            .with_end_at(self.end_at)
            .with_timezone(self.tz.name().to_string())
            .with_utility_filters(utility_filters)
            // .with_modelled_utility_filters(modelled_utility_filters)
            .with_first(1500)
        )
    }

    fn get_measurements(last_response: &meter::home_and_ev_consumption_breakdown::Response) -> anyhow::Result<&ForwardPageOf<meter::home_and_ev_consumption_breakdown::MeasurementInterface>> {
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

    fn initial_query(&mut self, start: &DateTime) -> anyhow::Result<Self::Query> {
        Ok(self.query_builder()?
            .with_start_at(start.clone())
            .build()?
        )
    }

    fn continuation_query(&mut self, start: &DateTime, last_response: &Self::Response) -> anyhow::Result<Option<Self::Query>> {


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

    fn items(&self, response: Self::Response) -> anyhow::Result<impl IntoIterator<Item = anyhow::Result<Self::Value>>> {
        // let measurements = response
        //         .account_
        //         .ok_or(anyhow!("No account in response"))?
        //         .properties_.first().ok_or(anyhow!("No property in response"))?
        //         // .electricity_meter_points_.first().ok_or("No meter points in response")?
        //         .measurements_;
        let query_time = DateTime::now(self.tz);
        Ok(response
            .account_
            .ok_or(anyhow!("No account in response"))?
            .properties_.into_iter().next().ok_or(anyhow!("No property in response"))?
            // .electricity_meter_points_.first().ok_or("No meter points in response")?
            .measurements_
            .edges
            .into_iter()
            .map(move |edge| {
                self.rows_loaded.set(self.rows_loaded.get() + 1);
                (self.on_progress)(self.rows_loaded.get());
                HomeAndEvReading::from(edge.node, &query_time)
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
        Ok((value.start.clone(), value.end.clone(), value.is_final))
    }
    
    fn refresh_after(&self) -> Duration {
        Duration::from_hours(24)
    }
}