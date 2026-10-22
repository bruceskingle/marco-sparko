use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use dioxus::prelude::*;
use display_json::DisplayAsJsonPretty;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use sparko_graphql::types::{Currency, Date, DateTime, ForwardPageOf};
use sparko_graphql::{GraphQLCaller, GraphQLQueryBuilder, ProgressMonitor};
use time_tz::{OffsetDateTimeExt, TimeZone, Tz};

use crate::CacheConfig;
use crate::components::export::{ExportTable, ExportValue};
use crate::data_set::{CancelToken, MonthlyFixedSlotDataSet, SlotSource};
use crate::octopus::decimal::Decimal;
use crate::octopus::graphql::{ElectricityFiltersInput, ReadingFrequencyType, UtilityFiltersInput};

use super::graphql::meter;
use super::category_filter::{CategoryFilter, CategoryValues, KNOWN_LABELS, STANDING_CHARGE_LABEL};
use super::stacked_bar_graph::{StackedBarGraph, StackedBarMode, StackedBarPeriod, StackedBarPoint, StackedBarSeries};
use super::trend_graph::{TrendGraph, TrendLegendItem, TrendLine, YEAR_DASHES, YEAR_LEGEND_COLOR};

/// The readings in a period of a graph added together, e.g. a day.
struct AggregatedPoint {
    start: i64,
    end: i64,
    tick_label: Option<String>,
    period_label: String,
    /// The units of each label, in the order of the labels.
    units: Vec<f64>,
    /// The cost of each label, in the order of the labels.
    costs: Vec<f64>,
    standing_charge: f64,
}

impl AggregatedPoint {
    /// Add the values of another point with the same labels.
    fn add_values_from(&mut self, other: &Self) {
        for (total, value) in self.units.iter_mut().zip(&other.units) {
            *total += value;
        }
        for (total, value) in self.costs.iter_mut().zip(&other.costs) {
            *total += value;
        }
        self.standing_charge += other.standing_charge;
    }
}

/// The consumption labels used by any of the readings, see order_labels.
fn ordered_labels<'a>(readings: impl IntoIterator<Item = &'a MeasurementsReading>) -> Vec<String> {
    order_labels(readings.into_iter().flat_map(|reading| reading.consumption.keys().map(String::as_str)))
}

/// The distinct labels, the known ones first in their usual order, then any others in alphabetical order.
fn order_labels<'a>(labels: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let all: BTreeSet<&str> = labels.into_iter().collect();
    let is_known = |label: &str| KNOWN_LABELS.iter().any(|(known, _)| *known == label);

    KNOWN_LABELS.iter()
        .map(|(label, _)| *label)
        .filter(|label| all.contains(label))
        .chain(all.iter().copied().filter(|label| !is_known(label)))
        .map(String::from)
        .collect()
}

/// Measurement readings for a meter over a period, cached in monthly buckets.
pub struct MeasurementsDataSet {
    pub data: Vec<MonthlyFixedSlotDataSet<MeasurementsConsumptionSource>>,
}

impl MeasurementsDataSet {
    pub async fn new(
        account_number: &String,
        mpan: &String,
        start: &DateTime,
        end: &DateTime,
        tz: &'static Tz,
        progress: &(impl ProgressMonitor + ?Sized),
        cancel: &CancelToken,
        cache_config: &Arc<CacheConfig>,
        request_manager: &impl GraphQLCaller)
        -> anyhow::Result<Self>
    {

        let source = MeasurementsConsumptionSource{
            account_number: account_number.clone(),
            mpan: mpan.clone(),
            hash_key: format!("{}#{}#MeasurementsConsumption", account_number, mpan),
            end_at: end.clone(),
            tz,
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

        let rows: Vec<_> = self.data.iter()
            .flat_map(|data_set| data_set.data.range(start..))
            .filter(|(key, _)| *key < end)
            .collect();

        // A group of columns for each label used in the period.
        let labels = ordered_labels(rows.iter().map(|(_, item)| &item.value));

        let mut items = Vec::new();
        let mut last_date = Date::from_calendar_date(0, time::Month::January, 1)?;
        for (key, item) in rows {
            let row_class = if item.attributes.cached { "cached"} else {"fetched"};
            let item_start_date = item.value.start.to_date();
            let (start_date, start_time) = if last_date == item_start_date {
                ("".to_string(), item.value.start.format(&time_format).unwrap())
            } else {
                (item.value.start.format(&date_format).unwrap(), item.value.start.format(&time_format).unwrap())
            };

            let end_time = item.value.end.format(&time_format).unwrap();

            items.push(item.value.summary_line(&labels, row_class, &start_date, &start_time, &end_time));
            last_date = key.to_date();
        }

        rsx! {
            table {
                {MeasurementsReading::summary_headers(&labels)}
                for item in items {
                    {item}
                }
            }
        }
    }

    /// The categories which can be shown in the graphs for the readings from start to end, the standing charge (for
    /// costs only) and then each label.
    pub fn categories(&self, start: &DateTime, end: &DateTime, mode: StackedBarMode) -> Vec<String> {
        let labels = ordered_labels(self.readings(start, end));
        Self::categories_for(&labels, mode)
    }

    fn categories_for(labels: &[String], mode: StackedBarMode) -> Vec<String> {
        let standing_charge = (mode == StackedBarMode::Cost).then(|| String::from(STANDING_CHARGE_LABEL));
        standing_charge.into_iter().chain(labels.iter().cloned()).collect()
    }

    /// The readings from start to end.
    fn readings<'a>(&'a self, start: &'a DateTime, end: &'a DateTime) -> impl Iterator<Item = &'a MeasurementsReading> + 'a {
        self.data.iter()
            .flat_map(move |data_set| data_set.data.range(start..))
            .filter(move |(key, _)| *key < end)
            .map(|(_, item)| &item.value)
    }

    /// The value of each category for each point, filtered by the filter.
    fn filtered_categories(labels: &[String], points: &[AggregatedPoint], mode: StackedBarMode, filter: &CategoryFilter) -> Vec<CategoryValues> {
        let categories = Self::categories_for(labels, mode).into_iter()
            .enumerate()
            .map(|(index, label)| {
                // In Cost mode the standing charge comes first, before the labels.
                let label_index = if mode == StackedBarMode::Cost { index.checked_sub(1) } else { Some(index) };
                let values = points.iter()
                    .map(|point| match (mode, label_index) {
                        (StackedBarMode::Cost, None) => point.standing_charge,
                        (StackedBarMode::Cost, Some(index)) => point.costs[index],
                        (StackedBarMode::Units, Some(index)) => point.units[index],
                        (StackedBarMode::Units, None) => 0.0,
                    })
                    .collect();
                (label, values)
            })
            .collect();
        filter.apply(categories, points.len())
    }

    /// The unit of consumption and the currency of the readings, e.g. kwh and GBP.
    fn graph_units(&self, start: &DateTime, end: &DateTime) -> (String, Currency) {
        let readings: Vec<&MeasurementsReading> = self.readings(start, end).collect();
        let unit = readings.iter()
            .map(|reading| reading.unit.as_str())
            .find(|unit| !unit.is_empty())
            .unwrap_or_default()
            .to_string();
        let currency = readings.iter()
            .map(|reading| reading.currency())
            .find(|currency| *currency != Currency::NONE)
            .unwrap_or(Currency::NONE);
        (unit, currency)
    }

    pub fn display_graph(&self, start: &DateTime, end: &DateTime, tz: &'static Tz, mode: StackedBarMode, period: StackedBarPeriod, filter: &CategoryFilter) -> Element {
        let (labels, aggregated) = self.graph_points(start, end, tz, period);
        let categories = Self::filtered_categories(&labels, &aggregated, mode, filter);
        let (unit, currency) = self.graph_units(start, end);

        let series = categories.iter()
            .map(|category| StackedBarSeries { label: category.label.clone(), color: category.color.clone() })
            .collect();
        let points = aggregated.into_iter()
            .enumerate()
            .map(|(index, point)| StackedBarPoint {
                start: point.start,
                end: point.end,
                tick_label: point.tick_label,
                period_label: point.period_label,
                values: categories.iter().map(|category| category.values[index]).collect(),
            })
            .collect();

        rsx! {
            StackedBarGraph {
                points,
                series,
                mode,
                currency,
                unit,
                per_hour: period == StackedBarPeriod::Rate,
            }
        }
    }

    /// A line graph over one year, with a line for each category (or the total) in each year from start to end.
    pub fn display_trend(&self, start: &DateTime, end: &DateTime, tz: &'static Tz, mode: StackedBarMode, period: StackedBarPeriod, filter: &CategoryFilter) -> Element {
        let (labels, aggregated) = self.graph_points(start, end, tz, period);
        let categories = Self::filtered_categories(&labels, &aggregated, mode, filter);
        let (unit, currency) = self.graph_units(start, end);
        let per_hour = period == StackedBarPeriod::Rate;

        // The year of each point and the position of its middle in that year.
        let mut positions = Vec::with_capacity(aggregated.len());
        for point in &aggregated {
            let middle = point.start + (point.end - point.start) / 2;
            let year = time::OffsetDateTime::from_unix_timestamp(middle)
                .map_err(|error| anyhow!(error))?
                .to_timezone(tz)
                .year();
            let year_start = Date::from_calendar_date(year, time::Month::January, 1)?.at_midnight(tz).unix_timestamp();
            let next_year_start = Date::from_calendar_date(year + 1, time::Month::January, 1)?.at_midnight(tz).unix_timestamp();
            positions.push((year, (middle - year_start) as f64 / (next_year_start - year_start) as f64));
        }
        let years: BTreeSet<i32> = positions.iter().map(|(year, _)| *year).collect();

        // Each year has its own dash pattern, in order of year.
        let mut lines = Vec::new();
        for (year_index, year) in years.iter().enumerate() {
            let dash = YEAR_DASHES[year_index % YEAR_DASHES.len()];
            for category in &categories {
                let points: Vec<(f64, f64)> = aggregated.iter()
                    .zip(&positions)
                    .zip(&category.values)
                    .filter(|((_, (point_year, _)), _)| point_year == year)
                    .map(|((point, (_, position)), value)| {
                        // Rate periods have different lengths, so their values are per hour.
                        let scale = if per_hour { 3600.0 / (point.end - point.start).max(1) as f64 } else { 1.0 };
                        (*position, value * scale)
                    })
                    .collect();
                lines.push(TrendLine { color: category.color.clone(), dash: String::from(dash), points });
            }
        }

        let legend = categories.iter()
            .map(|category| TrendLegendItem { label: category.label.clone(), color: category.color.clone(), dash: String::from("none") })
            .chain(years.iter().enumerate().map(|(year_index, year)| TrendLegendItem {
                label: year.to_string(),
                color: String::from(YEAR_LEGEND_COLOR),
                dash: String::from(YEAR_DASHES[year_index % YEAR_DASHES.len()]),
            }))
            .collect();

        rsx! {
            TrendGraph {
                lines,
                legend,
                mode,
                currency,
                unit,
                per_hour,
            }
        }
    }

    /// The readings from start to end aggregated by period, with the labels in the order of their values.
    fn graph_points(&self, start: &DateTime, end: &DateTime, tz: &'static Tz, period: StackedBarPeriod) -> (Vec<String>, Vec<AggregatedPoint>) {
        let date_format = time::format_description::parse("[year]-[month]-[day]").unwrap();
        let day_format = time::format_description::parse("[day] [month repr:short]").unwrap();
        let time_format = time::format_description::parse("[hour]:[minute]").unwrap();
        let month_format = time::format_description::parse("[month repr:short] [year]").unwrap();

        let values: Vec<_> = self.data.iter()
            .flat_map(|data_set| data_set.data.range(start..))
            .filter(|(key, _)| *key < end)
            .map(|(_, item)| &item.value)
            .collect();

        // The values of each label used in the period.
        let labels = ordered_labels(values.iter().copied());

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
            let point = AggregatedPoint {
                start: bucket_start.unix_timestamp(),
                end: bucket_end.unix_timestamp(),
                tick_label,
                period_label: group_label,
                units: labels.iter()
                    .map(|label| value.consumption.get(label).and_then(|c| c.units.to_f64()).unwrap_or_default())
                    .collect(),
                costs: labels.iter()
                    .map(|label| value.consumption.get(label).and_then(|c| c.cost.to_f64()).unwrap_or_default())
                    .collect(),
                standing_charge: value.standing_charge.to_f64().unwrap_or_default(),
            };

            if matches!(period, StackedBarPeriod::Raw) {
                points.push(point);
            } else if period == StackedBarPeriod::Rate {
                // Extend the last run if this reading follows on from it with the same rates, otherwise start a new run.
                let rates = value.rates();
                match points.last_mut() {
                    Some(existing) if last_rates.as_ref() == Some(&rates) && existing.end == point.start => {
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

        (labels, points)
    }

    /// The readings from start to end, as shown in the table.
    pub fn readings_export(&self, start: &DateTime, end: &DateTime) -> ExportTable {
        let readings: Vec<&MeasurementsReading> = self.data.iter()
            .flat_map(|data_set| data_set.data.range(start..))
            .filter(|(key, _)| *key < end)
            .map(|(_, item)| &item.value)
            .collect();
        let labels = ordered_labels(readings.iter().copied());
        let (unit, cost_unit) = Self::export_units(&readings);

        // All the costs, then all the units, then the rates.
        let mut columns = vec![String::from("Start"), String::from("End")];
        columns.extend(labels.iter().map(|label| format!("{label} cost ({cost_unit})")));
        columns.push(format!("Standing charge ({cost_unit})"));
        columns.extend(labels.iter().map(|label| format!("{label} units ({unit})")));
        columns.extend(labels.iter().map(|label| format!("{label} rate ({cost_unit}/{unit})")));

        let decimal = |value: &Decimal| value.to_f64().map(ExportValue::number).unwrap_or(ExportValue::Empty);
        let rows = readings.iter()
            .map(|reading| {
                // The value of a field of each label's consumption, empty if the reading does not have that label.
                let field = |get: fn(&MeasurementsReadingConsumption) -> &Decimal| labels.iter()
                    .map(move |label| reading.consumption.get(label).map(|c| decimal(get(c))).unwrap_or(ExportValue::Empty));

                let mut row = vec![ExportValue::time(&reading.start), ExportValue::time(&reading.end)];
                row.extend(field(|c| &c.cost));
                row.push(decimal(&reading.standing_charge));
                row.extend(field(|c| &c.units));
                row.extend(field(|c| &c.rate));
                row
            })
            .collect();

        ExportTable { columns, rows }
    }

    /// The readings from start to end aggregated by period, as shown in the graph.
    pub fn graph_export(&self, start: &DateTime, end: &DateTime, tz: &'static Tz, period: StackedBarPeriod) -> ExportTable {
        let readings: Vec<&MeasurementsReading> = self.data.iter()
            .flat_map(|data_set| data_set.data.range(start..))
            .filter(|(key, _)| *key < end)
            .map(|(_, item)| &item.value)
            .collect();
        let (unit, cost_unit) = Self::export_units(&readings);
        let (labels, points) = self.graph_points(start, end, tz, period);

        // All the costs, then all the units.
        let mut columns = vec![String::from("Start"), String::from("End")];
        columns.extend(labels.iter().map(|label| format!("{label} cost ({cost_unit})")));
        columns.push(format!("Standing charge ({cost_unit})"));
        columns.extend(labels.iter().map(|label| format!("{label} units ({unit})")));

        let rows = points.iter()
            .map(|point| {
                let mut row = vec![ExportValue::unix_time(point.start, tz), ExportValue::unix_time(point.end, tz)];
                row.extend(point.costs.iter().map(|cost| ExportValue::number(*cost)));
                row.push(ExportValue::number(point.standing_charge));
                row.extend(point.units.iter().map(|units| ExportValue::number(*units)));
                row
            })
            .collect();

        ExportTable { columns, rows }
    }

    /// The units of consumption and of cost for column headings, e.g. ("kwh", "p").
    fn export_units(readings: &[&MeasurementsReading]) -> (String, String) {
        let unit = readings.iter()
            .map(|reading| reading.unit.as_str())
            .find(|unit| !unit.is_empty())
            .unwrap_or_default()
            .to_string();
        let currency = readings.iter()
            .map(|reading| reading.currency())
            .find(|currency| *currency != Currency::NONE)
            .unwrap_or(Currency::NONE);
        // Costs are in the minor units of the currency.
        let cost_unit = currency.subunit_symbol().unwrap_or_else(|| currency.code().to_string());
        (unit, cost_unit)
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty, Eq, PartialEq, Clone)]
pub struct MeasurementsReadingConsumption {
    units: Decimal,
    cost:  Decimal,
    rate: Decimal,
    ccy: sparko_graphql::types::Currency,
}

impl MeasurementsReadingConsumption {
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

        MeasurementsReadingConsumption {
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

        Ok(MeasurementsReadingConsumption { units, cost: cost_incl, rate, ccy})
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty, Eq, PartialEq, Clone)]
pub struct MeasurementsReading {
    start:  DateTime,
    end: DateTime,
    is_final: bool,
    /// The consumption at each rate, indexed by the label from the API. A BTreeMap so that the serialized form, which
    /// is compared to detect changes, is always in the same order.
    consumption: BTreeMap<String, MeasurementsReadingConsumption>,
    standing_charge: Decimal,
    unit: String,
}

impl MeasurementsReading {
    const FINAL_DELAY: Duration = Duration::from_hours(3 * 24);

    /// The unit rate for each label, readings with the same rates are grouped by StackedBarPeriod::Rate.
    fn rates(&self) -> BTreeMap<String, Decimal> {
        self.consumption.iter()
            .map(|(label, consumption)| (label.clone(), consumption.rate))
            .collect()
    }

    /// The currency of the costs, a consumption with no cost has no currency so the first real one is used.
    fn currency(&self) -> Currency {
        self.consumption.values()
            .map(|consumption| consumption.ccy)
            .find(|ccy| *ccy != Currency::NONE)
            .unwrap_or(Currency::NONE)
    }

    /// Add the consumption for a label, which must appear only once in a reading.
    fn insert_consumption(consumption: &mut BTreeMap<String, MeasurementsReadingConsumption>, label: String,
        value: MeasurementsReadingConsumption, start: &DateTime) -> anyhow::Result<()> {
        if consumption.contains_key(&label) {
            return Err(anyhow!("Duplicate {} consumption in slot {}", label, start));
        }
        consumption.insert(label, value);
        Ok(())
    }

    /// Table headers with a group of columns for each label.
    fn summary_headers(labels: &[String]) -> Element {
        rsx! {
            tr {
                td { colspan: 3 }
                for label in labels {
                    th { colspan: 3, class: "header", "{label}" }
                }
            }
            tr { class: "header",
                th { colspan: 2, "From" }
                th { "To" }
                for _ in labels {
                    th { "Units" }
                    th { "Cost" }
                    th { "Rate" }
                }
                th { "Standing Charge" }
            }
        }
    }

    /// A table row matching summary_headers, labels which this reading does not have are left empty.
    fn summary_line(&self, labels: &[String], row_class: &str, start_date: &str, start_time: &str, end_time: &str) -> Element {
        rsx!{
            tr { class: row_class,
                td { "{start_date}" }
                td { "{start_time}" }
                td { "{end_time}" }
                for label in labels {
                    if let Some(consumption) = self.consumption.get(label) {
                        td { "{consumption.units}" }
                        td { "{consumption.cost}" }
                        td { "{consumption.rate}" }
                    }
                    else {
                        td {}
                        td {}
                        td {}
                    }
                }
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
                    let mut consumption = BTreeMap::new();
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
                                // Consumption at one rate, labelled with the rate, e.g. "EV Low Rate".
                                super::graphql::ReadingStatisticTypeEnum::ConsumptionCost => {
                                    let label = stat.label_.clone()
                                        .ok_or_else(|| anyhow!("No meta data label"))?;
                                    Self::insert_consumption(&mut consumption, label, MeasurementsReadingConsumption::from(stat), &interval_measurement_type.start_at_)?;
                                },
                                // The old format, whole house consumption in time of use buckets, e.g. "OFF_PEAK".
                                super::graphql::ReadingStatisticTypeEnum::TouBucketCost => {
                                    let label = stat.label_.clone()
                                        .ok_or_else(|| anyhow!("No meta data label"))?;
                                    Self::insert_consumption(&mut consumption, label, MeasurementsReadingConsumption::from_tou(stat)?, &interval_measurement_type.start_at_)?;
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

                    let standing_charge = if let Some(v) = standing_charge { v } else { Decimal::ZERO };

                    let is_final = **query_time - *interval_measurement_type.end_at_ > MeasurementsReading::FINAL_DELAY;
                    Ok(MeasurementsReading{
                        start: interval_measurement_type.start_at_,
                        end: interval_measurement_type.end_at_,
                        is_final,
                        consumption,
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

pub struct MeasurementsConsumptionSource {
    pub account_number: String,
    pub end_at: DateTime,
    pub tz: &'static Tz,
    pub mpan:String,
    pub hash_key: String,
}

impl MeasurementsConsumptionSource {
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

impl SlotSource for MeasurementsConsumptionSource {
    type Value = MeasurementsReading;

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
                MeasurementsReading::from(edge.node, &query_time)
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

#[cfg(test)]
mod tests {
    use super::{AggregatedPoint, order_labels};

    #[test]
    fn known_labels_come_first_in_their_usual_order_then_others_alphabetically() {
        let labels = order_labels(["STANDARD_RATE", "EV Standard Rate", "OFF_PEAK", "Home Low Rate", "EV Standard Rate"]);
        assert_eq!(labels, ["Home Low Rate", "EV Standard Rate", "OFF_PEAK", "STANDARD_RATE"]);
    }

    #[test]
    fn adding_points_sums_every_value() {
        let point = || AggregatedPoint {
            start: 0,
            end: 1,
            tick_label: None,
            period_label: String::new(),
            units: vec![1.0, 2.0],
            costs: vec![5.0, 6.0],
            standing_charge: 9.0,
        };
        let mut total = point();
        total.add_values_from(&point());
        assert_eq!(total.units, [2.0, 4.0]);
        assert_eq!(total.costs, [10.0, 12.0]);
        assert_eq!(total.standing_charge, 18.0);
    }
}
