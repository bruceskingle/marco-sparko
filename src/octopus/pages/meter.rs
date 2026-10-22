use std::sync::Arc;

use anyhow::anyhow;
use dioxus::prelude::*;
use time_tz::Tz;
use sparko_graphql::types::DateTime;

use crate::components::date_range_picker::{DateRangePicker, RangeSelection};
use crate::components::export::{ExportButton, export_file_name};
use crate::data_set::CancelToken;
use crate::octopus::context::OctopusContext;
use crate::octopus::consumption::ConsumptionDataSet;
use crate::octopus::property::PropertyDataSet;
use crate::octopus::stacked_bar_graph::StackedBarPeriod;

#[derive(Props, Clone, PartialEq)]
pub struct MeterPageProps {
    path: Vec<String>,
}

#[component]
pub fn MeterPage(props: MeterPageProps) -> Element {
    let context = use_context::<Arc<OctopusContext>>();

    // Fetch the list of all meters (properties), this runs once when the component is mounted.
    let properties_resource = use_resource({
        let context = context.clone();
        move || {
            let context = context.clone();
            async move { PropertyDataSet::new(&context.account_id, &context.cache_config, &*context.request_manager).await }
        }
    });

    match &*properties_resource.read() {
        None => rsx! {
            div { "Loading properties for account {context.account_id}..." }
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
                            account_id: context.account_id.clone(),
                            meter_id: meter_id.clone(),
                            billing_timezone: context.billing_timezone,
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
    billing_timezone: &'static Tz,
}

impl PartialEq for MeterConsumptionProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.meter_id == other.meter_id
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

    // The data loaded by MeterDetail for the applied range, None until it has loaded.
    let exportable = use_signal(|| None::<Arc<ConsumptionDataSet>>);

    let selection = *applied_range.read();
    let (start, end) = selection.to_date_times(tz).map_err(|error| anyhow!(error))?;

    // Export what is currently shown, the readings in the table or the aggregated graph values.
    let export_table = move |_: ()| {
        let data = exportable.read().clone()?;
        Some(match view() {
            ConsumptionView::Table => data.readings_export(&start, &end),
            ConsumptionView::Graph | ConsumptionView::Trend => data.graph_export(&start, &end, tz, graph_period()),
        })
    };
    let export_name = export_file_name("meter", &props.meter_id, &start, &end, &view().export_description(graph_period()));

    // The key causes MeterDetail to be re-mounted when the applied range changes, which drops
    // (and so cancels) any fetch in progress for the previous range and starts a new one.
    let detail = rsx! {
        MeterDetail {
            key: "{start}#{end}",
            account_id: props.account_id.clone(),
            meter_id: props.meter_id.clone(),
            billing_timezone: tz,
            start,
            end,
            view: view(),
            graph_period: graph_period(),
            exportable,
        }
    };

    rsx! {
        div { class: "date-range date-range-row",
            DateRangePicker { range: applied_range, timezone: tz }
            span { class: "view-toggle",
                for option in [ConsumptionView::Table, ConsumptionView::Graph, ConsumptionView::Trend] {
                    button {
                        r#type: "button",
                        class: if view() == option { "active" },
                        onclick: move |_| view.set(option),
                        "{option}"
                    }
                }
            }
            if view() != ConsumptionView::Table {
                span { class: "view-toggle",
                    for option in [
                        StackedBarPeriod::Raw,
                        StackedBarPeriod::Day,
                        StackedBarPeriod::Week,
                        StackedBarPeriod::Month,
                    ]
                    {
                        button {
                            r#type: "button",
                            class: if graph_period() == option { "active" },
                            onclick: move |_| graph_period.set(option),
                            "{option}"
                        }
                    }
                }
            }
            ExportButton {
                default_name: export_name,
                disabled: exportable.read().is_none(),
                table: export_table,
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
    billing_timezone: &'static Tz,
    start: DateTime,
    /// Exclusive end of the range
    end: DateTime,
    view: ConsumptionView,
    graph_period: StackedBarPeriod,
    /// Set to the data once it has loaded, so that the parent can export it.
    exportable: Signal<Option<Arc<ConsumptionDataSet>>>,
}

#[derive(Clone, Copy, PartialEq)]
enum ConsumptionView {
    Table,
    Graph,
    /// A line for each year, to compare years.
    Trend,
}

impl std::fmt::Display for ConsumptionView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsumptionView::Table => write!(f, "Table"),
            ConsumptionView::Graph => write!(f, "Graph"),
            ConsumptionView::Trend => write!(f, "Trend"),
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
            && self.exportable == other.exportable
            // && Arc::ptr_eq(&self.property, &other.property)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

#[component]
fn MeterDetail(props: MeterDetailProps) -> Element {
    let context = use_context::<Arc<OctopusContext>>();

    // The latest progress message from the fetch.
    let progress = use_signal(String::new);

    // Set by the cancel button, the fetch then stops and returns the data it has so far.
    let cancel = use_hook(CancelToken::new);

    // Fetch the data
    let transactions_resource = use_resource({
        let props = props.clone();
        let cancel = cancel.clone();
        let context = context.clone();
        move || {
            let props = props.clone();
            let cancel = cancel.clone();
            let context = context.clone();
            async move {
                let monitor = move |message: String| { let mut progress = progress; progress.set(message) };
                ConsumptionDataSet::new(&props.account_id, &props.meter_id,
                    &props.start, &props.end,
                    props.billing_timezone, &monitor,
                    &cancel, &context.cache_config, &*context.request_manager).await.map(Arc::new)
            }
        }
    });

    // Share the data with the parent for export once it has loaded.
    let mut exportable = props.exportable;
    use_effect(move || {
        let loaded = match &*transactions_resource.read() {
            Some(Ok(data)) => Some(data.clone()),
            _ => None,
        };
        exportable.set(loaded);
    });

    match &*transactions_resource.read() {
        None => rsx! {
            div {
                {format!("Loading consumption for meter {}...", props.meter_id)}
                " "
                button { r#type: "button", onclick: move |_| cancel.cancel(), "Cancel" }
            }
            div { "{progress}" }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load consumption for meter {}: {error:?}", props.meter_id))?,
        Some(Ok(consumption)) => match props.view {
            ConsumptionView::Table => consumption.display(&props.start, &props.end),
            ConsumptionView::Graph => consumption.display_graph(
                &props.start,
                &props.end,
                props.billing_timezone,
                props.graph_period,
            ),
            ConsumptionView::Trend => consumption.display_trend(
                &props.start,
                &props.end,
                props.billing_timezone,
                props.graph_period,
            ),
        },
    }
}

impl ConsumptionView {
    /// What an export of this view contains, for the file name.
    fn export_description(&self, period: StackedBarPeriod) -> String {
        match self {
            ConsumptionView::Table => String::from("readings"),
            ConsumptionView::Graph => period.to_string().to_lowercase(),
            ConsumptionView::Trend => format!("trend_{}", period.to_string().to_lowercase()),
        }
    }
}
