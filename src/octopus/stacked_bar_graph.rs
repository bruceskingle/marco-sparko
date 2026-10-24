use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use sparko_graphql::types::Currency;

const LEFT: f64 = 64.0;
const RIGHT: f64 = 16.0;
const TOP: f64 = 44.0;
const BOTTOM: f64 = 32.0;
const MAX_X_TICKS: usize = 12;
const Y_TICKS: f64 = 4.0;
const BAR_GAP: f64 = 2.0;
/// An estimate, a little on the wide side, of the width of a character of the 12px axis labels, to leave out x axis
/// labels which would not fit.
const LABEL_CHAR_WIDTH: f64 = 7.0;
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
    /// A CSS colour, e.g. #46ee43
    pub color: String,
}

#[derive(Clone, PartialEq)]
pub struct StackedBarPoint {
    pub start: i64,
    pub end: i64,
    pub tick_label: Option<String>,
    pub period_label: String,
    /// The value of each series, in the order of the graph's series, bottom first.
    pub values: Vec<f64>,
}

/// A bar which was right clicked, with times as unix timestamps.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BarClick {
    pub start: i64,
    pub end: i64,
    /// The start of the bar before it, or for the first bar the same length before it.
    pub previous_start: i64,
}

/// An item in the menu shown when a bar is right clicked.
#[derive(Clone, PartialEq)]
pub struct BarMenuItem {
    pub label: String,
    pub enabled: bool,
    pub action: EventHandler<BarClick>,
}

/// The height of an item in the bar menu and the width of a character of its labels, a little on the large side, and
/// its padding, to estimate its size.
const MENU_ITEM_HEIGHT: f64 = 30.0;
const MENU_CHAR_WIDTH: f64 = 8.0;
const MENU_PADDING: f64 = 40.0;

/// The CSS position of the bar menu for a right click at x, y in the window: below and to the right of the mouse, or
/// where that would go off the edge of the window, above or to the left of it. Then the menu is placed by its bottom or
/// right edge, so it is at the mouse however good the estimate of its size is.
fn menu_position(x: f64, y: f64, menu: &[BarMenuItem]) -> String {
    let window = dioxus::desktop::window();
    let size = window.window.inner_size().to_logical::<f64>(window.window.scale_factor());
    let height = menu.len() as f64 * MENU_ITEM_HEIGHT + MENU_PADDING;
    let width = menu.iter().map(|item| item.label.chars().count()).max().unwrap_or(0) as f64 * MENU_CHAR_WIDTH + MENU_PADDING;
    let vertical = if y + height > size.height { format!("bottom: {}px;", size.height - y) } else { format!("top: {y}px;") };
    let horizontal = if x + width > size.width { format!("right: {}px;", size.width - x) } else { format!("left: {x}px;") };
    format!("{vertical} {horizontal}")
}

/// A readable step between y axis ticks of at least raw, e.g. 0.25 or 50, and the number of decimal places it needs.
pub(super) fn nice_step(raw: f64) -> (f64, usize) {
    if !(raw > 0.0) {
        return (1.0, 0);
    }
    let magnitude = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0]
        .into_iter()
        .map(|m| m * magnitude)
        .find(|s| *s >= raw)
        .unwrap_or(10.0 * magnitude);
    let mut decimals = 0;
    while decimals < 6 && ((step * 10f64.powi(decimals as i32)).round() - step * 10f64.powi(decimals as i32)).abs() > 1e-9 {
        decimals += 1;
    }
    (step, decimals)
}

/// Format an amount in minor units (e.g. pence) in the major units of the currency, e.g. £1.23
pub(crate) fn format_money(minor_units: f64, currency: Currency) -> String {
    let decimals = currency.minor_units() as usize;
    let major = minor_units / 10f64.powi(decimals as i32);
    let sign = if major < 0.0 { "-" } else { "" };
    format!("{sign}{}{:.decimals$}", currency_symbol(currency), major.abs())
}

fn currency_symbol(currency: Currency) -> String {
    if currency == Currency::NONE { String::new() } else { currency.symbol() }
}

/// A y axis label for the value, just the number, with decimal places for units and in the major units of the currency
/// for costs. The units are shown once, see axis_unit.
pub(crate) fn axis_label(value: f64, decimals: usize, mode: StackedBarMode, currency: Currency) -> String {
    match mode {
        StackedBarMode::Units => format!("{value:.decimals$}"),
        StackedBarMode::Cost => {
            let decimals = currency.minor_units() as usize;
            format!("{:.decimals$}", value / 10f64.powi(decimals as i32))
        },
    }
}

/// The units of the y axis labels, shown once above them, followed by suffix, e.g. kwh/day or £/h
pub(crate) fn axis_unit(mode: StackedBarMode, currency: Currency, unit: &str, suffix: &str) -> String {
    match mode {
        StackedBarMode::Units => format!("{unit}{suffix}"),
        StackedBarMode::Cost => format!("{}{suffix}", currency_symbol(currency)),
    }
}

#[component]
pub fn StackedBarGraph(
    points: Vec<StackedBarPoint>,
    /// The series stacked in each bar, bottom first.
    series: Vec<StackedBarSeries>,
    /// Whether the values are units or costs, for formatting.
    mode: StackedBarMode,
    /// The currency of costs, which are in its minor units, e.g. pence
    currency: Currency,
    /// The unit of consumption, e.g. kwh
    unit: String,
    /// Draw a histogram, where bar heights are per hour so that the area of each bar is its total. Needed when the bars
    /// have different widths.
    #[props(default)] per_hour: bool,
    /// Whether each x axis label is in the middle of its bar, e.g. a day, rather than starting at the start of its bar,
    /// e.g. the first reading of a day.
    #[props(default)] centre_labels: bool,
    /// Called with the start and end of a bar when it is double clicked, to show it in more detail.
    #[props(default)] on_drill: Option<EventHandler<(i64, i64)>>,
    /// Called with the start of the first bar and the end of the last when the mouse is dragged across more than one
    /// bar, to show just those bars.
    #[props(default)] on_zoom: Option<EventHandler<(i64, i64)>>,
    /// The menu shown when a bar is right clicked, instead of the webview's menu.
    #[props(default)] menu: Vec<BarMenuItem>,
) -> Element {
    let mut hovered = use_signal::<Option<usize>>(|| None);
    // The bars where a drag started and where it is now, while the mouse button is down.
    let mut dragged = use_signal::<Option<(usize, usize)>>(|| None);
    // The bar which was right clicked and where, while its menu is open.
    let mut menu_at = use_signal::<Option<(BarClick, f64, f64)>>(|| None);
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
    // The label and colour of each segment, matching StackedBarPoint::values.
    let segment_names: Vec<(String, String)> = series.iter()
        .map(|series| (series.label.clone(), series.color.clone()))
        .collect();
    // The factor from a bar's values to its height.
    let scale = move |point: &StackedBarPoint| {
        if per_hour { 3600.0 / (point.end - point.start).max(1) as f64 } else { 1.0 }
    };
    let totals: Vec<f64> = points
        .iter()
        .map(|point| point.values.iter().map(|value| value.max(0.0)).sum())
        .collect();
    let max_value = points
        .iter()
        .zip(&totals)
        .map(|(point, total)| total * scale(point))
        .fold(0.0, f64::max);
    let format_value = |value: f64| match mode {
        StackedBarMode::Units => format!("{value:.3} {unit}"),
        StackedBarMode::Cost => format_money(value, currency),
    };
    let (step, decimals) = nice_step(max_value / Y_TICKS);
    let y_max = (max_value / step).ceil().max(1.0) * step;
    let y = |value: f64| TOP + plot_height - value / y_max * plot_height;

    let bars: Vec<(String, f64, f64, f64, f64)> = points
        .iter()
        .flat_map(|point| {
            let slot_width = x(point.end) - x(point.start);
            let gap = if slot_width > 3.0 * BAR_GAP { BAR_GAP } else { 0.0 };
            let bar_x = x(point.start) + gap / 2.0;
            let bar_width = (slot_width - gap).max(0.0);
            let bar_scale = scale(point);
            let mut baseline = 0.0;

            point
                .values
                .iter()
                .copied()
                .zip(segment_names.iter())
                .map(move |(value, (_, color))| {
                    let value = value.max(0.0) * bar_scale;
                    let top = y(baseline + value);
                    let segment = (
                        color.clone(),
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
            (y(value), axis_label(value, decimals, mode, currency))
        })
        .collect();
    let y_unit = axis_unit(mode, currency, &unit, if per_hour { "/h" } else { "" });

    // The tick at the start of each labelled bar, and its label in the middle of the bar or starting at the tick.
    let label_anchor = if centre_labels { "middle" } else { "start" };
    let all_x_ticks: Vec<(f64, f64, &String)> = points
        .iter()
        .filter_map(|point| point.tick_label.as_ref().map(|label| {
            let label_x = if centre_labels { (x(point.start) + x(point.end)) / 2.0 } else { x(point.start) };
            (x(point.start), label_x, label)
        }))
        .collect();
    let every = all_x_ticks.len().div_ceil(MAX_X_TICKS).max(1);
    let x_ticks: Vec<(f64, f64, String)> = all_x_ticks
        .into_iter()
        .step_by(every)
        // Leave out a label which would go off the edge, e.g. the last day of raw readings when it is near the end.
        .filter(|(_, label_x, label)| {
            let label_width = label.chars().count() as f64 * LABEL_CHAR_WIDTH;
            let (left, right) = if centre_labels {
                (label_x - label_width / 2.0, label_x + label_width / 2.0)
            }
            else {
                (*label_x, label_x + label_width)
            };
            left >= 0.0 && right <= width
        })
        .map(|(tick_x, label_x, label)| (tick_x, label_x, label.clone()))
        .collect();
    let legend: Vec<(f64, String, String)> = segment_names
        .iter()
        .enumerate()
        .map(|(index, (label, color))| {
            (LEFT + index as f64 * 145.0, label.clone(), color.clone())
        })
        .collect();

    // A column over each bar, the full height of the graph, which shows its tooltip when the mouse is over it and can
    // be double clicked or dragged across, and the start and end of each bar.
    let columns: Vec<(usize, f64, f64)> = points.iter()
        .enumerate()
        .map(|(index, point)| (index, x(point.start), (x(point.end) - x(point.start)).max(0.0)))
        .collect();
    let periods: std::rc::Rc<Vec<(i64, i64)>> = std::rc::Rc::new(points.iter().map(|point| (point.start, point.end)).collect());
    // The bars being dragged across, from the left of the first to the right of the last.
    let selection = dragged().map(|(from, to)| {
        let (first, last) = (from.min(to), from.max(to));
        let left = x(points[first].start);
        (left, x(points[last].end) - left)
    });

    let tooltip = hovered().and_then(|index| {
        points.get(index).map(|point| {
            let bar_x = x(point.start + (point.end - point.start) / 2);
            let tooltip_x = if bar_x > width / 2.0 {
                bar_x - TOOLTIP_WIDTH - 10.0
            } else {
                bar_x + 10.0
            }
            .clamp(LEFT, (width - TOOLTIP_WIDTH - RIGHT).max(LEFT));
            // The label, colour and value of each segment, leaving out those which are zero.
            let rows: Vec<(String, String, String)> = point.values
                .iter()
                .copied()
                .zip(&segment_names)
                .filter(|(value, _)| *value != 0.0)
                .map(|(value, (label, color))| (label.clone(), color.clone(), format_value(value)))
                .collect();
            (tooltip_x, point.period_label.clone(), rows, format_value(totals[index]))
        })
    });

    rsx! {
        svg {
            class: format_args!(
                "stacked-consumption-graph{}{}",
                if dimensions().is_some() { " ready" } else { "" },
                if dragged().is_some_and(|(from, to)| from != to) { " zooming" } else { "" },
            ),
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
            onmouseleave: move |_| {
                hovered.set(None);
                dragged.set(None);
            },
            for (ty, label) in y_ticks {
                line { class: "stacked-grid", x1: LEFT, x2: width - RIGHT, y1: ty, y2: ty }
                text { class: "stacked-axis-label", x: LEFT - 8.0, y: ty, text_anchor: "end", dominant_baseline: "middle", "{label}" }
            }
            // The units of the y axis labels, above them in the row of the legend.
            text { class: "stacked-axis-label", x: LEFT - 8.0, y: 24.0, text_anchor: "end", "{y_unit}" }
            for (legend_x, label, color) in legend {
                rect { class: "stacked-legend-swatch", fill: "{color}", x: legend_x, y: 15.0, width: 10.0, height: 10.0 }
                text { class: "stacked-axis-label", x: legend_x + 15.0, y: 24.0, "{label}" }
            }
            for (tx, lx, label) in x_ticks {
                line { class: "stacked-tick", x1: tx, x2: tx, y1: TOP + plot_height, y2: TOP + plot_height + 4.0 }
                text { class: "stacked-axis-label", x: lx, y: height - 10.0, text_anchor: label_anchor, "{label}" }
            }
            for (color, bx, by, bw, bh) in bars {
                rect {
                    class: "stacked-bar",
                    fill: "{color}",
                    x: bx,
                    y: by,
                    width: bw,
                    height: bh,
                }
            }
            if let Some((selection_x, selection_width)) = selection {
                rect { class: "stacked-selection", x: selection_x, y: TOP, width: selection_width, height: plot_height }
            }
            for (index, column_x, column_width) in columns {
                rect {
                    class: "stacked-hit",
                    x: column_x,
                    y: TOP,
                    width: column_width,
                    height: plot_height,
                    onmouseenter: move |_| {
                        hovered.set(Some(index));
                        if let Some((from, _)) = dragged() {
                            dragged.set(Some((from, index)));
                        }
                    },
                    onmousedown: move |event| {
                        // Only the left button drags, the right one opens the menu.
                        if event.trigger_button() == Some(MouseButton::Primary) {
                            dragged.set(Some((index, index)));
                        }
                    },
                    oncontextmenu: {
                        let periods = periods.clone();
                        let has_menu = !menu.is_empty();
                        move |event: MouseEvent| {
                            event.prevent_default();
                            if has_menu {
                                let (start, end) = periods[index];
                                let previous_start = if index > 0 { periods[index - 1].0 } else { start - (end - start) };
                                let point = event.client_coordinates();
                                menu_at.set(Some((BarClick { start, end, previous_start }, point.x, point.y)));
                            }
                        }
                    },
                    onmouseup: {
                        let periods = periods.clone();
                        move |_| {
                            // A drag across more than one bar zooms to them, a click on one bar does nothing.
                            if let (Some((from, to)), Some(on_zoom)) = (dragged.take(), on_zoom) {
                                if from != to {
                                    on_zoom.call((periods[from.min(to)].0, periods[from.max(to)].1));
                                }
                            }
                        }
                    },
                    ondoubleclick: {
                        let periods = periods.clone();
                        move |_| {
                            if let Some(on_drill) = on_drill {
                                on_drill.call(periods[index]);
                            }
                        }
                    },
                }
            }

            if let Some((tx, period, rows, total)) = tooltip {
                g { class: "stacked-tooltip",
                    rect { class: "stacked-tooltip-background", x: tx, y: TOP, width: TOOLTIP_WIDTH, height: 42.0 + rows.len() as f64 * 14.0, rx: 4 }
                    text { class: "stacked-tooltip-period", x: tx + TOOLTIP_TEXT_X, y: TOP + 16.0, "{period}" }
                    text { class: "stacked-tooltip-value", x: tx + TOOLTIP_TEXT_X, y: TOP + 34.0, "Total: {total}" }
                    for (index, (label, color, value)) in rows.iter().enumerate() {
                        rect { class: "stacked-legend-swatch", fill: "{color}", x: tx + 8.0, y: TOP + 43.0 + index as f64 * 14.0, width: 10.0, height: 10.0 }
                        text { class: "stacked-tooltip-value", x: tx + TOOLTIP_TEXT_X, y: TOP + 52.0 + index as f64 * 14.0, "{label}: {value}" }
                    }
                }

            }
        }
        if let Some((click, menu_x, menu_y)) = menu_at() {
            // Covers the page so that clicking anywhere else, or right clicking, closes the menu.
            div {
                class: "menu-backdrop",
                onclick: move |_| menu_at.set(None),
                oncontextmenu: move |event| {
                    event.prevent_default();
                    menu_at.set(None);
                },
            }
            div {
                class: "dropdown-menu bar-menu",
                style: menu_position(menu_x, menu_y, &menu),
                for item in menu.iter().cloned() {
                    button {
                        r#type: "button",
                        disabled: !item.enabled,
                        onclick: move |_| {
                            menu_at.set(None);
                            item.action.call(click);
                        },
                        "{item.label}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sparko_graphql::types::Currency;

    use super::{StackedBarMode, axis_label, axis_unit, format_money, nice_step};

    #[test]
    fn nice_step_rounds_up_to_readable_values() {
        assert_eq!(nice_step(0.0), (1.0, 0));
        assert_eq!(nice_step(0.3), (0.5, 1));
        assert_eq!(nice_step(0.21), (0.25, 2));
        assert_eq!(nice_step(1.7), (2.0, 0));
        assert_eq!(nice_step(42.0), (50.0, 0));
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
    fn axis_labels_are_just_numbers_and_the_units_are_shown_once() {
        let gbp = Currency::from_code("GBP").unwrap();
        assert_eq!(axis_label(2.5, 1, StackedBarMode::Units, gbp), "2.5");
        assert_eq!(axis_label(125.0, 0, StackedBarMode::Cost, gbp), "1.25");
        assert_eq!(axis_unit(StackedBarMode::Units, gbp, "kwh", "/day"), "kwh/day");
        assert_eq!(axis_unit(StackedBarMode::Cost, gbp, "kwh", "/h"), "£/h");
        assert_eq!(axis_unit(StackedBarMode::Cost, Currency::NONE, "kwh", ""), "");
    }
}
