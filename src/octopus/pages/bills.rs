use std::sync::Arc;

use anyhow::anyhow;
use dioxus::prelude::*;
use time_tz::Tz;

use crate::octopus::bill::{AbstractBill, BillDataSet, BillTransactionBreakDown};
use crate::octopus::context::OctopusContext;

#[derive(Props, Clone, PartialEq)]
pub struct BillsPageProps {
    path: Vec<String>,
}

#[component]
pub fn BillsPage(props: BillsPageProps) -> Element {
    let context = use_context::<Arc<OctopusContext>>();

    // Fetch the list of all bills, this runs once when the component is mounted.
    // The bills are shared with BillDetail, so they are held in an Arc.
    let bills_resource = use_resource({
        let context = context.clone();
        move || {
            let context = context.clone();
            async move { BillDataSet::new(&context.account_id, &context.cache_config, &*context.request_manager).await.map(Arc::new) }
        }
    });

    match &*bills_resource.read() {
        None => rsx! {
            div { "Loading Bills for account {context.account_id}..." }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load bills: {error:?}"))?,
        Some(Ok(bills)) => {
            // Are we looking at one bill?
            if let Some(bill_id) = props.path.first() {
                // The key causes BillDetail to be re-mounted when the bill changes, which drops
                // (and so cancels) any fetch in progress for the previous bill.
                rsx! {
                    BillDetail {
                        key: "{bill_id}",
                        account_id: context.account_id.clone(),
                        bill_id: bill_id.clone(),
                        bills: bills.clone(),
                        billing_timezone: context.billing_timezone,
                    }
                }
            }
            else {
                rsx! {
                    table {
                        {AbstractBill::gui_summary_header()?}
                        for (_id , (attributes , bill)) in &bills.data_set.data {
                            {bill.gui_summary_line(attributes)?}
                        }
                    }
                }
            }
        },
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
    // bills.data_set.data.get(bill_id)
    if let Some((_attributes, bill)) = bills.data_set.data.get(bill_id) {
        Some(bill)
    }
    else {
        None
    }
}
