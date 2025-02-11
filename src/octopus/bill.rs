use std::sync::Arc;

use sparko_graphql::AuthenticatedRequestManager;

use crate::octopus::decimal::Decimal;
use crate::util::as_decimal;

use super::graphql::latest_bill::get_account_latest_bill::{BillInterface, TransactionType};


use super::{token::OctopusTokenManager, Error};


impl BillInterface {
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
                print!(" {}", txn.note_);
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

pub struct BillManager {
    request_manager: Arc<AuthenticatedRequestManager<OctopusTokenManager>>,
}

impl BillManager {
    pub fn new(request_manager: Arc<AuthenticatedRequestManager<OctopusTokenManager>>) -> Self {
        Self {
            request_manager,
        }
    }

    pub async fn get_latest_bill(&mut self, account_number: &String)  -> Result<BillInterface, Error> {
        // let account_number = self.get_default_account().await?.number_;
        let query = super::graphql::latest_bill::get_account_latest_bill::Query::from(super::graphql::latest_bill::get_account_latest_bill::Variables::builder()
            .with_account_number(account_number.clone())
            .with_bills_first(1)
            .with_bills_transactions_first(100)
            .build()?
        );
        let mut response = self.request_manager.call(&query).await?;

        Ok(response.account_.bills_.edges_.remove(0).node_)
    }

    pub async fn print_transactions(&self, bill: &BillInterface) {
        
    }
}