//! The categories of measurement shown in graphs, e.g. the standing charge or the consumption at one rate, with the
//! colour and visibility chosen for each.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::{LdChevronDown, LdFilter}};

use crate::components::color_panel::move_color_panel_to_mouse;

pub const STANDING_CHARGE_LABEL: &str = "Standing charge";
pub const TOTAL_LABEL: &str = "Total";

const STANDING_CHARGE_COLOR: &str = "#b8c0cc";
const TOTAL_COLOR: &str = "#f5f5f5";

/// The labels of the current home and EV rates with their default colours, in the order they are shown. Readings may
/// have other labels, e.g. older readings have time of use buckets, which come after these.
pub const KNOWN_LABELS: [(&str, &str); 4] = [
    ("Home Low Rate", "#46ee43"),
    ("Home Standard Rate", "#f6671f"),
    ("EV Low Rate", "#271cff"),
    ("EV Standard Rate", "#f72525"),
];

/// Default colours for labels which are not in KNOWN_LABELS, used in turn.
const OTHER_COLORS: [&str; 6] = ["#f2c94c", "#bb6bd9", "#2dd4bf", "#f472b6", "#a3e635", "#60a5fa"];

#[derive(Clone, Debug, PartialEq)]
pub struct CategoryStyle {
    /// A CSS colour, e.g. #46ee43
    pub color: String,
    pub visible: bool,
}

/// The colours and visibility chosen for each category, categories which have not been changed have the default style.
/// There is also a total, which when visible replaces the visible categories with their total.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryFilter {
    styles: BTreeMap<String, CategoryStyle>,
    total: CategoryStyle,
}

impl Default for CategoryFilter {
    fn default() -> Self {
        Self {
            styles: BTreeMap::new(),
            total: CategoryStyle { color: String::from(TOTAL_COLOR), visible: false },
        }
    }
}

impl CategoryFilter {
    /// The style of each of the given categories, in the same order.
    pub fn styles(&self, categories: &[String]) -> Vec<CategoryStyle> {
        let mut other_colors = OTHER_COLORS.iter().cycle();
        categories.iter()
            .map(|label| {
                // Work out the default for every category so that the palette colours do not depend on which have
                // been changed.
                let default_color = if label == STANDING_CHARGE_LABEL {
                    STANDING_CHARGE_COLOR
                }
                else {
                    KNOWN_LABELS.iter()
                        .find(|(known, _)| known == label)
                        .map(|(_, color)| *color)
                        .unwrap_or_else(|| other_colors.next().unwrap())
                };
                self.styles.get(label)
                    .cloned()
                    .unwrap_or_else(|| CategoryStyle { color: String::from(default_color), visible: true })
            })
            .collect()
    }

    pub fn set_style(&mut self, label: &str, style: CategoryStyle) {
        self.styles.insert(label.to_string(), style);
    }

    pub fn total(&self) -> &CategoryStyle {
        &self.total
    }

    pub fn set_total(&mut self, style: CategoryStyle) {
        self.total = style;
    }
}

/// A category to draw, with the value for each point of the graph.
pub struct CategoryValues {
    pub label: String,
    pub color: String,
    pub values: Vec<f64>,
}

impl CategoryFilter {
    /// The visible categories, or if the total is visible a single category with their total.
    /// categories gives the label and the value for each point of every category.
    pub fn apply(&self, categories: Vec<(String, Vec<f64>)>, points: usize) -> Vec<CategoryValues> {
        let labels: Vec<String> = categories.iter().map(|(label, _)| label.clone()).collect();
        let visible: Vec<CategoryValues> = categories.into_iter()
            .zip(self.styles(&labels))
            .filter(|(_, style)| style.visible)
            .map(|((label, values), style)| CategoryValues { label, color: style.color, values })
            .collect();

        if self.total.visible {
            let values = (0..points)
                .map(|index| visible.iter().map(|category| category.values.get(index).copied().unwrap_or_default()).sum())
                .collect();
            vec![CategoryValues { label: String::from(TOTAL_LABEL), color: self.total.color.clone(), values }]
        }
        else {
            visible
        }
    }
}

/// A button with a menu of the categories, each with a colour picker and a check box to show or hide it, and a total.
#[component]
pub fn CategoryFilterMenu(filter: Signal<CategoryFilter>, categories: Vec<String>, disabled: bool) -> Element {
    let mut menu_open = use_signal(|| false);
    let styles = filter.read().styles(&categories);
    let total = filter.read().total().clone();

    rsx! {
        span { class: "menu-anchor",
            button {
                r#type: "button",
                title: "Choose the categories shown and their colours",
                disabled,
                onclick: move |_| menu_open.set(!menu_open()),
                Icon {
                    class: "inline-icon",
                    icon: LdFilter,
                    width: 14,
                    height: 14,
                }
                " Categories "
                Icon {
                    class: "inline-icon",
                    icon: LdChevronDown,
                    width: 14,
                    height: 14,
                }
            }
            if menu_open() && !disabled {
                // Covers the page so that clicking anywhere else closes the menu.
                div { class: "menu-backdrop", onclick: move |_| menu_open.set(false) }
                div { class: "dropdown-menu category-menu",
                    for (label, style) in categories.iter().cloned().zip(styles) {
                        {category_row(filter, label, style, false)}
                    }
                    div { class: "menu-separator" }
                    {category_row(filter, String::from(TOTAL_LABEL), total, true)}
                }
            }
        }
    }
}

/// A row of the category menu, setting the style of the category, or of the total if is_total.
fn category_row(filter: Signal<CategoryFilter>, label: String, style: CategoryStyle, is_total: bool) -> Element {
    let set_style = move |label: &str, style: CategoryStyle| {
        let mut filter = filter;
        if is_total {
            filter.write().set_total(style);
        }
        else {
            filter.write().set_style(label, style);
        }
    };

    rsx! {
        div { class: "category-row",
            input {
                r#type: "color",
                title: "Choose the colour of {label}",
                value: "{style.color}",
                // On macOS this makes "Show Colors…" open next to the picker rather than wherever it was last left.
                onclick: move |_| move_color_panel_to_mouse(),
                oninput: {
                    let label = label.clone();
                    let style = style.clone();
                    move |event: FormEvent| set_style(&label, CategoryStyle { color: event.value(), ..style.clone() })
                },
            }
            label {
                input {
                    r#type: "checkbox",
                    checked: style.visible,
                    onchange: {
                        let label = label.clone();
                        let style = style.clone();
                        move |event: FormEvent| set_style(&label, CategoryStyle { visible: event.checked(), ..style.clone() })
                    },
                }
                " {label}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CategoryFilter, CategoryStyle, STANDING_CHARGE_LABEL, TOTAL_LABEL};

    fn labels(labels: &[&str]) -> Vec<String> {
        labels.iter().map(|label| label.to_string()).collect()
    }

    #[test]
    fn known_labels_keep_their_colours_and_others_use_the_palette() {
        let styles = CategoryFilter::default().styles(&labels(&[STANDING_CHARGE_LABEL, "Home Low Rate", "OFF_PEAK", "STANDARD_RATE"]));
        let colors: Vec<&str> = styles.iter().map(|style| style.color.as_str()).collect();
        assert_eq!(colors, ["#b8c0cc", "#46ee43", "#f2c94c", "#bb6bd9"]);
        assert!(styles.iter().all(|style| style.visible));
    }

    #[test]
    fn changing_one_category_does_not_change_the_default_colours_of_the_others() {
        let mut filter = CategoryFilter::default();
        filter.set_style("OFF_PEAK", CategoryStyle { color: String::from("#000000"), visible: true });
        let colors: Vec<String> = filter.styles(&labels(&["OFF_PEAK", "STANDARD_RATE"])).into_iter().map(|style| style.color).collect();
        assert_eq!(colors, ["#000000", "#bb6bd9"]);
    }

    #[test]
    fn hidden_categories_are_left_out_and_the_total_replaces_the_visible_ones() {
        let categories = || vec![
            (String::from("A"), vec![1.0, 2.0]),
            (String::from("B"), vec![10.0, 20.0]),
            (String::from("C"), vec![100.0, 200.0]),
        ];
        let mut filter = CategoryFilter::default();
        filter.set_style("B", CategoryStyle { color: String::from("#000000"), visible: false });

        let shown: Vec<String> = filter.apply(categories(), 2).into_iter().map(|category| category.label).collect();
        assert_eq!(shown, ["A", "C"]);

        filter.set_total(CategoryStyle { color: String::from("#ffffff"), visible: true });
        let total = filter.apply(categories(), 2);
        assert_eq!(total.len(), 1);
        assert_eq!(total[0].label, TOTAL_LABEL);
        assert_eq!(total[0].values, [101.0, 202.0]);
    }
}
