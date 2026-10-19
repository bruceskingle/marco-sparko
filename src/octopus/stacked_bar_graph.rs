use dioxus::prelude::*;

use super::consumption_graph::nice_step;

const LEFT: f64 = 64.0;
const RIGHT: f64 = 16.0;
const TOP: f64 = 44.0;
const BOTTOM: f64 = 32.0;
const MAX_X_TICKS: usize = 12;
const Y_TICKS: f64 = 4.0;
const BAR_GAP: f64 = 2.0;
const TOOLTIP_WIDTH: f64 = 210.0;
const TOOLTIP_HEIGHT: f64 = 112.0;

#[derive(Clone, Copy, PartialEq)]
pub enum StackedBarMode {
    Units,
    Cost,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StackedBarPeriod {
    Raw,
    Day,
    Week,
    Month,
}

#[derive(Clone, PartialEq)]
pub struct StackedBarPoint {
    pub start: i64,
    pub end: i64,
    pub tick_label: Option<String>,
    pub period_label: String,
    pub home_standard_units: f64,
    pub home_low_units: f64,
    pub ev_standard_units: f64,
    pub ev_low_units: f64,
    pub home_standard_cost: f64,
    pub home_low_cost: f64,
    pub ev_standard_cost: f64,
    pub ev_low_cost: f64,
    pub standing_charge: f64,
}

impl StackedBarPoint {
    fn segments(&self, mode: StackedBarMode) -> [f64; 5] {
        match mode {
            StackedBarMode::Units => [
                self.home_standard_units,
                self.home_low_units,
                self.ev_standard_units,
                self.ev_low_units,
                0.0,
            ],
            StackedBarMode::Cost => [
                self.standing_charge,
                self.home_standard_cost,
                self.home_low_cost,
                self.ev_standard_cost,
                self.ev_low_cost,
            ],
        }
    }

    pub fn add_values_from(&mut self, other: &Self) {
        self.home_standard_units += other.home_standard_units;
        self.home_low_units += other.home_low_units;
        self.ev_standard_units += other.ev_standard_units;
        self.ev_low_units += other.ev_low_units;
        self.home_standard_cost += other.home_standard_cost;
        self.home_low_cost += other.home_low_cost;
        self.ev_standard_cost += other.ev_standard_cost;
        self.ev_low_cost += other.ev_low_cost;
        self.standing_charge += other.standing_charge;
    }
}

const UNIT_SEGMENTS: [(&str, &str); 4] = [
    ("Home standard", "home-standard"),
    ("Home low", "home-low"),
    ("EV standard", "ev-standard"),
    ("EV low", "ev-low"),
];
const COST_SEGMENTS: [(&str, &str); 5] = [
    ("Standing charge", "standing-charge"),
    ("Home standard", "home-standard"),
    ("Home low", "home-low"),
    ("EV standard", "ev-standard"),
    ("EV low", "ev-low"),
];

#[component]
pub fn StackedBarGraph(points: Vec<StackedBarPoint>, mode: StackedBarMode) -> Element {
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
    let segment_names = match mode {
        StackedBarMode::Units => &UNIT_SEGMENTS[..],
        StackedBarMode::Cost => &COST_SEGMENTS[..],
    };

    let totals: Vec<f64> = points
        .iter()
        .map(|point| point.segments(mode).iter().map(|value| value.max(0.0)).sum())
        .collect();
    let max_value = totals.iter().copied().fold(0.0, f64::max);
    let (step, decimals) = nice_step(max_value / Y_TICKS);
    let y_max = (max_value / step).ceil().max(1.0) * step;
    let y = |value: f64| TOP + plot_height - value / y_max * plot_height;

    let bars: Vec<(usize, &'static str, f64, f64, f64, f64)> = points
        .iter()
        .enumerate()
        .flat_map(|(i, point)| {
            let slot_width = x(point.end) - x(point.start);
            let gap = if slot_width > 3.0 * BAR_GAP { BAR_GAP } else { 0.0 };
            let bar_x = x(point.start) + gap / 2.0;
            let bar_width = (slot_width - gap).max(0.0);
            let mut baseline = 0.0;

            point
                .segments(mode)
                .into_iter()
                .zip(segment_names.iter())
                .map(move |(value, (_, class))| {
                    let value = value.max(0.0);
                    let top = y(baseline + value);
                    let segment = (
                        i,
                        *class,
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
                StackedBarMode::Units => format!("{value:.decimals$}"),
                StackedBarMode::Cost => format!("£{:.2}", value / 100.0),
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
    let legend: Vec<(f64, &str, &str)> = segment_names
        .iter()
        .enumerate()
        .map(|(index, (label, color_class))| {
            (LEFT + index as f64 * 145.0, *label, *color_class)
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
            let segments = point.segments(mode);
            (tooltip_x, point.period_label.clone(), segments, totals[index])
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

            if let Some((tx, period, segments, total)) = tooltip {
                g { class: "stacked-tooltip",
                    rect { x: tx, y: TOP, width: TOOLTIP_WIDTH, height: TOOLTIP_HEIGHT, rx: 4 }
                    text { class: "stacked-tooltip-period", x: tx + 8.0, y: TOP + 16.0, "{period}" }
                    text { class: "stacked-tooltip-value", x: tx + 8.0, y: TOP + 34.0, "Total: {total:.3}" }
                    for (index, (label, _)) in segment_names.iter().enumerate() {
                        text { class: "stacked-tooltip-value", x: tx + 8.0, y: TOP + 52.0 + index as f64 * 14.0, "{label}: {segments[index]:.3}" }
                    }
                }

            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{StackedBarMode, StackedBarPoint};

    fn sample_point() -> StackedBarPoint {
        StackedBarPoint {
            start: 0,
            end: 1,
            tick_label: None,
            period_label: String::new(),
            home_standard_units: 1.0,
            home_low_units: 2.0,
            ev_standard_units: 3.0,
            ev_low_units: 4.0,
            home_standard_cost: 5.0,
            home_low_cost: 6.0,
            ev_standard_cost: 7.0,
            ev_low_cost: 8.0,
            standing_charge: 9.0,
        }
    }

    #[test]
    fn cost_segments_put_standing_charge_at_the_bottom() {
        assert_eq!(
            sample_point().segments(StackedBarMode::Cost),
            [9.0, 5.0, 6.0, 7.0, 8.0]
        );
    }

    #[test]
    fn units_segments_contain_only_the_four_consumption_values() {
        assert_eq!(
            sample_point().segments(StackedBarMode::Units),
            [1.0, 2.0, 3.0, 4.0, 0.0]
        );
    }

    #[test]
    fn adding_bucket_values_sums_every_segment() {
        let mut total = sample_point();
        total.add_values_from(&sample_point());

        assert_eq!(total.segments(StackedBarMode::Units), [2.0, 4.0, 6.0, 8.0, 0.0]);
        assert_eq!(total.segments(StackedBarMode::Cost), [18.0, 10.0, 12.0, 14.0, 16.0]);
    }
}
