use std::sync::Arc;

use anyhow::anyhow;
use dioxus::prelude::*;
use time_tz::Tz;
use sparko_graphql::types::DateTime;

use crate::components::date_range_picker::{DateRangePicker, RangeSelection};
use crate::components::export::{ExportButton, export_file_name};
use crate::data_set::CancelToken;
use crate::octopus::context::OctopusContext;
use crate::octopus::home_and_ev::HomeAndEvDataSet;
use crate::octopus::meter_point::MeterPointDataSet;
use crate::octopus::stacked_bar_graph::{StackedBarMode, StackedBarPeriod};

#[derive(Props, Clone, PartialEq)]
pub struct HomeAndEvPageProps {
    path: Vec<String>,
}

#[component]
pub fn HomeAndEvPage(props: HomeAndEvPageProps) -> Element {
    let context = use_context::<Arc<OctopusContext>>();

    // Fetch the list of all meters (properties), this runs once when the component is mounted.
    let meter_points_resource = use_resource({
        let context = context.clone();
        move || {
            let context = context.clone();
            async move { MeterPointDataSet::new(&context.account_id, &context.cache_config, &*context.request_manager).await }
        }
    });

    match &*meter_points_resource.read() {
        None => rsx! {
            div { "Loading Meter Points for account {context.account_id}..." }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load Meter Points: {error:?}"))?,
        Some(Ok(meter_point_list)) => {

            // Are we looking at one meter?
            if let Some(mpan) = props.path.first() {
                    rsx! {
                        HomeAndEvConsumption {
                            key: "{mpan}",
                            account_id: context.account_id.clone(),
                            mpan: mpan.clone(),
                            billing_timezone: context.billing_timezone,
                        }
                    }
            }
            else {

                let mut lines = Vec::new();


                for meter_point in &meter_point_list.meter_points {

                    let product_code = meter_point.agreements_.first()
                        .and_then(|agreement| agreement.tariff_.as_ref())
                        .and_then(|tariff| match tariff {
                            crate::octopus::graphql::meter::active_meter_points_with_agreements::ElectricityTariffType::StandardTariff(t) => t.product_code_.as_deref(),
                            crate::octopus::graphql::meter::active_meter_points_with_agreements::ElectricityTariffType::DayNightTariff(t) => t.product_code_.as_deref(),
                            crate::octopus::graphql::meter::active_meter_points_with_agreements::ElectricityTariffType::ThreeRateTariff(t) => t.product_code_.as_deref(),
                            crate::octopus::graphql::meter::active_meter_points_with_agreements::ElectricityTariffType::FourRateEvTariff(t) => t.product_code_.as_deref(),
                            crate::octopus::graphql::meter::active_meter_points_with_agreements::ElectricityTariffType::HalfHourlyTariff(t) => t.product_code_.as_deref(),
                            crate::octopus::graphql::meter::active_meter_points_with_agreements::ElectricityTariffType::PrepayTariff(t) => t.product_code_.as_deref(),

                        })
                        .unwrap_or("UNKNOWN");


                        
                    let direction = meter_point.direction_.as_ref().map_or("UNKNOWN".to_string(), |v| v.to_string());
                    let id = meter_point.mpan_.clone();
                    let mpan_link = if match &meter_point.direction_ {
                        Some(dir) => match dir {
                            crate::octopus::graphql::ElectricityDirection::Import => true,
                            crate::octopus::graphql::ElectricityDirection::Export => false,
                        },
                        None => false,
                    } {
                        rsx! {
                            div {
                                class: "link",
                                onclick: move |_| {

                                    let mut path_signal = use_context::<Signal<Vec<String>>>();
                                    let new_path = vec![String::from("home_and_ev"), id.clone()];
                                    path_signal.set(new_path);
                                },
                                {meter_point.mpan_.clone()}
                            }
                        }
                    }
                    else {
                        rsx! {
                            {meter_point.mpan_.clone()}
                        }
                    };
                    lines.push(rsx!{
                        tr {
                            td { {mpan_link} }
                            td { {direction} }
                            td { {product_code} }
                        }
                    })
                }


                rsx! {
                    h1 { "Meter Points" }
                    table {
                        tr { class: "header",
                            th { "MPAN" }
                            th { "Direction" }
                            th { "Product Code" }
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
struct HomeAndEvConsumptionProps {
    account_id: String,
    mpan: String,
    billing_timezone: &'static Tz,
}

impl PartialEq for HomeAndEvConsumptionProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.mpan == other.mpan
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

/*
 * Date range and view selection for a meter, the selected range is passed as props to MeterDetail
 * which does the fetch and display.
 */
#[component]
fn HomeAndEvConsumption(props: HomeAndEvConsumptionProps) -> Element {
    let tz = props.billing_timezone;

    // The range currently being displayed.
    // Default to the start of the current month with no end.
    let applied_range = use_signal(|| RangeSelection::current_month(tz));

    // Whether consumption is shown as a table or a graph, kept here so it survives range changes.
    let mut view = use_signal(|| ConsumptionView::Table);
    let mut graph_mode = use_signal(|| StackedBarMode::Cost);
    let mut graph_period = use_signal(|| StackedBarPeriod::Raw);

    // The data loaded by HomeAndEvDetail for the applied range, None until it has loaded.
    let exportable = use_signal(|| None::<Arc<HomeAndEvDataSet>>);

    let selection = *applied_range.read();
    let (start, end) = selection.to_date_times(tz).map_err(|error| anyhow!(error))?;

    // Export what is currently shown, the readings in the table or the aggregated graph values.
    let export_table = move |_: ()| {
        let data = exportable.read().clone()?;
        Some(match view() {
            ConsumptionView::Table => data.readings_export(&start, &end),
            ConsumptionView::Graph => data.graph_export(&start, &end, tz, graph_period()),
        })
    };
    let export_name = export_file_name("home_and_ev", &props.mpan, &start, &end, &view().export_description(graph_period()));

    // FIXME: remove this hardcoded test date range
    // let start = DateTime::from_calendar_date_time(2026, time::Month::October, 1, 5, 0, 0)?;
    // let end = Some(DateTime::from_calendar_date_time(2026, time::Month::October, 1, 6, 0, 0)?);



    // The key causes MeterDetail to be re-mounted when the applied range changes, which drops
    // (and so cancels) any fetch in progress for the previous range and starts a new one.
    let detail = rsx! {
        HomeAndEvDetail {
            key: "{start}#{end}",
            account_id: props.account_id.clone(),
            mpan: props.mpan.clone(),
            billing_timezone: tz,
            start,
            end,
            view: view(),
            graph_mode: graph_mode(),
            graph_period: graph_period(),
            exportable,
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
                    for option in [StackedBarMode::Cost, StackedBarMode::Units] {
                        button {
                            r#type: "button",
                            class: if graph_mode() == option { "active" },
                            onclick: move |_| graph_mode.set(option),
                            "{option}"
                        }
                    }
                }
                span { class: "view-toggle",
                    for option in [
                        StackedBarPeriod::Raw,
                        StackedBarPeriod::Rate,
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
struct HomeAndEvDetailProps {
    // property: Arc<graphql::meter::account_properties_meters::MeterType>,
    account_id: String,
    mpan: String,
    billing_timezone: &'static Tz,
    start: DateTime,
    /// Exclusive end of the range
    end: DateTime,
    view: ConsumptionView,
    graph_mode: StackedBarMode,
    graph_period: StackedBarPeriod,
    /// Set to the data once it has loaded, so that the parent can export it.
    exportable: Signal<Option<Arc<HomeAndEvDataSet>>>,
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

impl PartialEq for HomeAndEvDetailProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.mpan == other.mpan
            && self.start == other.start
            && self.end == other.end
            && self.view == other.view
            && self.graph_mode == other.graph_mode
            && self.graph_period == other.graph_period
            && self.exportable == other.exportable
            // && Arc::ptr_eq(&self.property, &other.property)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

#[component]
fn HomeAndEvDetail(props: HomeAndEvDetailProps) -> Element {
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
                HomeAndEvDataSet::new(&props.account_id, &props.mpan,
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
                {format!("Loading home_and_ev_consumption for meter {}...", props.mpan)}
                " "
                button { r#type: "button", onclick: move |_| cancel.cancel(), "Cancel" }
            }
            div { "{progress}" }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load home_and_ev_consumption for meter {}: {error:?}", props.mpan))?,
        Some(Ok(consumption)) => match props.view {
            ConsumptionView::Table => consumption.display(&props.start, &props.end),
            ConsumptionView::Graph => consumption.display_graph(
                &props.start,
                &props.end,
                props.billing_timezone,
                props.graph_mode,
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
        }
    }
}
