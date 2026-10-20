use std::sync::Arc;

use anyhow::anyhow;
use dioxus::prelude::*;
use time_tz::Tz;
use sparko_graphql::types::DateTime;

use crate::components::date_range_picker::{DateRangePicker, RangeSelection};
use crate::data_set::CancelToken;
use crate::octopus::meter::MeterManager;
use crate::octopus::stacked_bar_graph::StackedBarPeriod;

#[derive(Props, Clone)]
pub struct MeterPageProps {
    account_id: String,
    meter_manager: Arc<MeterManager>,
    billing_timezone: &'static Tz,
    path: Vec<String>,
}

impl PartialEq for MeterPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
            && self.path == other.path
    }
}

#[component]
pub fn MeterPage(props: MeterPageProps) -> Element {
    // Fetch the list of all meters (properties), this runs once when the component is mounted.
    let properties_resource = use_resource({
        let meter_manager = props.meter_manager.clone();
        let account_id = props.account_id.clone();
        move || {
            let meter_manager = meter_manager.clone();
            let account_id = account_id.clone();
            async move { meter_manager.get_properties(&account_id).await }
        }
    });

    match &*properties_resource.read() {
        None => rsx! {
            div { "Loading properties for account {props.account_id}..." }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load properties: {error:?}"))?,
        Some(Ok(property_list)) => {

            // Are we looking at one meter?
            if let Some(meter_id) = props.path.first() {
            //     if let Some(property) = property_list.property_map.get(meter_id) {
            //         let p = property.clone();
                    // The key causes BillDetail to be re-mounted when the bill changes, which drops
                    // (and so cancels) any fetch in progress for the previous bill.
                    rsx! {
                        MeterConsumption {
                            key: "{meter_id}",
                            account_id: props.account_id.clone(),
                            meter_id: meter_id.clone(),
                            meter_manager: props.meter_manager.clone(),
                            billing_timezone: props.billing_timezone,
                        }
                    }
            //     }
            //     else {
            //         rsx! {
            //             div { "No such property {property_id}" }
            //         }
            //     }
            }
            else {

                let mut lines = Vec::new();

                for property in property_list.property_map.values() {
                    for meter_point in &property.electricity_meter_points_ {
                        for meter in &meter_point.meters_ {
                            let direction = if meter.import_meter_.is_some() { "Export" } else {"Import"};
                            let hh_readings = if meter.has_and_allows_hh_readings_ { "True" } else { "False" };
                            let id = meter.node_id_.clone();
                            lines.push(rsx!{
                                tr {
                                    td {
                                        div {
                                            class: "link",
                                            onclick: move |_| {

                                                let mut path_signal = use_context::<Signal<Vec<String>>>();
                                                let new_path = vec![String::from("meters"), id.clone()];
                                                path_signal.set(new_path);
                                            },
                                            "{meter.node_id_}"
                                        }
                                    }
                                    td { "{meter.serial_number_}" }
                                    td { {hh_readings} }
                                    td { "Electricity" }
                                    td { {direction} }
                                    td { {meter.consumption_units_.clone()} }
                                }
                            })
                        }
                    }


                    for meter_point in &property.gas_meter_points_ {
                        for meter in &meter_point.meters_ {
                            let direction = "Import";
                            let hh_readings = if meter.has_and_allows_hh_readings_ { "True" } else { "False" };
                            let id = meter.node_id_.clone();
                            lines.push(rsx!{
                                tr {
                                    td {
                                        div {
                                            class: "link",
                                            onclick: move |_| {

                                                let mut path_signal = use_context::<Signal<Vec<String>>>();
                                                let new_path = vec![String::from("meters"), id.clone()];
                                                path_signal.set(new_path);
                                            },
                                            "{meter.node_id_}"
                                        }
                                    }
                                    td { "{meter.serial_number_}" }
                                    td { {hh_readings} }
                                    td { "Gas" }
                                    td { {direction} }
                                    td { {meter.consumption_units_.clone()} }
                                }
                            })
                        }
                    }
                }
                rsx! {
                    table {
                        tr { class: "header",
                            th { "Node ID" }
                            th { "Serial Number" }
                            th { "HH Readings" }
                            th { "Type" }
                            th { "Direction" }
                            th { "Units" }
                        }

                        for line in lines {
                            {line}
                        }
                    }
                }
            }
        },
    }
}

#[derive(Props, Clone)]
struct MeterConsumptionProps {
    account_id: String,
    meter_id: String,
    meter_manager: Arc<MeterManager>,
    billing_timezone: &'static Tz,
}

impl PartialEq for MeterConsumptionProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.meter_id == other.meter_id
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

/*
 * Date range and view selection for a meter, the selected range is passed as props to MeterDetail
 * which does the fetch and display.
 */
#[component]
fn MeterConsumption(props: MeterConsumptionProps) -> Element {
    let tz = props.billing_timezone;

    // The range currently being displayed.
    // Default to the start of the current month with no end.
    let applied_range = use_signal(|| RangeSelection::current_month(tz));

    // Whether consumption is shown as a table or a graph, kept here so it survives range changes.
    let mut view = use_signal(|| ConsumptionView::Table);
    let mut graph_period = use_signal(|| StackedBarPeriod::Raw);

    let selection = *applied_range.read();
    let (start, end) = selection.to_date_times(tz).map_err(|error| anyhow!(error))?;

    // The key causes MeterDetail to be re-mounted when the applied range changes, which drops
    // (and so cancels) any fetch in progress for the previous range and starts a new one.
    let detail = rsx! {
        MeterDetail {
            key: "{start}#{end}",
            account_id: props.account_id.clone(),
            meter_id: props.meter_id.clone(),
            meter_manager: props.meter_manager.clone(),
            billing_timezone: tz,
            start,
            end,
            view: view(),
            graph_period: graph_period(),
        }
    };

    rsx! {
        div { class: "date-range home-ev-date-range",
            DateRangePicker { range: applied_range, timezone: tz }
            span { class: "view-toggle",
                for option in [ConsumptionView::Table, ConsumptionView::Graph] {
                    button {
                        r#type: "button",
                        class: if view() == option { "active" },
                        onclick: move |_| view.set(option),
                        "{option}"
                    }
                }
            }
            if view() == ConsumptionView::Graph {
                span { class: "view-toggle",
                    for option in [StackedBarPeriod::Raw, StackedBarPeriod::Day, StackedBarPeriod::Week, StackedBarPeriod::Month] {
                        button {
                            r#type: "button",
                            class: if graph_period() == option { "active" },
                            onclick: move |_| graph_period.set(option),
                            "{option}"
                        }
                    }
                }
            }
        }
        {detail}
    }
}

#[derive(Props, Clone)]
struct MeterDetailProps {
    // property: Arc<graphql::meter::account_properties_meters::MeterType>,
    account_id: String,
    meter_id: String,
    meter_manager: Arc<MeterManager>,
    billing_timezone: &'static Tz,
    start: DateTime,
    /// Exclusive end of the range
    end: DateTime,
    view: ConsumptionView,
    graph_period: StackedBarPeriod,
}

#[derive(Clone, Copy, PartialEq)]
enum ConsumptionView {
    Table,
    Graph,
}

impl std::fmt::Display for ConsumptionView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsumptionView::Table => write!(f, "Table"),
            ConsumptionView::Graph => write!(f, "Graph"),
        }
    }
}

impl PartialEq for MeterDetailProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.meter_id == other.meter_id
            && self.start == other.start
            && self.end == other.end
            && self.view == other.view
            && self.graph_period == other.graph_period
            // && Arc::ptr_eq(&self.property, &other.property)
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

#[component]
fn MeterDetail(props: MeterDetailProps) -> Element {
    // Set by the cancel button, the fetch then stops and returns the data it has so far.
    let cancel = use_hook(CancelToken::new);

    // Fetch the data
    let transactions_resource = use_resource({
        let props = props.clone();
        let cancel = cancel.clone();
        move || {
            let props = props.clone();
            let cancel = cancel.clone();
            async move {
                props.meter_manager.get_consumption(&props.account_id, &props.meter_id,
                    &props.start, &props.end,
                    props.billing_timezone, &cancel).await
                // props.bill_manager.fetch_bill_transaction_breakdown(props.account_id, props.bill_id, 
                //     props.billing_timezone).await
            }
        }
    });

    match &*transactions_resource.read() {
        None => rsx! {
            {format!("Loading consumption for meter {}...", props.meter_id)}
            " "
            button {
                r#type: "button",
                onclick: move |_| cancel.cancel(),
                "Cancel"
            }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load consumption for meter {}: {error:?}", props.meter_id))?,
        Some(Ok(consumption)) => match props.view {
            ConsumptionView::Table => MeterManager::display_consumption(consumption, &props.start, &props.end),
            ConsumptionView::Graph => MeterManager::display_consumption_graph(
                consumption,
                &props.start,
                &props.end,
                props.billing_timezone,
                props.graph_period,
            ),
        },
    }
}
