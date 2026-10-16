use std::sync::Arc;

use anyhow::anyhow;
use dioxus::prelude::*;
use time_tz::Tz;

use crate::octopus::bill::{AbstractBill, BillDataSet, BillManager};

#[derive(Props, Clone)]
pub struct BillsPageProps {
    account_id: String,
    bill_manager: Arc<BillManager>,
    billing_timezone: &'static Tz,
    path: Vec<String>,
}

impl PartialEq for BillsPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && Arc::ptr_eq(&self.bill_manager, &other.bill_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
            && self.path == other.path
    }
}

#[component]
pub fn BillsPage(props: BillsPageProps) -> Element {
    // Fetch the list of all bills, this runs once when the component is mounted.
    let bills_resource = use_resource({
        let bill_manager = props.bill_manager.clone();
        let account_id = props.account_id.clone();
        move || {
            let bill_manager = bill_manager.clone();
            let account_id = account_id.clone();
            async move { bill_manager.fetch_bills(account_id).await }
        }
    });

    match &*bills_resource.read() {
        None => rsx! {
            div { "Loading Bills for account {props.account_id}..." }
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
                        account_id: props.account_id.clone(),
                        bill_id: bill_id.clone(),
                        bills: bills.clone(),
                        bill_manager: props.bill_manager.clone(),
                        billing_timezone: props.billing_timezone,
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
    bill_manager: Arc<BillManager>,
    billing_timezone: &'static Tz,
}

impl PartialEq for BillDetailProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.bill_id == other.bill_id
            && Arc::ptr_eq(&self.bills, &other.bills)
            && Arc::ptr_eq(&self.bill_manager, &other.bill_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

#[component]
fn BillDetail(props: BillDetailProps) -> Element {
    // Fetch all transactions for this bill.
    let transactions_resource = use_resource({
        let props = props.clone();
        move || {
            let props = props.clone();
            async move {
                props.bill_manager.fetch_bill_transaction_breakdown(props.account_id, props.bill_id, 
                    props.billing_timezone).await
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
