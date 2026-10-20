use std::sync::Arc;

use anyhow::anyhow;
use dioxus::prelude::*;
use time_tz::Tz;
use sparko_graphql::types::DateTime;

use crate::components::date_range_picker::{DateRangePicker, RangeSelection};
use crate::octopus::meter::MeterManager;
use crate::octopus::stacked_bar_graph::{StackedBarMode, StackedBarPeriod};

#[derive(Props, Clone)]
pub struct HomeAndEvPageProps {
    account_id: String,
    meter_manager: Arc<MeterManager>,
    billing_timezone: &'static Tz,
    path: Vec<String>,
}

impl PartialEq for HomeAndEvPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
            && self.path == other.path
    }
}

#[component]
pub fn HomeAndEvPage(props: HomeAndEvPageProps) -> Element {
    // Fetch the list of all meters (properties), this runs once when the component is mounted.
    let meter_points_resource = use_resource({
        let meter_manager = props.meter_manager.clone();
        let account_id = props.account_id.clone();
        move || {
            let meter_manager = meter_manager.clone();
            let account_id = account_id.clone();
            async move { meter_manager.get_meter_points(&account_id).await }
        }
    });

    match &*meter_points_resource.read() {
        None => rsx! {
            div { "Loading Meter Points for account {props.account_id}..." }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load Meter Points: {error:?}"))?,
        Some(Ok(meter_point_list)) => {

            // Are we looking at one meter?
            if let Some(mpan) = props.path.first() {
            //     if let Some(property) = property_list.property_map.get(mpan) {
            //         let p = property.clone();
                    // The key causes BillDetail to be re-mounted when the bill changes, which drops
                    // (and so cancels) any fetch in progress for the previous bill.
                    rsx! {
                        HomeAndEvConsumption {
                            key: "{mpan}",
                            account_id: props.account_id.clone(),
                            mpan: mpan.clone(),
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
    meter_manager: Arc<MeterManager>,
    billing_timezone: &'static Tz,
}

impl PartialEq for HomeAndEvConsumptionProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.mpan == other.mpan
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
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

    let selection = *applied_range.read();
    let (start, end) = selection.to_date_times(tz).map_err(|error| anyhow!(error))?;

    // FIXME: remove this hardcoded test date range
    // let start = DateTime::from_calendar_date_time(2026, time::Month::October, 1, 5, 0, 0)?;
    // let end = Some(DateTime::from_calendar_date_time(2026, time::Month::October, 1, 6, 0, 0)?);


    let end_key = end.as_ref().map(|d| d.to_string()).unwrap_or_default();

    // The key causes MeterDetail to be re-mounted when the applied range changes, which drops
    // (and so cancels) any fetch in progress for the previous range and starts a new one.
    let detail = rsx! {
        HomeAndEvDetail {
            key: "{start}#{end_key}",
            account_id: props.account_id.clone(),
            mpan: props.mpan.clone(),
            meter_manager: props.meter_manager.clone(),
            billing_timezone: tz,
            start,
            end,
            view: view(),
            graph_mode: graph_mode(),
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
        }
        {detail}
    }
}

#[derive(Props, Clone)]
struct HomeAndEvDetailProps {
    // property: Arc<graphql::meter::account_properties_meters::MeterType>,
    account_id: String,
    mpan: String,
    meter_manager: Arc<MeterManager>,
    billing_timezone: &'static Tz,
    start: DateTime,
    /// None means up to the present
    end: Option<DateTime>,
    view: ConsumptionView,
    graph_mode: StackedBarMode,
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

impl PartialEq for HomeAndEvDetailProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.mpan == other.mpan
            && self.start == other.start
            && self.end == other.end
            && self.view == other.view
            && self.graph_mode == other.graph_mode
            && self.graph_period == other.graph_period
            // && Arc::ptr_eq(&self.property, &other.property)
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

#[component]
fn HomeAndEvDetail(props: HomeAndEvDetailProps) -> Element {
    // Fetch the data
    let transactions_resource = use_resource({
        let props = props.clone();
        move || {
            let props = props.clone();
            async move {
                props.meter_manager.get_home_and_ev_consumption(&props.account_id, &props.mpan,
                    &props.start, &props.end,
                    props.billing_timezone).await
                // props.bill_manager.fetch_bill_transaction_breakdown(props.account_id, props.bill_id, 
                //     props.billing_timezone).await
            }
        }
    });

    match &*transactions_resource.read() {
        None => rsx! {
            {format!("Loading home_and_ev_consumption for meter {}...", props.mpan)}
        },
        Some(Err(error)) => Err(anyhow!("Failed to load home_and_ev_consumption for meter {}: {error:?}", props.mpan))?,
        Some(Ok(consumption)) => match props.view {
            ConsumptionView::Table => MeterManager::display_home_and_ev_consumption(consumption, &props.start, &props.end),
            ConsumptionView::Graph => MeterManager::display_home_and_ev_consumption_graph(
                consumption,
                &props.start,
                &props.end,
                props.billing_timezone,
                props.graph_mode,
                props.graph_period,
            ),
        },
    }
}
