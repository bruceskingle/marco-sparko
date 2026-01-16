use std::collections::HashMap;
use dioxus::prelude::*;
use indexmap::IndexMap;
use std::sync::Arc;

use anyhow::anyhow;

use sparko_graphql::AuthenticatedRequestManager;

use crate::octopus::bill::transaction::BillTransactionBreakDown;
use crate::octopus::decimal::Decimal;
use crate::octopus::meter::MeterType;
use crate::util::as_decimal;
use crate::CacheManager;

use super::graphql::{bill, meter};
use super::meter::{MeterManager, Tariff};
// use bill::get_bills::AbstractBill;
// use bill::get_statement_transactions::TransactionType;
// use super::graphql::BillType;

use super::{token::OctopusTokenManager};

pub mod manager;

// const one_hundred: Decimal = Decimal::new(100, 0);
// const format: time::format_description = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

type BillType = super::graphql::BillType;

impl BillType {
    fn as_str(&self) -> &'static str {
        match self {
            super::graphql::BillType::Statement => "Statement",
            super::graphql::BillType::Invoice => "Invoice",
            super::graphql::BillType::CreditNote => "CreditNote",
            super::graphql::BillType::PreKraken => "PreKraken",
        }
    }
}

type Bill = bill::get_bills::AbstractBill;

impl Bill {
    // pub async fn bill_gui_handler(&self, account_number: &String, meter_manager: &MeterManager, billing_timezone: &time_tz::Tz) ->  anyhow::Result<()> {
    //     // let one_hundred = Decimal::new(100, 0);
    //     // let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
    //     let cache_manager: Arc<CacheManager> = self.cache_manager.clone();
    //     let request_manager = self.request_manager.clone();
    //     let bills = self.get_bills(account_number).await?;

    //     if let Some(bill_id) = args.next() {
    //         for (_id, bill) in &bills.bills {
    //             if bill_id == bill.as_bill_interface().id_ {
    //                 let transactions = if let bill::get_bills::AbstractBill::StatementType(_) = bill {


    //                     let transactions = Self::fetch_bill_transaction_breakdown(&cache_manager, &request_manager, account_number.clone(), bill_id.to_string(), meter_manager, billing_timezone).await?;

    //                     Some(transactions)
    //                 }
    //                 else {
    //                     None
    //                 };

    //                 bill.print(transactions);
    //                 return Ok(())
    //             }
    //         }
    //         //println!("Unknown bill '{}'", bill_id);
    //     }
    //     else {
    //         if bills.bills.is_empty() {
    //             //println!("There are no bills in this account");
    //         }
    //         else {
    //             let (_id, bill) = bills.bills.get(bills.bills.len() - 1).unwrap();
    //             let transactions = if let bill::get_bills::AbstractBill::StatementType(_) = bill {
    //                 Some(Self::fetch_bill_transaction_breakdown(&cache_manager, &request_manager, account_number.clone(), bill.as_bill_interface().id_.to_string(), meter_manager, billing_timezone).await?)
    //             }
    //             else {
    //                 None
    //             };
    //             bill.print(transactions);
    //         }
    //     }
    //     Ok(())
    // }

    pub fn print_summary_line_headers() {
        println!("{:-^54} {:-^10} {:-^32} {:-^32} {:-^10}",
            "",
            "Balance",
            "Charges",
            "Credits",
            "Balance"
        );
        println!("{:-^10} {:-^10} {:-^10} {:-^10} {:-^10} {:-^10} {:-^10} {:-^10} {:-^10} {:-^10} {:-^10} {:-^10} {:-^10}", 
            "Date",
            "Ref",
            "From",
            "To",
            "Type",
            "b/f",
            "Net",
            "Tax",
            "Gross",
            "Net",
            "Tax",
            "Gross",
            "c/f"
        );
    }
    pub fn print_summary_line(&self) {
        let abstract_bill = self.as_bill_interface();

        print!("{:10} {:>10} {:10} {:10} {:10}", 
            abstract_bill.issued_date_,
            abstract_bill.id_,
            abstract_bill.from_date_,
            abstract_bill.to_date_,
            abstract_bill.bill_type_.as_str(),
        );

        match self {
            Bill::StatementType(statement) => {
                print!(" {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
                    as_decimal(statement.opening_balance_, 2),
                    as_decimal(statement.total_charges_.net_total_, 2),
                    as_decimal(statement.total_charges_.tax_total_, 2),
                    as_decimal(statement.total_charges_.gross_total_, 2),
                    as_decimal(statement.total_credits_.net_total_, 2),
                    as_decimal(statement.total_credits_.tax_total_, 2),
                    as_decimal(statement.total_credits_.gross_total_, 2),
                    as_decimal(statement.closing_balance_, 2)
                );
            },
            Bill::PreKrakenBillType(_) => {},
            Bill::PeriodBasedDocumentType(period_based_document) => {
                print!(" {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
                    "",
                    as_decimal(period_based_document.total_charges_.net_total_, 2),
                    as_decimal(period_based_document.total_charges_.tax_total_, 2),
                    as_decimal(period_based_document.total_charges_.gross_total_, 2),
                    as_decimal(period_based_document.total_credits_.net_total_, 2),
                    as_decimal(period_based_document.total_credits_.tax_total_, 2),
                    as_decimal(period_based_document.total_credits_.gross_total_, 2),
                    ""
                );
            },
            Bill::InvoiceType(invoice) => {
                print!(" {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
                    "",
                    "",
                    "",
                    "",
                    "",
                    "",
                    as_decimal(invoice.gross_amount_, 2),
                    "",
                );
            },
        }

        println!();
    }

     pub fn gui_summary_header() -> Element{
        rsx!{
            tr {
                style: "background: #666666;",

                th {
                    colspan: 5,
                    ""
                }
                th {
                    "Balance"
                }
                th {
                    colspan: 3,
                    "Charges"
                }
                th {
                    colspan: 3,
                    "Credits"
                }
                th {
                    "Balance"
                }
            }
            tr {
                style: "background: #666666;",
                th{"Date"}
                th{"Ref"}
                th{"From"}
                th{"To"}
                th{"Type"}
                th{"b/f"}
                th{"Net"}
                th{"Tax"}
                th{"Gross"}
                th{"Net"}
                th{"Tax"}
                th{"Gross"}
                th{"c/f"}
            }
        }
    }

    pub fn gui_summary_line(&self) -> Element {
        let abstract_bill = self.as_bill_interface();

        // let new_path = vec!(String::from("bills"), abstract_bill.id_.clone());
        // nav_callback(abstract_bill.id_.clone());

        // print!("{:10} {:>10} {:10} {:10} {:10}", 
        //     abstract_bill.issued_date_,
        //     abstract_bill.id_,
        //     abstract_bill.from_date_,
        //     abstract_bill.to_date_,
        //     abstract_bill.bill_type_.as_str(),
        // );

        let detail = match self {
            Bill::StatementType(statement) => {
                rsx!{
                    td { class: "numeric", "{as_decimal(statement.opening_balance_, 2)}" }
                    td { class: "numeric", "{as_decimal(statement.total_charges_.net_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(statement.total_charges_.tax_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(statement.total_charges_.gross_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(statement.total_credits_.net_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(statement.total_credits_.tax_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(statement.total_credits_.gross_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(statement.closing_balance_, 2)}" }
                }
            },
            Bill::PreKrakenBillType(_) => rsx!{},
            Bill::PeriodBasedDocumentType(period_based_document) => {
                rsx!{
                    td {  }
                    td { class: "numeric", "{as_decimal(period_based_document.total_charges_.net_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(period_based_document.total_charges_.tax_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(period_based_document.total_charges_.gross_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(period_based_document.total_credits_.net_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(period_based_document.total_credits_.tax_total_, 2)}" }
                    td { class: "numeric", "{as_decimal(period_based_document.total_credits_.gross_total_, 2)}" }
                    td {}
                }
            },
            Bill::InvoiceType(invoice) => {
                rsx!{
                    td {  }
                    td {  }
                    td {  }
                    td {  }
                    td {  }
                    td {  }
                    td { class: "numeric", "{as_decimal(invoice.gross_amount_, 2)}" }
                    td {  }
                }
            },
        };

        let id = abstract_bill.id_.clone();
        rsx!{
            tr {
                td { "{abstract_bill.issued_date_}" }
                td { 
                    div {
                        class: "link",
                        // onclick: |_| {path_signal.set(vec!(String::from("bills"), abstract_bill.id_.clone()))},
                        onclick: move |_| {
                            // println!("Id={}", id);
                            // let id = abstract_bill.id_.clone();
                            // nav_callback(id);

                            let mut path_signal = use_context::<Signal<Vec<String>>>();
                            let new_path = vec!(String::from("bills"),id.clone());
                            path_signal.set(new_path);
                        },
                        "{abstract_bill.id_}" }
                    }
                td { "{abstract_bill.from_date_}" }
                td { "{abstract_bill.to_date_}" }
                td { "{abstract_bill.bill_type_.as_str()}" }
                {detail}
            }
        }
    }

    pub fn gui_display(&self, transactions: &Vec<BillTransactionBreakDown>) -> Element {



        let mut total_charges = TotalCharges::new();
            BillTransaction::print_summary_line_headers();
            for transaction in transactions {
                transaction.print_summary_line(&mut total_charges);
            }

            if total_charges.units.is_positive() {





                println!("\nTOTALS");
                let rate = Decimal::from(total_charges.charge) / total_charges.units;

                print!("{:30} {:10} ", 
                    "Electricity Import",
                    ""
                );
                print!("{:>10} {:>10} {:>10} {:>10} ", 
                    "",
                    "", 
                    as_decimal(total_charges.charge, 2),
                    ""
                );
                print!("{:10} {:10} {:10} {:>12.4} ", 
                    "",
                    "",
                    "",
                    total_charges.units
                );
                print!("{:>10.3}", rate);
                println!();
            }
            println!();
            println!("Detailed Breakdown");
            println!("==================");
            total_charges = TotalCharges::new();
            
            for transaction in transactions {
                transaction.print(&mut total_charges);
            }













        let abstract_bill = self.as_bill_interface();

        rsx!{
            h1 {"Energy Account Statement"}
            table {
                tr {
                    th{"Date"}          td{ "{abstract_bill.issued_date_}" }
                    th{"Ref"}           td{ "{abstract_bill.id_}" }
                    th{"From"}          td{ "{abstract_bill.from_date_}" }
                    th{"To"}            td{ "{abstract_bill.to_date_}" }
                  }
            }
        }
    }

    pub fn print(&self, transactions: Option<Vec<BillTransactionBreakDown>>) {
        let abstract_bill = self.as_bill_interface();

        println!("Energy Account Statement");
        println!("========================");
        println!("Date                 {}", abstract_bill.issued_date_);
        println!("Ref                  {}", abstract_bill.id_);
        println!("From                 {}", abstract_bill.from_date_);
        println!("To                   {}", abstract_bill.to_date_);
        println!();

        if let Some(transactions) = transactions {
            let mut total_charges = TotalCharges::new();
            BillTransaction::print_summary_line_headers();
            for transaction in &transactions {
                transaction.print_summary_line(&mut total_charges);
            }

            if total_charges.units.is_positive() {





                println!("\nTOTALS");
                let rate = Decimal::from(total_charges.charge) / total_charges.units;

                print!("{:30} {:10} ", 
                    "Electricity Import",
                    ""
                );
                print!("{:>10} {:>10} {:>10} {:>10} ", 
                    "",
                    "", 
                    as_decimal(total_charges.charge, 2),
                    ""
                );
                print!("{:10} {:10} {:10} {:>12.4} ", 
                    "",
                    "",
                    "",
                    total_charges.units
                );
                print!("{:>10.3}", rate);
                println!();
            }
            println!();
            println!("Detailed Breakdown");
            println!("==================");
            total_charges = TotalCharges::new();
            
            for transaction in &transactions {
                transaction.print(&mut total_charges);
            }
        }
        
    }
}

pub struct TotalCharges {
    charge: i32,
    units: Decimal,
}

impl TotalCharges {
    fn new() -> Self {
        TotalCharges{
            charge: 0,
            units: Decimal::new(0, 0),
        }
    }
}

pub mod transaction;
pub mod list;




// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_bill_deserialize() {
//         let all_json = r#"{
//   "account": {
//     "bills": {
//       "edges": [
//         {
//           "node": {
//             "__typename": "StatementType",
//             "billType": "STATEMENT",
//             "closingBalance": 30711,
//             "fromDate": "2025-01-10",
//             "heldStatus": {
//               "isHeld": false,
//               "reason": null
//             },
//         "pageInfo": {
//           "endCursor": "YXJyYXljb25uZWN0aW9uOjE5",
//           "hasNextPage": true
//         }
//       }
//     }
//   }"#;
//         let response: super::super::graphql::bill::get_bills_and_transactions::Response = serde_json::from_str(all_json).unwrap();

//         serde_json::to_string_pretty(&response);
//     }
// }