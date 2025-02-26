use std::collections::HashMap;
use std::sync::Arc;

use sparko_graphql::AuthenticatedRequestManager;

use crate::octopus::decimal::Decimal;
use crate::octopus::meter::MeterType;
use crate::util::as_decimal;
use crate::CacheManager;

use super::graphql::bill;
use super::meter::MeterManager;
use bill::get_bills::BillInterface;
use bill::get_statement_transactions::TransactionType;
use super::graphql::BillTypeEnum;
use super::RequestManager;
use super::{token::OctopusTokenManager, Error};

pub struct BillManager {
    // pub account_number: String,
    pub cache_manager: Arc<CacheManager>,
    pub request_manager: Arc<RequestManager>,
    pub bills: HashMap<String, BillList>,
}

impl BillManager {
    pub fn new(cache_manager: &Arc<CacheManager>, request_manager: &Arc<RequestManager>)  -> Self {
        Self {
            // account_number,
            cache_manager: cache_manager.clone(),
            request_manager: request_manager.clone(),
            bills: HashMap::new(),
        }
    }

    pub async fn get_bills(&mut self, account_number: &String) -> Result<&BillList, Error> {
        Ok(self.bills.entry(account_number.clone()).or_insert(BillList::new(&self.cache_manager, &self.request_manager, account_number.clone(), crate::CHECK_FOR_UPDATES).await?))
    }

    // pub async fn get_statement_transactions(&self, account_number: String, statement_id: String)  -> Result<BillTransactionList, Error> {
    //     BillTransactionList::new(&self.cache_manager, &self.request_manager, account_number, statement_id).await
    // }

    async fn get_statement_transactions2(cache_manager: &Arc<CacheManager>, request_manager: &Arc<RequestManager>, account_number: String, statement_id: String)  -> Result<BillTransactionList, Error> {
        BillTransactionList::new(cache_manager, request_manager, account_number, statement_id).await
    }

    pub async fn bills_handler(&mut self, _args: std::str::SplitWhitespace<'_>, account_number: &String) ->  Result<(), Error> {
        self.get_bills(account_number).await?.print_summary_lines();
        Ok(())
    }


    pub async fn bill_handler(&mut self, mut args: std::str::SplitWhitespace<'_>, account_number: &String, meter_manager: &mut MeterManager) ->  Result<(), Error> {
        let one_hundred = Decimal::new(100, 0);
        let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
        let cache_manager = self.cache_manager.clone();
        let request_manager = self.request_manager.clone();
        let bills = self.get_bills(account_number).await?;

        if let Some(bill_id) = args.next() {
            for (_id, bill) in &bills.bills {
                if bill_id == bill.as_bill_interface().id_ {
                    let transactions = if let bill::get_bills::BillInterface::StatementType(_) = bill {


                        let transactions = Self::get_statement_transactions2(&cache_manager, &request_manager, account_number.clone(), bill_id.to_string()).await?;
                        for (_cursor, transaction) in &transactions.transactions {
                            if let TransactionType::Charge(charge) = transaction {
                                if let Some(consumption) = &charge.consumption_ {
                                    // print the line items making up this charge
                                    let line_item_map = meter_manager.get_line_items(account_number, &MeterType::Electricity, charge.is_export_, &consumption.start_date_, &consumption.end_date_).await?;

                                    // println!("got_line_items {}", line_item_map.len());

                                    for (agreement_id, line_items) in line_item_map {
                                        let mut total = Decimal::new(0,0);
                                        println!("Line items for agreement {}", agreement_id);
                                        println!("{:-^20} {:-^20} {:-^10} {:-^10} {:-^10}", "From", "To", "Amount", "Units", "p / unit");
                                
                                        for item in line_items {
                                            let amount = item.net_amount_ / one_hundred;

                                            total += amount;

                                            println!("{:20} {:20} {:10.3} {:10.3} {:10.3}", item.start_at_.format(&format).unwrap(), item.end_at_.format(&format).unwrap(), amount, item.number_of_units_, 
                                                if item.number_of_units_.is_non_zero() {item.net_amount_ / item.number_of_units_} else { item.net_amount_ }  );
                                        }
                                        println!("{:20} {:20} {:10.3}", "TOTAL", "", total);
                                    }

                                }
                            }
                        }

                        Some(transactions)
                    }
                    else {
                        None
                    };

                    bill.print(transactions);
                    return Ok(())
                }
            }
            println!("Unknown bill '{}'", bill_id);
        }
        else {
            if bills.bills.is_empty() {
                println!("There are no bills in this account");
            }
            else {
                let (_id, bill) = bills.bills.get(bills.bills.len() - 1).unwrap();
                let transactions = if let bill::get_bills::BillInterface::StatementType(_) = bill {
                    Some(Self::get_statement_transactions2(&cache_manager, &request_manager, account_number.clone(), bill.as_bill_interface().id_.to_string()).await?)
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

    pub fn print(&self, transactions: Option<BillTransactionList>) {
        let abstract_bill = self.as_bill_interface();

        println!("Energy Account Statement");
        println!("========================");
        println!("Date                 {}", abstract_bill.issued_date_);
        println!("Ref                  {}", abstract_bill.id_);
        println!("From                 {}", abstract_bill.from_date_);
        println!("To                   {}", abstract_bill.to_date_);
        println!();

        if let Some(transactions) = transactions {
            transactions.print();
        }
    }
}


pub struct BillList {
    pub account_number: String,
    pub start_cursor: Option<String>,
    pub has_previous_page: bool,
    pub bills: Vec<(String, BillInterface)>,
    hash_key: String,
}

impl BillList {
    pub fn print_summary_lines(&self) {
        BillInterface::print_summary_line_headers();

        for (_key, bill) in &self.bills {
            bill.print_summary_line();
        }
    }

    pub async fn fetch_all(&mut self, request_manager: &RequestManager)  -> Result<(), Error> {
        let mut has_previous_page = self.has_previous_page;

        println!("fetch_all bills {} in buffer", self.bills.len());

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

            println!("request for {} bills after {:?} returned {} bills", 20, self.start_cursor, response.account_.bills_.edges.len());

            if let Some(start_cursor) = response.account_.bills_.page_info.start_cursor {
                self.start_cursor = Some(start_cursor.clone());
                has_previous_page = response.account_.bills_.page_info.has_previous_page.clone();
            }
            else {
                has_previous_page = false;
            }

            for edge in response.account_.bills_.edges.into_iter().rev() {
                let sort_key = edge.cursor; //format!("{}#{}", &edge.node.as_bill_interface().issued_date_, &edge.cursor);
                self.bills.push((sort_key, edge.node));
            }
        }
        self.has_previous_page = has_previous_page;
        Ok(())
    }
    
   async fn new(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: String, check_for_updates: bool) -> Result<Self, Error> {
    let hash_key = format!("{}#Bills", account_number);

        let mut bills = Vec::new();

        cache_manager.read(&hash_key, &mut bills)?;

        let cached_cnt = bills.len();

        let mut result = if bills.is_empty() {
        
            let query = super::graphql::bill::get_bills::Query::builder()
                .with_account_number(account_number.clone())
                .with_last(1)
                .build()?;
            let response = request_manager.call(&query).await?;

            for edge in response.account_.bills_.edges {
                let sort_key = edge.cursor; //format!("{}#{}", &edge.node.as_bill_interface().issued_date_, &edge.cursor);
                bills.push((sort_key, edge.node));
            }

            BillList {
                account_number,
                start_cursor: response.account_.bills_.page_info.start_cursor,
                has_previous_page: response.account_.bills_.page_info.has_previous_page,
                bills,
                hash_key,
            }
        }
        else {
            let (start_cursor, _) = bills.get(bills.len() - 1).unwrap();
            BillList {
                account_number,
                start_cursor: Some(start_cursor.clone()),
                has_previous_page: true,
                bills,
                hash_key,
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
    pub transactions: Vec<(String, TransactionType)>,
    hash_key: String,
}

impl BillTransactionList {
    async fn new(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: String, statement_id: String) -> Result<Self, Error> {
        let hash_key = format!("{}#{}#StatementTransactions", account_number, statement_id);
    
            let mut transactions = Vec::new();
    
            cache_manager.read(&hash_key, &mut transactions)?;
    
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
                        let sort_key = edge.cursor; //format!("{}#{}", &edge.node.as_bill_interface().issued_date_, &edge.cursor);
                        transactions.push((sort_key, edge.node));
                    }
        
                    let mut result = BillTransactionList {
                        account_number,
                        statement_id,
                        start_cursor: statement.transactions_.page_info.start_cursor,
                        has_previous_page: statement.transactions_.page_info.has_previous_page,
                        transactions,
                        hash_key,
                    };

                    result.fetch_all(request_manager).await?;

                    result
                }
                else {
                    return Err(Error::StringError(format!("Bill {} is not a statement", statement_id)))
                }
            }
            else {
                let (start_cursor, _) = transactions.get(transactions.len() - 1).unwrap();
                BillTransactionList {
                    account_number,
                    statement_id,
                    start_cursor: Some(start_cursor.clone()),
                    has_previous_page: true,
                    transactions,
                    hash_key,
                }
            };
    
            // don't think this will ever be necessary but could be gated on check_for_updates
            // result.fetch_all(request_manager).await?;
    
            if result.transactions.len() > cached_cnt {
                cache_manager.write(&result.hash_key, &result.transactions, cached_cnt)?;
            }
            
            Ok(result)
        }

    pub fn print(&self) {
        
        print!("{:30} {:10} ", 
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

        for (_key, transaction) in &self.transactions {
        // for edge in (&statement.transactions_.edges).into_iter().rev() {
            let txn = transaction.as_transaction_type();

            if let TransactionType::Charge(charge) = &transaction {
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

            if let TransactionType::Charge(charge) = &transaction {
                print!("{:>10} {:>10} {:>10} {:>10} ", 
                    as_decimal(txn.amounts_.net_, 2),
                    as_decimal(txn.amounts_.tax_, 2), 
                    as_decimal(txn.amounts_.gross_, 2),
                    as_decimal(txn.balance_carried_forward_, 2)
                );
                if let Some(consumption) = &charge.consumption_ {
                    print!("{:10} {:10} {:>12.4} ", 
                        consumption.start_date_,
                        consumption.end_date_,
                        consumption.quantity_
                    );

                    let rate = if consumption.quantity_.is_non_zero() {Decimal::from(txn.amounts_.gross_) / consumption.quantity_} else {Decimal::new(0, 0)};

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
                else {
                    print!("{:47}","");
                }
            }
            else {
                print!("{:>10} {:>10} {:>10} {:>10} ", 
                    as_decimal(-txn.amounts_.net_, 2),
                    as_decimal(-txn.amounts_.tax_, 2), 
                    as_decimal(-txn.amounts_.gross_, 2),
                    as_decimal(txn.balance_carried_forward_, 2)
                );
                print!("{:47}","");
            }
            if let Some(note) = &txn.note_ {
                let note = note.trim();
                print!(" {}", note);
            }
            println!();
        }

        if total_electric_units.is_positive() {
            println!("\nTOTALS");
            let rate = Decimal::from(total_electric_charge) / total_electric_units;

            print!("{:30} {:10} ", 
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

    pub async fn fetch_all(&mut self, request_manager: &RequestManager)  -> Result<(), Error> {
        let mut has_previous_page = self.has_previous_page;

        println!("fetch_all statement transactions {} in buffer", self.transactions.len());

        

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
                println!("request for {} statement transactions after {:?} returned {} statement transactions", 100, self.start_cursor, statement.transactions_.len());

                self.start_cursor = statement.transactions_.page_info.start_cursor.clone();
                has_previous_page = statement.transactions_.page_info.has_previous_page.clone();

                for edge in statement.transactions_.edges.into_iter().rev() {
                    let sort_key = edge.cursor;
                    self.transactions.push((sort_key, edge.node));
                }
                
                println!("has_previous_page = {:?}", has_previous_page);
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