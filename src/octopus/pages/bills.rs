use std::sync::Arc;

use anyhow::anyhow;
use dioxus::desktop::LogicalSize;
use dioxus::prelude::*;
use time_tz::Tz;

use crate::components::app::MAIN_CSS;
use crate::components::child_windows;
use crate::octopus::bill::{AbstractBill, BillDataSet, BillTransactionBreakDown};
use crate::octopus::context::OctopusContext;

#[derive(Props, Clone, PartialEq)]
pub struct BillsPageProps {
    path: Vec<String>,
}

/// The list of bills. Each bill opens in a window of its own, see child_windows, so that the list stays where it was
/// behind it, and bills can be compared side by side.
#[component]
pub fn BillsPage(props: BillsPageProps) -> Element {
    let _ = props;
    let context = use_context::<Arc<OctopusContext>>();

    // Fetch the list of all bills, this runs once when the component is mounted.
    // The bills are shared with the bill windows, so they are held in an Arc.
    let bills_resource = use_resource({
        let context = context.clone();
        move || {
            let context = context.clone();
            async move { BillDataSet::new(&context.account_id, &context.cache_config, &*context.request_manager).await.map(Arc::new) }
        }
    });

    let open_bill = use_callback({
        let context = context.clone();
        move |bill_id: String| {
            let Some(Ok(bills)) = &*bills_resource.peek() else { return };
            let title = match find_bill(&bill_id, bills) {
                Some(bill) => {
                    let bill = bill.as_bill_interface();
                    format!("Bill {} ({} to {})", bill.id_, bill.from_date_, bill.to_date_)
                },
                None => format!("Bill {bill_id}"),
            };
            child_windows::open(&bill_id, title, LogicalSize::new(1100.0, 800.0), || VirtualDom::new_with_props(BillWindow, BillWindowProps {
                context: context.clone(),
                bill_id: bill_id.clone(),
                bills: bills.clone(),
            }));
        }
    });

    match &*bills_resource.read() {
        None => rsx! {
            div { "Loading Bills for account {context.account_id}..." }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load bills: {error:?}"))?,
        Some(Ok(bills)) => {
            rsx! {
                table {
                    {AbstractBill::gui_summary_header()?}
                    // The most recent first.
                    for (_id , (attributes , bill)) in bills.data_set.data.iter().rev() {
                        {bill.gui_summary_line(attributes, open_bill)?}
                    }
                }
            }
        },
    }
}

#[derive(Props, Clone)]
struct BillWindowProps {
    context: Arc<OctopusContext>,
    bill_id: String,
    bills: Arc<BillDataSet>,
}

impl PartialEq for BillWindowProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.context, &other.context)
            && self.bill_id == other.bill_id
            && Arc::ptr_eq(&self.bills, &other.bills)
    }
}

/// The root of a bill's window, which has its own VirtualDom, so it provides the OctopusContext and the stylesheet
/// itself. Escape closes it.
#[component]
fn BillWindow(props: BillWindowProps) -> Element {
    use_context_provider(|| props.context.clone());
    child_windows::use_child_window();

    use_future(|| async {
        let mut keys = document::eval("document.addEventListener('keydown', (event) => { if (event.key === 'Escape') { dioxus.send(true); } });");
        while keys.recv::<bool>().await.is_ok() {
            dioxus::desktop::window().close();
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        ErrorBoundary {
            handle_error: |errors: ErrorContext| rsx! {
                div { class: "error", "Failed to show the bill" }
                pre { "{errors:?}" }
            },
            div { class: "bill-window",
                BillDetail {
                    account_id: props.context.account_id.clone(),
                    bill_id: props.bill_id.clone(),
                    bills: props.bills.clone(),
                    billing_timezone: props.context.billing_timezone,
                }
            }
        }
    }
}

#[derive(Props, Clone)]
struct BillDetailProps {
    account_id: String,
    bill_id: String,
    bills: Arc<BillDataSet>,
    billing_timezone: &'static Tz,
}

impl PartialEq for BillDetailProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.bill_id == other.bill_id
            && Arc::ptr_eq(&self.bills, &other.bills)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

#[component]
fn BillDetail(props: BillDetailProps) -> Element {
    let context = use_context::<Arc<OctopusContext>>();

    // Fetch all transactions for this bill.
    let transactions_resource = use_resource({
        let props = props.clone();
        let context = context.clone();
        move || {
            let props = props.clone();
            let context = context.clone();
            async move {
                BillTransactionBreakDown::for_statement(props.account_id, props.bill_id,
                    props.billing_timezone, &context.cache_config, &*context.request_manager).await
            }
        }
    });

    let Some(bill) = find_bill(&props.bill_id, &props.bills) else {
        return rsx! {
            {format!("No such bill {}", props.bill_id)}
        };
    };

    match &*transactions_resource.read() {
        None => rsx! {
            {format!("Loading transactions for bill {}...", props.bill_id)}
        },
        Some(Err(error)) => Err(anyhow!("Failed to load transactions for bill {}: {error:?}", props.bill_id))?,
        Some(Ok(bill_transactions)) => bill.gui_display(bill_transactions),
    }
}

fn find_bill<'a>(bill_id: &String, bills: &'a BillDataSet) -> Option<&'a AbstractBill> {
    bills.data_set.data.get(bill_id).map(|(_attributes, bill)| bill)
}
