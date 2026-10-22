use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use dioxus::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use sparko_graphql::types::{Currency, Date, DateTime};
use sparko_graphql::{GraphQLCaller, GraphQLQueryBuilder, ProgressMonitor};
use time_tz::Tz;

use crate::CacheConfig;
use crate::components::export::{ExportTable, ExportValue};
use crate::data_set::{CancelToken, MonthlyFixedSlotDataSet, SlotSource};

use super::consumption_graph::{ConsumptionGraph, GraphPoint, GraphStyle};
use super::graphql::meter;
use super::stacked_bar_graph::{StackedBarMode, StackedBarPeriod};
use super::trend_graph::{TrendGraph, TrendLegendItem, TrendLine, year_dash, year_positions};

/// The colour of the lines in the trend graph, the same as the accent colour used for the bars.
const TREND_COLOR: &str = "#06b6d4";

/// Half hourly consumption readings for a meter over a period, cached in monthly buckets.
pub struct ConsumptionDataSet {
    pub data: Vec<MonthlyFixedSlotDataSet<ConsumptionSource>>,
}

impl ConsumptionDataSet {
    pub async fn new(
        account_number: &String,
        meter_node_id: &String,
        start: &DateTime,
        end: &DateTime,
        tz: &'static Tz,
        progress: &(impl ProgressMonitor + ?Sized),
        cancel: &CancelToken,
        cache_config: &Arc<CacheConfig>,
        request_manager: &impl GraphQLCaller)
        -> anyhow::Result<Self>
    {

        let source = ConsumptionSource{
            meter_node_id: meter_node_id.clone(),
            hash_key: format!("{}#{}#ConsumptionRecords", account_number, meter_node_id),
        };
        
        let data = MonthlyFixedSlotDataSet::fetch(
            source,
            tz,
            cache_config,
            request_manager,
            start,
            end,
            cancel,
            progress,
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

    pub fn display_graph(&self, start: &DateTime, end: &DateTime, tz: &'static Tz, period: StackedBarPeriod) -> Element {
        let points = self.graph_points(start, end, tz, period);

        rsx! {
            ConsumptionGraph { points, style: GraphStyle::Bar }
        }
    }

    /// A line graph over one year, with a line for each year from start to end.
    pub fn display_trend(&self, start: &DateTime, end: &DateTime, tz: &'static Tz, period: StackedBarPeriod) -> Element {
        let points = self.graph_points(start, end, tz, period);
        let positions = year_positions(points.iter().map(|point| (point.start, point.end)), tz)?;
        let years: BTreeSet<i32> = positions.iter().map(|(year, _)| *year).collect();

        // Each year has its own dash pattern, in order of year.
        let lines = years.iter()
            .enumerate()
            .map(|(year_index, year)| TrendLine {
                color: String::from(TREND_COLOR),
                dash: String::from(year_dash(year_index)),
                points: points.iter()
                    .zip(&positions)
                    .filter(|(_, (point_year, _))| point_year == year)
                    .map(|(point, (_, position))| (*position, point.value))
                    .collect(),
            })
            .collect();
        let legend = years.iter()
            .enumerate()
            .map(|(year_index, year)| TrendLegendItem {
                label: year.to_string(),
                color: String::from(TREND_COLOR),
                dash: String::from(year_dash(year_index)),
            })
            .collect();

        rsx! {
            TrendGraph {
                lines,
                legend,
                mode: StackedBarMode::Units,
                currency: Currency::NONE,
                unit: String::new(),
            }
        }
    }

    /// The readings from start to end aggregated by period.
    fn graph_points(&self, start: &DateTime, end: &DateTime, tz: &'static Tz, period: StackedBarPeriod) -> Vec<GraphPoint> {
        let date_format = time::format_description::parse("[year]-[month]-[day]").unwrap();
        let day_format = time::format_description::parse("[day] [month repr:short]").unwrap();
        let time_format = time::format_description::parse("[hour]:[minute]").unwrap();
        let month_format = time::format_description::parse("[month repr:short] [year]").unwrap();

        let values: Vec<_> = self.data.iter()
            .flat_map(|data_set| data_set.data.range(start..))
            .filter(|(key, _)| *key < end)
            .map(|(_, item)| &item.value)
            .collect();

        // Plain consumption has no rates, so Rate is the same as Raw.
        let single_day = matches!(period, StackedBarPeriod::Raw | StackedBarPeriod::Rate)
            && values.first().map(|value| value.start_at_.to_date())
                == values.last().map(|value| value.start_at_.to_date());

        let mut points = Vec::with_capacity(values.len());
        let mut last_date = None;
        let mut last_bucket_start = None;
        for value in values {
            let date = value.start_at_.to_date();
            let (bucket_start, bucket_end) = match period {
                StackedBarPeriod::Raw | StackedBarPeriod::Rate => (value.start_at_, value.end_at_),
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
                    .unwrap() - time::Duration::days(days_since_monday);
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
                StackedBarPeriod::Raw | StackedBarPeriod::Rate if single_day => {
                    (value.start_at_.minute() == 0 && value.start_at_.hour() % 3 == 0)
                        .then(|| value.start_at_.format(&time_format).unwrap())
                }
                StackedBarPeriod::Raw | StackedBarPeriod::Rate => {
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

            let period_label = match period {
                StackedBarPeriod::Raw | StackedBarPeriod::Rate => format!(
                    "{} {}–{}",
                    value.start_at_.format(&date_format).unwrap(),
                    value.start_at_.format(&time_format).unwrap(),
                    value.end_at_.format(&time_format).unwrap(),
                ),
                StackedBarPeriod::Day => bucket_start.format(&date_format).unwrap(),
                StackedBarPeriod::Week => format!(
                    "{} – {}",
                    bucket_start.format(&date_format).unwrap(),
                    bucket_end.to_date().previous_day().format(&date_format).unwrap(),
                ),
                StackedBarPeriod::Month => bucket_start.format(&month_format).unwrap(),
            };
            let point = GraphPoint {
                start: bucket_start.unix_timestamp(),
                end: bucket_end.unix_timestamp(),
                value: value.value_.to_f64().unwrap_or_default(),
                tick_label,
                period_label,
            };

            if matches!(period, StackedBarPeriod::Raw | StackedBarPeriod::Rate) {
                points.push(point);
            } else if let Some(existing) = points.last_mut() {
                if existing.start == bucket_start.unix_timestamp() {
                    existing.value += point.value;
                } else {
                    points.push(point);
                }
            } else {
                points.push(point);
            }

            last_date = Some(date);
            last_bucket_start = Some(bucket_start.unix_timestamp());
        }

        points
    }

    /// The readings from start to end, as shown in the table.
    pub fn readings_export(&self, start: &DateTime, end: &DateTime) -> ExportTable {
        let rows = self.data.iter()
            .flat_map(|data_set| data_set.data.range(start..))
            .filter(|(key, _)| *key < end)
            .map(|(_, item)| vec![
                ExportValue::time(&item.value.start_at_),
                ExportValue::time(&item.value.end_at_),
                item.value.value_.to_f64().map(ExportValue::number).unwrap_or(ExportValue::Empty),
            ])
            .collect();

        ExportTable { columns: Self::export_columns(), rows }
    }

    /// The readings from start to end aggregated by period, as shown in the graph.
    pub fn graph_export(&self, start: &DateTime, end: &DateTime, tz: &'static Tz, period: StackedBarPeriod) -> ExportTable {
        let rows = self.graph_points(start, end, tz, period).iter()
            .map(|point| vec![
                ExportValue::unix_time(point.start, tz),
                ExportValue::unix_time(point.end, tz),
                ExportValue::number(point.value),
            ])
            .collect();

        ExportTable { columns: Self::export_columns(), rows }
    }

    fn export_columns() -> Vec<String> {
        vec![String::from("Start"), String::from("End"), String::from("Consumption")]
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

    fn initial_query(&mut self, start: &DateTime) -> anyhow::Result<Self::Query> {
        Ok(meter::meter_consumption::Query::builder()
            .with_meter_id(self.meter_node_id.clone())
            .with_grouping(super::graphql::ConsumptionGroupings::HalfHour)
            .with_start_at(start.clone())
            .with_timezone(String::from("Europe/London"))
            .with_first(50)
            .build()?)
    }

    fn continuation_query(&mut self, start: &DateTime, last_response: &Self::Response) -> anyhow::Result<Option<Self::Query>> {
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

    fn items(&self, response: Self::Response) -> anyhow::Result<impl IntoIterator<Item = anyhow::Result<Self::Value>>> {
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

        Ok(page.edges.into_iter().map(|edge| Ok(edge.node)))
    }

    fn index(&self, value: &Self::Value) -> anyhow::Result<(DateTime, DateTime, bool)> {
       Ok((value.start_at_.clone(), value.end_at_.clone(), true))
    }
    
    fn refresh_after(&self) -> Duration {
        Duration::from_hours(24)
    }
}
