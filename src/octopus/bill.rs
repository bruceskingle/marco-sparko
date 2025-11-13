use std::collections::HashMap;
use dioxus::prelude::*;
use indexmap::IndexMap;
use tokio::sync::Mutex;
use std::sync::Arc;

use anyhow::anyhow;

use sparko_graphql::AuthenticatedRequestManager;

use crate::cache_manager::Indexer;
use crate::octopus::decimal::Decimal;
use crate::octopus::meter::MeterType;
use crate::util::as_decimal;
use crate::CacheManager;

use super::graphql::{bill, meter};
use super::meter::{MeterManager, Tariff};
use bill::get_bills::BillInterface;
use bill::get_statement_transactions::TransactionType;
use super::graphql::BillTypeEnum;
use super::RequestManager;
use super::{token::OctopusTokenManager};

// const one_hundred: Decimal = Decimal::new(100, 0);
// const format: time::format_description = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

/**********************************************************************************
 * The manager manages an in-memory cache of Bills and related data objects.
 * 
 * The current "management" of this cache is to do nothing so our memory footprint will get bigger and bigger over time so we
 * need to do something about that.
 * 
 * 
 * Each Data Object is immutable but the types have methods which know how to fetch them so for each
 * fetch_XXX method on the manager the XXX data object has a fetch() method which takes additional parameters
 * which are e.g. the cache_manager and the request_manager. Those manager objects are themselves stateless.
 * 
 * Those data object methods use the local file system cache which currently has imperfect locking so that's something else
 * which needs fixing.
 * 
 * The Manager is stateless but contains Arc refs to managers which are needed to fetch data objects.
 * 
 * 
 * 
 */
pub struct BillManager {
    pub cache_manager: Arc<CacheManager>,
    pub request_manager: Arc<RequestManager>,
    meter_manager: Arc<MeterManager>,
    pub bills: Mutex<HashMap<String, Arc<BillList>>>,
}

impl BillManager {
    pub fn new(cache_manager: &Arc<CacheManager>, request_manager: &Arc<RequestManager>, meter_manager: &Arc<MeterManager>)  -> Self {
        Self {
            // account_number,
            cache_manager: cache_manager.clone(),
            request_manager: request_manager.clone(),
            meter_manager: meter_manager.clone(),
            bills: Mutex::new(HashMap::new()),
        }
    }

    pub async fn fetch_bills(&self, account_number: String) -> anyhow::Result<Arc<BillList>> {
        let mut locked_map = self.bills.lock().await;
        // let mut map = &*locked_map;
        Ok((&*locked_map
            .entry(account_number.clone())
            .or_insert(
                Arc::new(BillList::fetch(&self.cache_manager, &self.request_manager, &account_number, crate::CHECK_FOR_UPDATES).await?))).clone())
        // BillList::new(&self.cache_manager, &self.request_manager, &account_number, crate::CHECK_FOR_UPDATES).await
    }

    // pub async fn get_statement_transactions(&self, account_number: String, statement_id: String)  -> anyhow::Result<BillTransactionList> {
    //     BillTransactionList::new(&self.cache_manager, &self.request_manager, account_number, statement_id).await
    // }

    pub async fn fetch_bill_transaction_breakdown(&self, account_number: String, statement_id: String, billing_timezone: &time_tz::Tz)  -> anyhow::Result<Vec<BillTransactionBreakDown>> {


        let mut result = Vec::new();
        let transactions = BillTransactionList::new(&self.cache_manager, &self.request_manager, account_number.clone(), statement_id).await?;


        for (_key, (_cursor, transaction)) in transactions.transactions {
            if let TransactionType::Charge(charge) = &transaction {
                if let Some(consumption) = &charge.consumption_ {
                    // print the line items making up this charge
                    //println!("Get line items {:?} - {:?}",  &consumption.start_date_, &consumption.end_date_);

                    let meter_type = match transaction.as_transaction_type().title_.as_str() {
                        "Gas" => MeterType::Gas,
                        "Electricity" => MeterType::Electricity,
                        _ => panic!("Unknown consumption type")
                    };

                    let line_items = Some(self.meter_manager.get_line_items(&account_number, &meter_type, charge.is_export_, &consumption.start_date_, &consumption.end_date_, billing_timezone).await?);

                    result.push(BillTransactionBreakDown{
                        transaction,
                        line_items,
                    });
                    continue;
                }
            }
            result.push(BillTransactionBreakDown{
                transaction,
                line_items: None,
            });
        }

        Ok(result)
    }

    pub async fn bills_handler(&self, _args: std::str::SplitWhitespace<'_>, account_number: String) ->  anyhow::Result<()> {
        self.fetch_bills(account_number).await?.print_summary_lines();
        Ok(())
    }


    pub async fn bill_handler(&self, mut args: std::str::SplitWhitespace<'_>, account_number: String, meter_manager: &MeterManager, billing_timezone: &time_tz::Tz) ->  anyhow::Result<()> {
        // let one_hundred = Decimal::new(100, 0);
        // let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
        let cache_manager: Arc<CacheManager> = self.cache_manager.clone();
        let request_manager = self.request_manager.clone();
        let bills = self.fetch_bills(account_number.clone()).await?;

        if let Some(bill_id) = args.next() {
            for (_id, bill) in bills.bills.values() {
                if bill_id == bill.as_bill_interface().id_ {
                    let transactions = if let bill::get_bills::BillInterface::StatementType(_) = bill {


                        let transactions = self.fetch_bill_transaction_breakdown(account_number, bill_id.to_string(), billing_timezone).await?;

                        Some(transactions)
                    }
                    else {
                        None
                    };

                    bill.print(transactions);
                    return Ok(())
                }
            }
            //println!("Unknown bill '{}'", bill_id);
        }
        else {
            if bills.bills.is_empty() {
                //println!("There are no bills in this account");
            }
            else {
                let (_key, (_id, bill)) = bills.bills.get_index(bills.bills.len() - 1).unwrap();
                let transactions = if let bill::get_bills::BillInterface::StatementType(_) = bill {
                    Some(self.fetch_bill_transaction_breakdown(account_number.clone(), bill.as_bill_interface().id_.to_string(), billing_timezone).await?)
                }
                else {
                    None
                };
                bill.print(transactions);
            }
        }
        Ok(())
    }
}

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
    // pub async fn bill_gui_handler(&self, account_number: &String, meter_manager: &MeterManager, billing_timezone: &time_tz::Tz) ->  anyhow::Result<()> {
    //     // let one_hundred = Decimal::new(100, 0);
    //     // let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
    //     let cache_manager: Arc<CacheManager> = self.cache_manager.clone();
    //     let request_manager = self.request_manager.clone();
    //     let bills = self.get_bills(account_number).await?;

    //     if let Some(bill_id) = args.next() {
    //         for (_id, bill) in &bills.bills {
    //             if bill_id == bill.as_bill_interface().id_ {
    //                 let transactions = if let bill::get_bills::BillInterface::StatementType(_) = bill {


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
    //             let transactions = if let bill::get_bills::BillInterface::StatementType(_) = bill {
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
            BillInterface::PreKrakenBillType(_) => {},
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
            BillInterface::StatementType(statement) => {
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
            BillInterface::PreKrakenBillType(_) => rsx!{},
            BillInterface::PeriodBasedDocumentType(period_based_document) => {
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
            BillInterface::InvoiceType(invoice) => {
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
            TransactionType::print_summary_line_headers();
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
                transaction.print();
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

impl TransactionType {
    pub fn print_summary_line_headers() {
        print!("{:-^30} {:-^10} ", 
            "Description",
            "Posted"
        );
        print!("{:-^10} {:-^10} {:-^10} {:-^10} ", 
            "Net",
            "Tax", 
            "Total",
            "Balance"
        );
        println!("{:-^10} {:-^10} {:-^10} {:-^12} {:-^10}", "From", "To", "Amount", "Units", "p/unit");
    }

    pub fn print_break_down_line_headers() {
        println!("{:-^20} {:-^20} {:-^10} {:-^12} {:-^10}", "From", "To", "Amount", "Units", "p/unit");
    }

    pub fn print_summary_line(&self, total_charges: &mut TotalCharges) {
            let txn = self.as_transaction_type();

            if let TransactionType::Charge(charge) = &self {
                if charge.is_export_ {
                    print!("{} {:width$} ", txn.title_, "Export", width = 30 - txn.title_.len() - 1);
                }
                else {
                        print!("{:30} ", txn.title_);
                }
            }
            else {
                print!("{:30} ", txn.title_);
            }
            print!("{:10} ", 
                        txn.posted_date_
                    );

            if let TransactionType::Charge(charge) = &self {
                print!("{:>10} {:>10} {:>10} {:>10} ", 
                    as_decimal(txn.amounts_.net_, 2),
                    as_decimal(txn.amounts_.tax_, 2), 
                    as_decimal(txn.amounts_.gross_, 2),
                    as_decimal(txn.balance_carried_forward_, 2)
                );
                if let Some(consumption) = &charge.consumption_ {
                    print!("{:10} {:10} {:>10} {:>12.4} ", 
                        consumption.start_date_,
                        consumption.end_date_,
                        as_decimal(txn.amounts_.net_, 3),
                        consumption.quantity_
                    );

                    let rate = if consumption.quantity_.is_non_zero() {Decimal::from(txn.amounts_.gross_) / consumption.quantity_} else {Decimal::new(0, 0)};

                    print!("{:>10.3}", rate); //.round_dp(2));

                    if charge.is_export_ {
                        
                    }
                    else {
                            if txn.title_.eq("Electricity") {
                                total_charges.charge += *&txn.amounts_.gross_;
                                total_charges.units += consumption.quantity_;
                            }
                        }
                }
                else {
                    print!("{:56}","");
                }
            }
            else {
                print!("{:>10} {:>10} {:>10} {:>10} ", 
                    as_decimal(-txn.amounts_.net_, 2),
                    as_decimal(-txn.amounts_.tax_, 2), 
                    as_decimal(-txn.amounts_.gross_, 2),
                    as_decimal(txn.balance_carried_forward_, 2)
                );
                print!("{:56}","");
            }
            if let Some(note) = &txn.note_ {
                let note = note.trim();
                print!(" {}", note);
            }
            println!();
    }
}

pub struct BillTransactionBreakDown {
    transaction: TransactionType,
    line_items: Option<IndexMap<String, (Tariff, Vec<meter::electricity_agreement_line_items::LineItemType>)>>,
}

impl BillTransactionBreakDown {
    pub fn print_summary_line(&self, total_charges: &mut TotalCharges) {
        self.transaction.print_summary_line(total_charges);
    }

    pub fn print(&self) {
        let one_hundred = Decimal::new(100, 0);
        let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
        let time_format = time::format_description::parse("           [hour]:[minute]:[second]").unwrap();

        if let Some(line_item_map) = &self.line_items {



            for (agreement_id, (tariff, line_items)) in line_item_map {

                let mut amount_map = IndexMap::new();
                let mut total_amount = Decimal::new(0,0);
                let mut total_units = Decimal::new(0,0);

                println!();
                tariff.print();
                println!();

                TransactionType::print_break_down_line_headers();
                // println!("Line items for agreement {}", agreement_id);
                // println!("{:-^20} {:-^20} {:-^10} {:-^10} {:-^10}", "From", "To", "Amount", "Units", "p / unit");
        
                let mut prev = None;
                for item in line_items {
                    let amount = item.net_amount_ / one_hundred;

                    total_amount += amount;
                    total_units += item.number_of_units_;

                    let unit_cost = if item.number_of_units_.is_non_zero() {item.net_amount_ / item.number_of_units_} else { item.net_amount_ };


                    println!("{:20} {:20} {:10.3} {:12.4} {:10.3}", 
                        if let Some(prev) = prev {
                            if prev == item.start_at_.date() {
                                item.start_at_.format(&time_format).unwrap()
                            } else {
                                item.start_at_.format(&format).unwrap()
                            }
                        } else {
                            item.start_at_.format(&format).unwrap()
                        },
                        if item.end_at_.date() == item.start_at_.date() {item.end_at_.format(&time_format).unwrap()} else {item.end_at_.format(&format).unwrap()},
                             amount, item.number_of_units_, 
                        &unit_cost  );
                    
                    if item.number_of_units_.is_positive() {
                        let key = format!("{:.2}", unit_cost);
                        if let Some((total_amount, total_units)) = amount_map.get(&key) {
                            amount_map.insert(key, (amount + *total_amount, item.number_of_units_ + *total_units));
                        }
                        else {
                            amount_map.insert(key, (amount, item.number_of_units_));
                        }
                    }
                    
                    prev = Some(item.start_at_.date());
                }
                println!("{:41} {:10.3} {:12.4}", "Total Consumption", total_amount, total_units);
                if line_items.len() > 0 {
                    let start_date = line_items.get(0).unwrap().start_at_.date();
                    let end_date = line_items.get(line_items.len() - 1).unwrap().end_at_.date();
                    let days = end_date.to_julian_day() - start_date.to_julian_day();
                    let standing_charge = Decimal::new((tariff.standing_charge() * (10000 * days) as f64) as i64,6);
                    println!("{:41} {:10.3}", format!("Standing charge ({} days @ {:.3})", days,tariff.standing_charge()) , standing_charge);
                    println!("{:41} {:10.3}", "Total", total_amount + standing_charge);
            
        
                    let txn = self.transaction.as_transaction_type();
                    print!("{:30} {:10} ", "as shown on bill", "");
            
                    if let TransactionType::Charge(charge) = &self.transaction {
                        print!("{:>9}  ", as_decimal(txn.amounts_.net_, 2));
                        if let Some(consumption) = &charge.consumption_ {
                            print!("{:>12.4} ",consumption.quantity_);
            
                            let rate = if consumption.quantity_.is_non_zero() {Decimal::from(txn.amounts_.gross_) / consumption.quantity_} else {Decimal::new(0, 0)};
            
                            print!("{:>10.3}", rate);
                        }
                        println!("");
                    }
                    println!("");
                    println!("Analysis");
                    println!("--------");
            
                    if !amount_map.is_empty() {
                        println!("{:-^15} {:-^10} {:-^10} {:-^10} {:-^10} {:-^10}", "Unit Rate", "Cost", "Units", "% Cost", "% Units", "% Bill");
                        for (key, (amount, units)) in amount_map {
                            println!("{:>15} {:10.2} {:10.2} {:10.2} {:10.2} {:10.2}",
                                key,
                                amount,
                                units,
                                one_hundred * amount / total_amount,
                                one_hundred * units / total_units,
                                one_hundred * amount / (standing_charge + total_amount)
                            );
                        }
                        println!("{:60}{:10.2}",
                            "Standing Charge",
                            one_hundred * standing_charge / (standing_charge + total_amount)
                        );
                    }
                    println!("");
                    println!("");
                    println!("");
                }
            }
        }

    }
}

// static BILL_INDEXER: Indexer<BillInterface> = Box::new(|bill: &BillInterface| bill.as_bill_interface().id_.clone());

pub struct BillList {
    pub account_number: String,
    pub start_cursor: Option<String>,
    pub has_previous_page: bool,
    pub bills: IndexMap<String, (String, BillInterface)>,
    hash_key: String,
    indexer: Indexer<BillInterface>,
}

impl BillList {
    pub fn print_summary_lines(&self) {
        BillInterface::print_summary_line_headers();

        for (_bill_id, (_key, bill)) in &self.bills {
            bill.print_summary_line();
        }
    }

    pub async fn fetch_all(&mut self, request_manager: &RequestManager)  -> anyhow::Result<()> {
        let mut has_previous_page = self.has_previous_page;

        //println!("fetch_all bills {} in buffer", self.bills.len());

        while has_previous_page 
        {
            let mut builder = super::graphql::bill::get_bills::Query::builder()
            .with_account_number(self.account_number.clone())
            .with_last(20);

            if let Some(start_cursor) = &self.start_cursor {
                builder = builder.with_before(start_cursor.clone())
            }

            let query = builder.build()?;
            // let query = super::graphql::bill::get_bills::Query::from(builder.build()?);
            let response = request_manager.call(&query).await?;

            //println!("request for {} bills after {:?} returned {} bills", 20, self.start_cursor, response.account_.bills_.edges.len());

            if let Some(start_cursor) = response.account_.bills_.page_info.start_cursor {
                self.start_cursor = Some(start_cursor.clone());
                has_previous_page = response.account_.bills_.page_info.has_previous_page.clone();
            }
            else {
                has_previous_page = false;
            }
            let indexer = &self.indexer;

            for edge in response.account_.bills_.edges.into_iter().rev() {
                let sort_key = edge.cursor; //format!("{}#{}", &edge.node.as_bill_interface().issued_date_, &edge.cursor);
                self.bills.insert(indexer(&edge.node), (sort_key, edge.node));
            }
        }
        self.has_previous_page = has_previous_page;
        Ok(())
    }
    
   async fn fetch(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: &String, check_for_updates: bool) -> anyhow::Result<Self> {
    let hash_key = format!("{}#Bills", account_number);

    let account_number = account_number.clone();
        let mut bills = IndexMap::new();

        let indexer: Indexer<BillInterface> = Box::new(|bill: &BillInterface| bill.as_bill_interface().id_.clone());;

        cache_manager.read(&hash_key, &mut bills, &indexer)?;

        let cached_cnt = bills.len();

        let mut result = if bills.is_empty() {
        
            let query = super::graphql::bill::get_bills::Query::builder()
                .with_account_number(account_number.clone())
                .with_last(1)
                .build()?;
            let response = request_manager.call(&query).await?;

            for edge in response.account_.bills_.edges {
                let sort_key = edge.cursor; //format!("{}#{}", &edge.node.as_bill_interface().issued_date_, &edge.cursor);
                bills.insert(indexer(&edge.node), (sort_key, edge.node));
            }

            BillList {
                account_number,
                start_cursor: response.account_.bills_.page_info.start_cursor,
                has_previous_page: response.account_.bills_.page_info.has_previous_page,
                bills,
                hash_key,
                indexer,
            }
        }
        else {
            let (key, (start_cursor, _)) = bills.get_index(bills.len() - 1).unwrap();
            BillList {
                account_number,
                start_cursor: Some(start_cursor.clone()),
                has_previous_page: true,
                bills,
                hash_key,
                indexer,
            }
        };

        if check_for_updates {
            result.fetch_all(request_manager).await?;
        }

        if result.bills.len() > cached_cnt {
            cache_manager.write(&result.hash_key, &result.bills, cached_cnt)?;
        }
        
        Ok(result)
    }
}


pub struct BillTransactionList {
    pub account_number: String,
    pub statement_id: String,
    pub start_cursor: Option<String>,
    pub has_previous_page: bool,
    pub transactions: IndexMap<String, (String, TransactionType)>,
    hash_key: String,
    indexer: Indexer<TransactionType>,
}

impl BillTransactionList {
    async fn new(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: String, statement_id: String) -> anyhow::Result<Self> {
        let hash_key = format!("{}#{}#StatementTransactions", account_number, statement_id);
            let indexer: Indexer<TransactionType> = Box::new(|txn: &TransactionType| txn.as_transaction_type().id_.clone());;
            let mut transactions = IndexMap::new();
    
            cache_manager.read(&hash_key, &mut transactions, &indexer)?;
    
            let cached_cnt = transactions.len();
    
            let mut result = if transactions.is_empty() {
                
                let query = super::graphql::bill::get_statement_transactions::Query::builder()
                        .with_account_number(account_number.clone())
                        .with_statement_id(statement_id.clone())
                        .with_transactions_last(1)
                        .build()?;
                let response = request_manager.call(&query).await?;
                let bill = response.account_.bill_;

                if let bill::get_statement_transactions::BillInterface::StatementType(statement) = bill {

                    for edge in statement.transactions_.edges {
                        let key = indexer(&edge.node);
                        let sort_key = edge.cursor; //format!("{}#{}", &edge.node.as_bill_interface().issued_date_, &edge.cursor);
                        transactions.insert(key, (sort_key, edge.node));
                    }
        
                    let mut result = BillTransactionList {
                        account_number,
                        statement_id,
                        start_cursor: statement.transactions_.page_info.start_cursor,
                        has_previous_page: statement.transactions_.page_info.has_previous_page,
                        transactions,
                        hash_key,
                        indexer,
                    };

                    result.fetch_all(request_manager).await?;

                    result
                }
                else {
                    return Err(anyhow!(format!("Bill {} is not a statement", statement_id)))
                }
            }
            else {
                let (key, (start_cursor, _)) = transactions.get_index(transactions.len() - 1).unwrap();
                BillTransactionList {
                    account_number,
                    statement_id,
                    start_cursor: Some(start_cursor.clone()),
                    has_previous_page: true,
                    transactions,
                    hash_key,
                    indexer,
                }
            };
    
            // don't think this will ever be necessary but could be gated on check_for_updates
            // result.fetch_all(request_manager).await?;
    
            if result.transactions.len() > cached_cnt {
                cache_manager.write(&result.hash_key, &result.transactions, cached_cnt)?;
            }
            
            Ok(result)
        }

    pub async fn fetch_all(&mut self, request_manager: &RequestManager)  -> anyhow::Result<()> {
        let mut has_previous_page = self.has_previous_page;

        //println!("fetch_all statement transactions {} in buffer", self.transactions.len());

        

        while has_previous_page {
            let mut builder = super::graphql::bill::get_statement_transactions::Query::builder()
                .with_account_number(self.account_number.clone())
                .with_statement_id(self.statement_id.clone())
                .with_transactions_first(100);

            if let Some(end_cursor) = &self.start_cursor {
                builder = builder.with_transactions_before(end_cursor.clone());
            }
            let query = //super::graphql::bill::get_statement_transactions::Query::from(
                builder.build()?;
            let response = request_manager.call(&query).await?;

            
            if let super::graphql::bill::get_statement_transactions::BillInterface::StatementType(statement) = response.account_.bill_ {
                //println!("request for {} statement transactions after {:?} returned {} statement transactions", 100, self.start_cursor, statement.transactions_.len());

                self.start_cursor = statement.transactions_.page_info.start_cursor.clone();
                has_previous_page = statement.transactions_.page_info.has_previous_page.clone();

                for edge in statement.transactions_.edges.into_iter().rev() {
                    let sort_key = edge.cursor;
                    let key = (self.indexer)(&edge.node);
                    self.transactions.insert(key, (sort_key, edge.node));
                }
                
                //println!("has_previous_page = {:?}", has_previous_page);
            }
        }
        self.has_previous_page = has_previous_page;
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
//         let response: super::super::graphql::bill::get_bills_and_transactions::Response = serde_json::from_str(all_json).unwrap();

//         serde_json::to_string_pretty(&response);
//     }
// }