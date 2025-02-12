use std::fmt::Display;
use std::sync::Arc;

use sparko_graphql::AuthenticatedRequestManager;

use crate::octopus::decimal::Decimal;
use crate::octopus::graphql::bill::get_bills::TransactionType;
use crate::util::as_decimal;

use super::graphql::bill::get_bills::BillInterface;
use super::graphql::BillTypeEnum;
use super::RequestManager;
use super::{token::OctopusTokenManager, Error};

impl BillTypeEnum {
    fn as_str(&self) -> &'static str {
        match self {
            BillTypeEnum::Statement => "Statement",
            BillTypeEnum::Invoice => "Invoice",
            BillTypeEnum::CreditNote => "CreditNote",
            BillTypeEnum::PreKraken => "PreKraken",
        }
    }
}

impl BillInterface {
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
            BillInterface::StatementType(statement) => {
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
            BillInterface::PreKrakenBillType(abstract_bill_interface) => {},
            BillInterface::PeriodBasedDocumentType(period_based_document) => {
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
            BillInterface::InvoiceType(invoice) => {
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

    pub fn print(&self) {
        let abstract_bill = self.as_bill_interface();

        println!("Energy Account Statement");
        println!("========================");
        println!("Date                 {}", abstract_bill.issued_date_);
        println!("Ref                  {}", abstract_bill.id_);
        println!("From                 {}", abstract_bill.from_date_);
        println!("To                   {}", abstract_bill.to_date_);
        println!();

        if let BillInterface::StatementType(statement) = self {
            print!("{:20} {:10} ", 
                "Description",
                "Posted"
            );
            print!("{:>10} {:>10} {:>10} {:>10} ", 
                "Net",
                "Tax", 
                "Total",
                "Balance"
            );
            print!("{:10} {:10} {:>12} ", 
                "From",
                "To",
                "Units"
            );
            print!("{:>12}", "p / unit");
            println!();

            let mut total_electric_charge = 0;
            let mut total_electric_units = Decimal::new(0, 0);

            // for transaction in &mut map.values() {
            for edge in (&statement.transactions_.edges_).into_iter().rev() {
                let txn = edge.node_.as_transaction_type();

                if let TransactionType::Charge(charge) = &edge.node_ {
                    if charge.is_export_ {
                        print!("{} {:width$} ", txn.title_, "Export", width = 20 - txn.title_.len() - 1);
                    }
                    else {
                            print!("{:20} ", txn.title_);
                    }
                }
                else {
                    print!("{:20} ", txn.title_);
                }
                print!("{:10} ", 
                            txn.posted_date_
                        );
                print!("{:>10} {:>10} {:>10} {:>10} ", 
                    as_decimal(txn.amounts_.net_, 2),
                    as_decimal(txn.amounts_.tax_, 2), 
                    as_decimal(txn.amounts_.gross_, 2),
                    as_decimal(txn.balance_carried_forward_, 2)
                );
                if let TransactionType::Charge(charge) = &edge.node_ {
                    if let Some(consumption) = &charge.consumption_ {
                        print!("{:10} {:10} {:>12.4} ", 
                            consumption.start_date_,
                            consumption.end_date_,
                            consumption.quantity_
                        );

                        let rate = Decimal::from(txn.amounts_.gross_) / consumption.quantity_;

                        print!("{:>12.4}", rate); //.round_dp(2));

                        if charge.is_export_ {
                            
                        }
                        else {
                                if txn.title_.eq("Electricity") {
                                    total_electric_charge += *&txn.amounts_.gross_;
                                    total_electric_units += consumption.quantity_;
                                }
                            }
                    }
                }
                print!(" {:?}", txn.note_);
                println!();
            }

            println!("\nTOTALS");

            if total_electric_units.is_positive() {
                let rate = Decimal::from(total_electric_charge) / total_electric_units;

                print!("{:20} {:10} ", 
                    "Electricity Import",
                    ""
                );
                print!("{:>10} {:>10} {:>10} {:>10} ", 
                    "",
                    "", 
                    as_decimal(total_electric_charge, 2),
                    ""
                );
                print!("{:10} {:10} {:>12.4} ", 
                    "",
                    "",
                    total_electric_units
                );
                print!("{:>12.4}", rate);
                println!();
            }
        }
    }
}



// pub async fn get_latest_bill(request_manager: &RequestManager, account_number: &String)  -> Result<BillInterface, Error> {
//     // let account_number = self.get_default_account().await?.number_;
//     let query = super::graphql::latest_bill::get_account_latest_bill::Query::from(super::graphql::latest_bill::get_account_latest_bill::Variables::builder()
//         .with_account_number(account_number.clone())
//         .with_bills_first(1)
//         .with_bills_transactions_first(100)
//         .build()?
//     );
//     let mut response = request_manager.call(&query).await?;

//     Ok(response.account_.bills_.edges_.remove(0).node_)
// }

pub async fn get_bills(request_manager: &RequestManager, account_number: String, first: i32, transactions: i32)  -> Result<BillList, Error> {

    let query = super::graphql::bill::get_bills::Query::from(super::graphql::bill::get_bills::Variables::builder()
        .with_account_number(account_number.clone())
        .with_first(first)
        .with_transactions_first(transactions)
        .build()?
    );
    let response = request_manager.call(&query).await?;

    let mut bills = Vec::new();

    for edge in response.account_.bills_.edges_ {
        bills.push(edge.node_);
    }
    Ok(BillList {
        account_number,
        transactions,
        end_cursor: response.account_.bills_.page_info_.end_cursor_,
        has_next_page: response.account_.bills_.page_info_.has_next_page_,
        bills,
    })
}

pub struct BillList {
    pub account_number: String,
    pub transactions: i32,
    pub end_cursor: String,
    pub has_next_page: bool,
    pub bills: Vec<super::graphql::bill::get_bills::BillInterface>
}

impl BillList {
    pub fn print_summary_lines(&self) {
        BillInterface::print_summary_line_headers();

        for bill in &self.bills {
            bill.print_summary_line();
        }
    }

    pub async fn fetch_all(&mut self, request_manager: &RequestManager)  -> Result<(), Error> {
        let mut has_next_page = self.has_next_page;

        println!("fetch_all bills {} in buffer", self.bills.len());

        while has_next_page {
            let query = super::graphql::bill::get_bills::Query::from(super::graphql::bill::get_bills::Variables::builder()
                .with_account_number(self.account_number.clone())
                .with_first(20)
                .with_transactions_first(self.transactions)
                .with_after(self.end_cursor.clone())
                .build()?
            );
            let response = request_manager.call(&query).await?;

            println!("request for {} bills after {} returned {} bills", 20, self.end_cursor, response.account_.bills_.edges_.len());

            for edge in response.account_.bills_.edges_ {
                self.bills.push(edge.node_);
            }
            self.end_cursor = response.account_.bills_.page_info_.end_cursor_;
            has_next_page = response.account_.bills_.page_info_.has_next_page_;
            println!("has_next_page = {:?}", has_next_page);
        }
        self.has_next_page = has_next_page;
        Ok(())
    }
}



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
//         let response: super::super::graphql::bill::get_bills::Response = serde_json::from_str(all_json).unwrap();

//         serde_json::to_string_pretty(&response);
//     }
// }