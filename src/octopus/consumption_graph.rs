use dioxus::prelude::*;

/// One sample to plot. Times are unix timestamps, labels are pre-formatted for display.
#[derive(Clone, PartialEq)]
pub struct GraphPoint {
    pub start: i64,
    pub end: i64,
    pub value: f64,
    /// Label for the x axis tick placed at the start of this point, if any.
    pub tick_label: Option<String>,
    /// Description of the time period shown in the tooltip.
    pub period_label: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GraphStyle {
    Line,
    Bar,
}

// Margins are in SVG user units; after mounting one unit corresponds to one screen pixel.
const LEFT: f64 = 56.0;
const RIGHT: f64 = 16.0;
const TOP: f64 = 16.0;
const BOTTOM: f64 = 32.0;
const TOOLTIP_WIDTH: f64 = 180.0;
const TOOLTIP_HEIGHT: f64 = 46.0;
const MAX_X_TICKS: usize = 12;
const Y_TICKS: f64 = 4.0;
/// Gap between adjacent bars, dropped when bars are too narrow to spare it.
const BAR_GAP: f64 = 2.0;

/// A line or bar graph of a single series of consumption values, with a tooltip on hover.
#[component]
pub fn ConsumptionGraph(points: Vec<GraphPoint>, style: GraphStyle) -> Element {
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
    let mid_x = |p: &GraphPoint| x(p.start + (p.end - p.start) / 2);

    let max_value = points.iter().map(|p| p.value).fold(0.0, f64::max);
    let (step, decimals) = nice_step(max_value / Y_TICKS);
    let y_max = (max_value / step).ceil().max(1.0) * step;
    let y = |v: f64| TOP + plot_height - v / y_max * plot_height;

    // Break the line wherever samples are missing rather than joining across the gap.
    let mut path = String::new();
    if style == GraphStyle::Line {
        let mut previous_end = None;
        for p in &points {
            let command = if previous_end == Some(p.start) { 'L' } else { 'M' };
            path.push_str(&format!("{command}{:.1},{:.1} ", mid_x(p), y(p.value)));
            previous_end = Some(p.end);
        }
    }

    let bars: Vec<(&'static str, f64, f64, f64, f64)> = if style == GraphStyle::Bar {
        points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let slot_width = x(p.end) - x(p.start);
                let gap = if slot_width > 3.0 * BAR_GAP { BAR_GAP } else { 0.0 };
                let class = if hovered() == Some(i) { "bar hovered" } else { "bar" };
                let top = y(p.value.max(0.0));
                (class, x(p.start) + gap / 2.0, top, slot_width - gap, y(0.0) - top)
            })
            .collect()
    } else {
        Vec::new()
    };

    let y_ticks: Vec<(f64, String)> = (0..)
        .map(|i| i as f64 * step)
        .take_while(|v| *v <= y_max + step / 2.0)
        .map(|v| (y(v), format!("{v:.decimals$}")))
        .collect();

    let all_x_ticks: Vec<(f64, &String)> = points
        .iter()
        .filter_map(|p| p.tick_label.as_ref().map(|label| (x(p.start), label)))
        .collect();
    let every = all_x_ticks.len().div_ceil(MAX_X_TICKS).max(1);
    let x_ticks: Vec<(f64, String)> = all_x_ticks
        .into_iter()
        .step_by(every)
        .map(|(x, label)| (x, label.clone()))
        .collect();

    let hit_targets: Vec<(usize, f64, f64)> = points
        .iter()
        .enumerate()
        .map(|(i, p)| (i, x(p.start), (x(p.end) - x(p.start)).max(1.0)))
        .collect();

    let tooltip = hovered().and_then(|i| points.get(i)).map(|p| {
        let px = mid_x(p);
        let py = y(p.value);
        // Keep the tooltip inside the plot by flipping it to the left of the point on the right half.
        let tx = if px > width / 2.0 { px - TOOLTIP_WIDTH - 10.0 } else { px + 10.0 };
        let value = format!("{:.3}", p.value);
        (px, py, tx, p.period_label.clone(), value)
    });

    rsx! {
        svg {
            class: if dimensions().is_some() { "consumption-graph ready" } else { "consumption-graph" },
            view_box: "0 0 {width} {height}",
            role: "img",
            onresize: move |event| {
                match event.data().get_content_box_size() {
                    Ok(size) if size.width > 0.0 && size.height > 0.0 => {
                        dimensions.set(Some((size.width, size.height)));
                    }
                    Ok(_) => eprintln!("Consumption graph resized to an empty size"),
                    Err(error) => eprintln!("Failed to read consumption graph size: {error:?}"),
                }
            },
            onmouseleave: move |_| hovered.set(None),
            for (ty, label) in y_ticks {
                line { class: "grid", x1: LEFT, x2: width - RIGHT, y1: ty, y2: ty }
                text { class: "axis-label", x: LEFT - 8.0, y: ty, text_anchor: "end", dominant_baseline: "middle", "{label}" }
            }
            for (tx, label) in x_ticks {
                line { class: "tick", x1: tx, x2: tx, y1: TOP + plot_height, y2: TOP + plot_height + 4.0 }
                text { class: "axis-label", x: tx, y: height - 10.0, text_anchor: "middle", "{label}" }
            }
            for (class, bx, by, bw, bh) in bars {
                rect { class, x: bx, y: by, width: bw, height: bh }
            }
            if style == GraphStyle::Line {
                path { class: "line", d: "{path}" }
            }
            if let Some((px, py, tx, period, value)) = tooltip {
                if style == GraphStyle::Line {
                    line { class: "crosshair", x1: px, x2: px, y1: TOP, y2: TOP + plot_height }
                    circle { class: "marker", cx: px, cy: py, r: 4 }
                }
                g { class: "tooltip",
                    rect { x: tx, y: TOP, width: TOOLTIP_WIDTH, height: TOOLTIP_HEIGHT, rx: 4 }
                    text { class: "tooltip-period", x: tx + 10.0, y: TOP + 18.0, "{period}" }
                    text { class: "tooltip-value", x: tx + 10.0, y: TOP + 36.0, "{value}" }
                }
            }
            // Invisible full height bands, one per sample, so hovering anywhere above or below a point selects it.
            for (i, hx, hw) in hit_targets {
                rect {
                    class: "hit",
                    x: hx,
                    y: TOP,
                    width: hw,
                    height: plot_height,
                    onmouseenter: move |_| hovered.set(Some(i)),
                }
            }
        }
    }
}

/// Round a raw axis step up to 1, 2, 2.5 or 5 times a power of ten, returning it with the
/// number of decimal places needed to display multiples of it.
fn nice_step(raw: f64) -> (f64, usize) {
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

#[cfg(test)]
mod tests {
    use super::nice_step;

    #[test]
    fn nice_step_rounds_up_to_readable_values() {
        assert_eq!(nice_step(0.0), (1.0, 0));
        assert_eq!(nice_step(0.3), (0.5, 1));
        assert_eq!(nice_step(0.21), (0.25, 2));
        assert_eq!(nice_step(1.7), (2.0, 0));
        assert_eq!(nice_step(42.0), (50.0, 0));
    }
}
