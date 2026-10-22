//! A line graph of values over one year from 1 January, with a line for each year so that years can be compared.

use dioxus::prelude::*;
use sparko_graphql::types::Currency;

use super::consumption_graph::nice_step;
use super::stacked_bar_graph::{StackedBarMode, axis_label};

const LEFT: f64 = 64.0;
const RIGHT: f64 = 16.0;
const TOP: f64 = 44.0;
const BOTTOM: f64 = 32.0;
const Y_TICKS: f64 = 4.0;
const LEGEND_SPACING: f64 = 150.0;
const LEGEND_SAMPLE_LENGTH: f64 = 24.0;
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
/// The day of the year each month starts on, in a year which is not a leap year.
const MONTH_START_DAYS: [f64; 12] = [0.0, 31.0, 59.0, 90.0, 120.0, 151.0, 181.0, 212.0, 243.0, 273.0, 304.0, 334.0];

/// The dash patterns of the lines for each year in turn, as SVG stroke-dasharray values, starting with solid.
pub const YEAR_DASHES: [&str; 6] = ["none", "8 4", "2 3", "12 4 2 4", "4 4", "1 5"];

/// The colour of the year samples in the legend.
pub const YEAR_LEGEND_COLOR: &str = "#c4c4ce";

#[derive(Clone, PartialEq)]
pub struct TrendLine {
    /// A CSS colour, e.g. #46ee43
    pub color: String,
    /// An SVG stroke-dasharray, e.g. "8 4", or "none" for a solid line.
    pub dash: String,
    /// The position of each point in the year, from 0 at the start of 1 January to 1 at the end of 31 December, and
    /// its value.
    pub points: Vec<(f64, f64)>,
}

/// An item in the legend, shown as a short sample of a line.
#[derive(Clone, PartialEq)]
pub struct TrendLegendItem {
    pub label: String,
    pub color: String,
    pub dash: String,
}

#[component]
pub fn TrendGraph(
    lines: Vec<TrendLine>,
    legend: Vec<TrendLegendItem>,
    /// Whether the values are units or costs, for formatting.
    mode: StackedBarMode,
    /// The currency of costs, which are in its minor units, e.g. pence
    currency: Currency,
    /// The unit of consumption, e.g. kwh
    unit: String,
    /// Whether the values are per hour.
    #[props(default)] per_hour: bool,
) -> Element {
    let mut dimensions = use_signal(|| None::<(f64, f64)>);

    if lines.iter().all(|line| line.points.is_empty()) {
        return rsx! { p { "No data for this period" } };
    }

    let (width, height) = dimensions().unwrap_or((960.0, 320.0));
    let plot_width = (width - LEFT - RIGHT).max(1.0);
    let plot_height = (height - TOP - BOTTOM).max(1.0);
    let x = |position: f64| LEFT + position * plot_width;

    let max_value = lines.iter()
        .flat_map(|line| line.points.iter().map(|(_, value)| *value))
        .fold(0.0, f64::max);
    let (step, decimals) = nice_step(max_value / Y_TICKS);
    let y_max = (max_value / step).ceil().max(1.0) * step;
    let y = |value: f64| TOP + plot_height - value.max(0.0) / y_max * plot_height;

    let y_ticks: Vec<(f64, String)> = (0..)
        .map(|i| i as f64 * step)
        .take_while(|value| *value <= y_max + step / 2.0)
        .map(|value| (y(value), axis_label(value, decimals, mode, currency, &unit, per_hour)))
        .collect();

    // A tick at the start of each month, with the name of the month in the middle of it.
    let month_ticks: Vec<(f64, f64, &str)> = MONTHS.iter()
        .zip(MONTH_START_DAYS)
        .enumerate()
        .map(|(index, (name, start_day))| {
            let end_day = MONTH_START_DAYS.get(index + 1).copied().unwrap_or(365.0);
            (x(start_day / 365.0), x((start_day + end_day) / 2.0 / 365.0), *name)
        })
        .collect();

    let paths: Vec<(String, String, String)> = lines.iter()
        .filter(|line| !line.points.is_empty())
        .map(|line| {
            let points = line.points.iter()
                .map(|(position, value)| format!("{:.1},{:.1}", x(*position), y(*value)))
                .collect::<Vec<_>>()
                .join(" ");
            (points, line.color.clone(), line.dash.clone())
        })
        .collect();

    let legend: Vec<(f64, TrendLegendItem)> = legend.into_iter()
        .enumerate()
        .map(|(index, item)| (LEFT + index as f64 * LEGEND_SPACING, item))
        .collect();

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
                    Ok(_) => eprintln!("Trend graph resized to an empty size"),
                    Err(error) => eprintln!("Failed to read trend graph size: {error:?}"),
                }
            },
            for (ty, label) in y_ticks {
                line { class: "stacked-grid", x1: LEFT, x2: width - RIGHT, y1: ty, y2: ty }
                text { class: "stacked-axis-label", x: LEFT - 8.0, y: ty, text_anchor: "end", dominant_baseline: "middle", "{label}" }
            }
            for (legend_x, item) in legend {
                line {
                    class: "trend-line",
                    stroke: "{item.color}",
                    stroke_dasharray: "{item.dash}",
                    x1: legend_x,
                    x2: legend_x + LEGEND_SAMPLE_LENGTH,
                    y1: 20.0,
                    y2: 20.0,
                }
                text { class: "stacked-axis-label", x: legend_x + LEGEND_SAMPLE_LENGTH + 6.0, y: 24.0, "{item.label}" }
            }
            for (tick_x, label_x, name) in month_ticks {
                line { class: "stacked-tick", x1: tick_x, x2: tick_x, y1: TOP + plot_height, y2: TOP + plot_height + 4.0 }
                text { class: "stacked-axis-label", x: label_x, y: height - 10.0, text_anchor: "middle", "{name}" }
            }
            for (points, color, dash) in paths {
                polyline {
                    class: "trend-line",
                    points: "{points}",
                    stroke: "{color}",
                    stroke_dasharray: "{dash}",
                }
            }
        }
    }
}
