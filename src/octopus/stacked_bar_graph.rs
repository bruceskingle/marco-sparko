use dioxus::prelude::*;
use sparko_graphql::types::Currency;

use super::consumption_graph::nice_step;

const LEFT: f64 = 64.0;
const RIGHT: f64 = 16.0;
const TOP: f64 = 44.0;
const BOTTOM: f64 = 32.0;
const MAX_X_TICKS: usize = 12;
const Y_TICKS: f64 = 4.0;
const BAR_GAP: f64 = 2.0;
const TOOLTIP_WIDTH: f64 = 226.0;
/// The tooltip text starts after a column of colour swatches.
const TOOLTIP_TEXT_X: f64 = 24.0;

#[derive(Clone, Copy, PartialEq)]
pub enum StackedBarMode {
    Units,
    Cost,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StackedBarPeriod {
    Raw,
    /// Consecutive readings with the same unit rates, so the bars have different widths.
    Rate,
    Day,
    Week,
    Month,
}

impl std::fmt::Display for StackedBarMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cost => write!(f, "Cost"),
            Self::Units => write!(f, "Units"),
        }
    }
}

impl std::fmt::Display for StackedBarPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raw => write!(f, "Raw"),
            Self::Rate => write!(f, "Rate"),
            Self::Day => write!(f, "Day"),
            Self::Week => write!(f, "Week"),
            Self::Month => write!(f, "Month"),
        }
    }
}

/// One series stacked in each bar, e.g. the consumption at one rate.
#[derive(Clone, PartialEq)]
pub struct StackedBarSeries {
    pub label: String,
    /// The CSS class giving the colour of the series.
    pub class: String,
}

#[derive(Clone, PartialEq)]
pub struct StackedBarPoint {
    pub start: i64,
    pub end: i64,
    pub tick_label: Option<String>,
    pub period_label: String,
    /// The units of each series, in the order of the graph's series.
    pub units: Vec<f64>,
    /// The cost of each series, in the order of the graph's series.
    pub costs: Vec<f64>,
    pub standing_charge: f64,
    /// The currency of the costs, which are in its minor units, e.g. pence
    pub currency: Currency,
    /// The unit of the consumption values, e.g. kwh
    pub unit: String,
}

impl StackedBarPoint {
    /// The values stacked in the bar, bottom first. In Cost mode the standing charge is at the bottom.
    fn segments(&self, mode: StackedBarMode) -> Vec<f64> {
        match mode {
            StackedBarMode::Units => self.units.clone(),
            StackedBarMode::Cost => std::iter::once(self.standing_charge).chain(self.costs.iter().copied()).collect(),
        }
    }

    /// Add the values of another point with the same series.
    pub fn add_values_from(&mut self, other: &Self) {
        for (total, value) in self.units.iter_mut().zip(&other.units) {
            *total += value;
        }
        for (total, value) in self.costs.iter_mut().zip(&other.costs) {
            *total += value;
        }
        self.standing_charge += other.standing_charge;
        if self.currency == Currency::NONE {
            self.currency = other.currency;
        }
        if self.unit.is_empty() {
            self.unit = other.unit.clone();
        }
    }
}

/// Format an amount in minor units (e.g. pence) in the major units of the currency, e.g. £1.23
fn format_money(minor_units: f64, currency: Currency) -> String {
    let decimals = currency.minor_units() as usize;
    let major = minor_units / 10f64.powi(decimals as i32);
    let sign = if major < 0.0 { "-" } else { "" };
    let symbol = if currency == Currency::NONE { String::new() } else { currency.symbol() };
    format!("{sign}{symbol}{:.decimals$}", major.abs())
}

const STANDING_CHARGE_LABEL: &str = "Standing charge";
const STANDING_CHARGE_CLASS: &str = "standing-charge";

#[component]
pub fn StackedBarGraph(
    points: Vec<StackedBarPoint>,
    /// The series stacked in each bar, bottom first.
    series: Vec<StackedBarSeries>,
    mode: StackedBarMode,
    /// Draw a histogram, where bar heights are per hour so that the area of each bar is its total. Needed when the bars
    /// have different widths.
    #[props(default)] per_hour: bool,
) -> Element {
    let mut hovered = use_signal::<Option<usize>>(|| None);
    let mut dimensions = use_signal(|| None::<(f64, f64)>);

    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return rsx! { p { "No data for this period" } };
    };

    let (width, height) = dimensions().unwrap_or((960.0, 320.0));
    let t_min = first.start as f64;
    let t_span = ((last.end - first.start) as f64).max(1.0);
    let plot_width = (width - LEFT - RIGHT).max(1.0);
    let plot_height = (height - TOP - BOTTOM).max(1.0);
    let x = |t: i64| LEFT + (t as f64 - t_min) / t_span * plot_width;
    // The label and colour class of each segment, matching StackedBarPoint::segments.
    let segment_names: Vec<(String, String)> = match mode {
        StackedBarMode::Units => Vec::new(),
        StackedBarMode::Cost => vec![(String::from(STANDING_CHARGE_LABEL), String::from(STANDING_CHARGE_CLASS))],
    }
    .into_iter()
    .chain(series.iter().map(|series| (series.label.clone(), series.class.clone())))
    .collect();
    // The factor from a bar's values to its height.
    let scale = move |point: &StackedBarPoint| {
        if per_hour { 3600.0 / (point.end - point.start).max(1) as f64 } else { 1.0 }
    };
    let totals: Vec<f64> = points
        .iter()
        .map(|point| point.segments(mode).iter().map(|value| value.max(0.0)).sum())
        .collect();
    let max_value = points
        .iter()
        .zip(&totals)
        .map(|(point, total)| total * scale(point))
        .fold(0.0, f64::max);
    let per_hour_suffix = if per_hour { "/h" } else { "" };
    let currency = points
        .iter()
        .map(|point| point.currency)
        .find(|currency| *currency != Currency::NONE)
        .unwrap_or(Currency::NONE);
    let unit = points
        .iter()
        .map(|point| point.unit.as_str())
        .find(|unit| !unit.is_empty())
        .unwrap_or_default();
    let format_value = move |value: f64| match mode {
        StackedBarMode::Units => format!("{value:.3} {unit}"),
        StackedBarMode::Cost => format_money(value, currency),
    };
    let (step, decimals) = nice_step(max_value / Y_TICKS);
    let y_max = (max_value / step).ceil().max(1.0) * step;
    let y = |value: f64| TOP + plot_height - value / y_max * plot_height;

    let bars: Vec<(usize, String, f64, f64, f64, f64)> = points
        .iter()
        .enumerate()
        .flat_map(|(i, point)| {
            let slot_width = x(point.end) - x(point.start);
            let gap = if slot_width > 3.0 * BAR_GAP { BAR_GAP } else { 0.0 };
            let bar_x = x(point.start) + gap / 2.0;
            let bar_width = (slot_width - gap).max(0.0);
            let bar_scale = scale(point);
            let mut baseline = 0.0;

            point
                .segments(mode)
                .into_iter()
                .zip(segment_names.iter())
                .map(move |(value, (_, class))| {
                    let value = value.max(0.0) * bar_scale;
                    let top = y(baseline + value);
                    let segment = (
                        i,
                        class.clone(),
                        bar_x,
                        top,
                        bar_width,
                        y(baseline) - top,
                    );
                    baseline += value;
                    segment
                })
        })
        .collect();

    let y_ticks: Vec<(f64, String)> = (0..)
        .map(|i| i as f64 * step)
        .take_while(|value| *value <= y_max + step / 2.0)
        .map(|value| {
            let label = match mode {
                StackedBarMode::Units => format!("{value:.decimals$} {unit}{per_hour_suffix}"),
                StackedBarMode::Cost => format!("{}{per_hour_suffix}", format_money(value, currency)),
            };
            (y(value), label)
        })
        .collect();

    let all_x_ticks: Vec<(f64, &String)> = points
        .iter()
        .filter_map(|point| point.tick_label.as_ref().map(|label| (x(point.start), label)))
        .collect();
    let every = all_x_ticks.len().div_ceil(MAX_X_TICKS).max(1);
    let x_ticks: Vec<(f64, String)> = all_x_ticks
        .into_iter()
        .step_by(every)
        .map(|(position, label)| (position, label.clone()))
        .collect();
    let legend: Vec<(f64, String, String)> = segment_names
        .iter()
        .enumerate()
        .map(|(index, (label, color_class))| {
            (LEFT + index as f64 * 145.0, label.clone(), color_class.clone())
        })
        .collect();

    let tooltip = hovered().and_then(|index| {
        points.get(index).map(|point| {
            let bar_x = x(point.start + (point.end - point.start) / 2);
            let tooltip_x = if bar_x > width / 2.0 {
                bar_x - TOOLTIP_WIDTH - 10.0
            } else {
                bar_x + 10.0
            }
            .clamp(LEFT, (width - TOOLTIP_WIDTH - RIGHT).max(LEFT));
            // The label, colour class and value of each segment, leaving out those which are zero.
            let rows: Vec<(String, String, String)> = point.segments(mode)
                .into_iter()
                .zip(&segment_names)
                .filter(|(value, _)| *value != 0.0)
                .map(|(value, (label, class))| (label.clone(), class.clone(), format_value(value)))
                .collect();
            (tooltip_x, point.period_label.clone(), rows, format_value(totals[index]))
        })
    });

    rsx! {
        svg {
            class: if dimensions().is_some() { "stacked-consumption-graph ready" } else { "stacked-consumption-graph" },
            view_box: "0 0 {width} {height}",
            role: "img",
            onresize: move |event| {
                match event.data().get_content_box_size() {
                    Ok(size) if size.width > 0.0 && size.height > 0.0 => {
                        dimensions.set(Some((size.width, size.height)));
                    }
                    Ok(_) => eprintln!("Stacked consumption graph resized to an empty size"),
                    Err(error) => eprintln!("Failed to read stacked consumption graph size: {error:?}"),
                }
            },
            onmouseleave: move |_| hovered.set(None),
            for (ty, label) in y_ticks {
                line { class: "stacked-grid", x1: LEFT, x2: width - RIGHT, y1: ty, y2: ty }
                text { class: "stacked-axis-label", x: LEFT - 8.0, y: ty, text_anchor: "end", dominant_baseline: "middle", "{label}" }
            }
            for (legend_x, label, color_class) in legend {
                rect { class: "stacked-legend-swatch {color_class}", x: legend_x, y: 15.0, width: 10.0, height: 10.0 }
                text { class: "stacked-axis-label", x: legend_x + 15.0, y: 24.0, "{label}" }
            }
            for (tx, label) in x_ticks {
                line { class: "stacked-tick", x1: tx, x2: tx, y1: TOP + plot_height, y2: TOP + plot_height + 4.0 }
                text { class: "stacked-axis-label", x: tx, y: height - 10.0, text_anchor: "middle", "{label}" }
            }
            for (index, class, bx, by, bw, bh) in bars {
                rect {
                    class: "stacked-bar {class}",
                    x: bx,
                    y: by,
                    width: bw,
                    height: bh,
                    onmouseenter: move |_| hovered.set(Some(index)),
                }
            }

            if let Some((tx, period, rows, total)) = tooltip {
                g { class: "stacked-tooltip",
                    rect { class: "stacked-tooltip-background", x: tx, y: TOP, width: TOOLTIP_WIDTH, height: 42.0 + rows.len() as f64 * 14.0, rx: 4 }
                    text { class: "stacked-tooltip-period", x: tx + TOOLTIP_TEXT_X, y: TOP + 16.0, "{period}" }
                    text { class: "stacked-tooltip-value", x: tx + TOOLTIP_TEXT_X, y: TOP + 34.0, "Total: {total}" }
                    for (index, (label, color_class, value)) in rows.iter().enumerate() {
                        rect { class: "stacked-legend-swatch {color_class}", x: tx + 8.0, y: TOP + 43.0 + index as f64 * 14.0, width: 10.0, height: 10.0 }
                        text { class: "stacked-tooltip-value", x: tx + TOOLTIP_TEXT_X, y: TOP + 52.0 + index as f64 * 14.0, "{label}: {value}" }
                    }
                }

            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sparko_graphql::types::Currency;

    use super::{StackedBarMode, StackedBarPoint, format_money};

    fn sample_point() -> StackedBarPoint {
        StackedBarPoint {
            start: 0,
            end: 1,
            tick_label: None,
            period_label: String::new(),
            units: vec![1.0, 2.0, 3.0],
            costs: vec![5.0, 6.0, 7.0],
            standing_charge: 9.0,
            currency: Currency::NONE,
            unit: String::from("kwh"),
        }
    }

    #[test]
    fn money_is_formatted_in_major_units_with_the_currency_symbol() {
        let gbp = Currency::from_code("GBP").unwrap();
        assert_eq!(format_money(123.0, gbp), "£1.23");
        assert_eq!(format_money(-5.5, gbp), "-£0.06");
        assert_eq!(format_money(1234.0, Currency::from_code("JPY").unwrap()), "¥1234");
        assert_eq!(format_money(12.0, Currency::NONE), "12");
    }

    #[test]
    fn cost_segments_put_standing_charge_at_the_bottom() {
        assert_eq!(sample_point().segments(StackedBarMode::Cost), [9.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn units_segments_contain_only_the_series_units() {
        assert_eq!(sample_point().segments(StackedBarMode::Units), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn adding_bucket_values_sums_every_segment() {
        let mut total = sample_point();
        total.add_values_from(&sample_point());

        assert_eq!(total.segments(StackedBarMode::Units), [2.0, 4.0, 6.0]);
        assert_eq!(total.segments(StackedBarMode::Cost), [18.0, 10.0, 12.0, 14.0]);
    }
}
