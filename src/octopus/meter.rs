use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use sparko_graphql::types::{Date, DateTime, EdgeOf, PageInfo};
use sparko_graphql::AuthenticatedRequestManager;

use crate::octopus::decimal::Decimal;
use crate::util::as_decimal;
use crate::CacheManager;

use super::graphql::meter;
use super::RequestManager;
use super::{token::OctopusTokenManager, Error};

pub enum MeterType {
    Gas,
    Electricity
}

impl Display for MeterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeterType::Gas => write!(f, "Gas"),
            MeterType::Electricity => write!(f, "Electricity"),
        }
    }
}

pub struct MeterManager {
    // pub account_number: String,
    pub cache_manager: Arc<CacheManager>,
    pub request_manager: Arc<RequestManager>,
    pub properties: HashMap<String, PropertyList>,
    pub agreements: HashMap<String,MeterAgreementList>,
}

//   Rita the
impl MeterManager {
    pub fn new(cache_manager: &Arc<CacheManager>, request_manager: &Arc<RequestManager>)  -> Self {
        // let properties = PropertyList::new(cache_manager, request_manager, account_number.clone()).await?;
        // let agreements = MeterAgreementList::new(cache_manager, request_manager, account_number.clone(), &properties.meter_node_ids).await?;

        Self {
            // account_number,
            cache_manager: cache_manager.clone(),
            request_manager: request_manager.clone(),
            properties: HashMap::new(),
            agreements: HashMap::new(),
        }
    }

    pub async fn get_line_items(&mut self, account_number: &String, meter_type: &MeterType, is_export: bool, start_date: &Date, end_date: &Date) -> Result<HashMap<String, Vec<meter::electricity_agreement_line_items::LineItemType>>, Error>{
        if let std::collections::hash_map::Entry::Vacant(entry) = self.properties.entry(account_number.clone()) {
            entry.insert(PropertyList::new(&self.cache_manager, &self.request_manager, account_number.clone()).await?);
        }
        
        let properties = self.properties.get(account_number).unwrap();
        let meter_agreements = MeterAgreementList::new(&self.cache_manager, &self.request_manager, account_number.clone(), &properties.meter_node_ids).await?;

        // println!("Meter Agreements");
        // for (cursor, agreement_vec) in &meter_agreements.electricity_map {
        //     println!("Cursor {}", cursor);
        //     for agreement in agreement_vec {
        //         println!("    {}", serde_json::to_string_pretty(&agreement)?)
        //     }
        // }

        let start_date_time = start_date.at_midnight();
        let end_date_time = end_date.at_next_midnight();

        let in_scope_agreements = meter_agreements.get_in_scope(meter_type, is_export, &start_date_time, &end_date_time);

        async fn get_line_items2(
            cache_manager: &CacheManager, request_manager: &RequestManager,
            account_number: &String, meter_type: &MeterType, agreement_id: &String, start_date: &Date,
            start_date_time: &DateTime, end_date_time: &DateTime) -> Result<Vec<meter::electricity_agreement_line_items::LineItemType>, Error> {
                let mut in_scope_items = Vec::new();
                let mut bucket_date = start_date_time.to_date();
                loop {
                    let line_items = AgreementLineItems::new(cache_manager, request_manager, account_number.clone(), meter_type, agreement_id.clone(), &bucket_date).await?;

                    for (_cursor, item) in line_items.line_items {
                        if &item.start_at_ >= end_date_time {
                            // thats it
                            return Ok(in_scope_items)
                        }
                        if &item.start_at_ >= start_date_time {
                            in_scope_items.push(item);
                        }
                    }

                    bucket_date = line_items.end_date;
                }
                // Ok(in_scope_items)



        //     // TODO this is the whole months worth of line items
        //     let line_items = AgreementLineItems::new(&self.cache_manager, &self.request_manager, account_number.clone(), meter_type, agreement_id, start_date).await?;

        //     for (cursor, item) in &line_items.line_items {
        //         if item.start_at_ >= end_date_time {
        //             // thats it
        //             return Ok(())
        //         }
        //         if item.start_at_ >= start_date_time {
        //             result.push(item);
        //         }
        //     }
        }
        
        let mut item_map = HashMap::new();

        for agreement_id in in_scope_agreements {
            let agreement_id = agreement_id.to_string(); // Ugh!

            let in_scope_items = get_line_items2(&self.cache_manager, &self.request_manager, account_number, meter_type, &agreement_id, start_date, &start_date_time, &end_date_time).await?;
            item_map.insert(agreement_id, in_scope_items);
        }

        // self.properties.entry(account_number.clone()).or_insert_with(||  PropertyList::new(&self.cache_manager, &self.request_manager, account_number.clone()).await?)
        // first get the properties belonging to the account
        // let properties = PropertyList::new(&self.cache_manager, &self.request_manager, account_number.clone()).await?;

        // Now find all of the in scope meters

        // for property in &properties.properties.account_.properties_ {
        //     match meter_type {
        //         MeterType::Gas => todo!(),
        //         MeterType::Electricity => {
        //             for meter_point in property.electricity_meter_points_ {
        //                 for meter in meter_point.meters_ {
        //                     println("Meter {}", meter.node_id_);
        //                 }
        //             }
        //         },
        //     }
            
        // }

        

        Ok(item_map)
    }

    fn in_scope(from: &Date, to: &Option<Date>, valid_from: &DateTime, valid_to: &Option<DateTime>) -> bool {
        let end_in_scope = if let Some(to) = to {
            // if let Some(valid_from) = valid_from {
                valid_from.to_date() <= *to
            // }
            // else {
            //     false
            // }
        }
        else {
            true
        };

        let start_in_scope = if let Some(valid_to) = valid_to {
                valid_to.to_date() >= *from

        }
        else {
            true
        };
        
        // println!("in_scope({:?},{:?},{:?},{:?}) = {}",
        //     from,
        //     to,
        //     valid_from,
        //     valid_to,
        //     end_in_scope && start_in_scope
        // );
        end_in_scope && start_in_scope
    }
    
    // pub async fn get_electric_line_items(&self,
    //     from: &Date,
    //     to: &Option<Date>) {
    //         for (meter_node_id, agreement) in self.agreements.electricity_map {
    //             if Self::in_scope(&from, &to, &agreement.valid_from_, &agreement.valid_to_) {
    //                 // let agreement_id = Self::unexpected_none()?.to_string();
    //                 // println!("Electricity agreement {}", &agreement.id_);

    //                 self.get_electric_line_items(format!("{}", &agreement.id_), from, to).await?;

    //             }
    //         }
    // }

    // pub async fn bills_handler(&mut self, _args: std::str::SplitWhitespace<'_>) ->  Result<(), Error> {
    //     self.bills.print_summary_lines();
    //     Ok(())
    // }


    // pub async fn bill_handler(&mut self, mut args: std::str::SplitWhitespace<'_>) ->  Result<(), Error> {
    //     if let Some(bill_id) = args.next() {
    //         for (_id, bill) in &self.bills.bills {
    //             if bill_id == bill.as_bill_interface().id_ {
    //                 let transactions = if let bill::get_bills::BillInterface::StatementType(_) = bill {
    //                     Some(self.get_statement_transactions(self.account_number.clone(), bill_id.to_string()).await?)
    //                 }
    //                 else {
    //                     None
    //                 };

    //                 bill.print(transactions);
    //                 return Ok(())
    //             }
    //         }
    //         println!("Unknown bill '{}'", bill_id);
    //     }
    //     else {
    //         if self.bills.bills.is_empty() {
    //             println!("There are no bills in this account");
    //         }
    //         else {
    //             let (_id, bill) = self.bills.bills.get(self.bills.bills.len() - 1).unwrap();
    //             let transactions = if let bill::get_bills::BillInterface::StatementType(_) = bill {
    //                 Some(self.get_statement_transactions(self.account_number.clone(), bill.as_bill_interface().id_.to_string()).await?)
    //             }
    //             else {
    //                 None
    //             };
    //             bill.print(transactions);
    //         }
    //     }
    //     Ok(())
    // }
}

pub struct PropertyList {
    pub properties: meter::account_properties_meters::Response,
    pub meter_node_ids: Vec<String>,
    hash_key: String,
}

impl PropertyList {
    // pub fn print_summary_lines(&self) {
    //     BillInterface::print_summary_line_headers();

    //     for (_key, bill) in &self.bills {
    //         bill.print_summary_line();
    //     }
    // }

    // pub async fn fetch_all(&mut self, request_manager: &RequestManager)  -> Result<(), Error> {
    //     let mut has_previous_page = self.has_previous_page;

    //     println!("fetch_all bills {} in buffer", self.bills.len());

    //     while has_previous_page 
    //     {
    //         let mut builder = super::graphql::bill::get_bills::Variables::builder()
    //         .with_account_number(self.account_number.clone())
    //         .with_last(20);

    //         if let Some(start_cursor) = &self.start_cursor {
    //             builder = builder.with_before(start_cursor.clone())
    //         }

            
    //         let query = super::graphql::bill::get_bills::Query::from(builder.build()?);
    //         let response = request_manager.call(&query).await?;

    //         println!("request for {} bills after {:?} returned {} bills", 20, self.start_cursor, response.account_.bills_.edges.len());

    //         if let Some(start_cursor) = response.account_.bills_.page_info.start_cursor {
    //             self.start_cursor = Some(start_cursor.clone());
    //             has_previous_page = response.account_.bills_.page_info.has_previous_page.clone();
    //         }
    //         else {
    //             has_previous_page = false;
    //         }

    //         for edge in response.account_.bills_.edges.into_iter().rev() {
    //             let sort_key = edge.cursor; //format!("{}#{}", &edge.node.as_bill_interface().issued_date_, &edge.cursor);
    //             self.bills.push((sort_key, edge.node));
    //         }
    //     }
    //     self.has_previous_page = has_previous_page;
    //     Ok(())
    // }
    
   async fn new(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: String) -> Result<Self, Error> {
    let hash_key = format!("{}#Properties", account_number);

        let opt_properties: Option<meter::account_properties_meters::Response> = cache_manager.read_one(&hash_key)?;

        let properties = if let Some(properties) = opt_properties {
            properties
        }
        else {
            let query = meter::account_properties_meters::Query::builder()
                .with_account_number(account_number.clone())
                .build()?;
            let properties = request_manager.call(&query).await?;

            cache_manager.write_one(&hash_key, &properties)?;

            properties
        };

        let mut meter_node_ids: Vec<String> = Vec::new();

        for property in &properties.account_.properties_ {
            for point in &property.electricity_meter_points_ {
                for meter in &point.meters_ {
                    meter_node_ids.push(meter.node_id_.clone());
                }
            }
            for point in &property.gas_meter_points_ {
                for meter in &point.meters_ {
                    meter_node_ids.push(meter.node_id_.clone());
                }
            }
        }

        Ok(PropertyList {
            properties,
            meter_node_ids,
            hash_key,
        })
    }
}


pub struct MeterAgreementList {
    pub account_number: String,
    pub import_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
    pub export_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
    pub gas_map: HashMap<String, Vec<meter::meter_agreements::GasAgreementType>>,
    hash_key: String,
}

impl MeterAgreementList {
    async fn new(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: String, meter_node_ids: &Vec<String>) -> Result<Self, Error> {
        let hash_key = format!("{}#MeterAgreements", account_number);
            let the_beginning: DateTime = DateTime::from_calendar_date(2000, time::Month::January, 1)?;
            let mut agreements = Vec::new();
    
            cache_manager.read(&hash_key, &mut agreements)?;
    
            let cached_cnt = agreements.len();
    
            if agreements.is_empty() {
                for meter_node_id in meter_node_ids {
                    let query = meter::meter_agreements::Query::builder()
                            .with_meter_node_id(meter_node_id.clone())
                            .with_valid_after(the_beginning.clone())
                            .build()?;
                    let response = request_manager.call(&query).await?;

                    agreements.push((meter_node_id.clone(), response));
                }
                cache_manager.write(&hash_key, &agreements, cached_cnt)?;
            }

            let mut export_electricity_map = HashMap::new();
            let mut import_electricity_map = HashMap::new();
            let mut gas_map = HashMap::new();

            for (meter_node_id, response) in agreements {
                match response.node_ {
                    meter::meter_agreements::Node::ElectricityMeterType(electricity_meter_type) => {
                        if electricity_meter_type.import_meter_.is_some() {
                            export_electricity_map.insert(meter_node_id, electricity_meter_type.meter_point_.agreements_);
                        }
                        else {
                            import_electricity_map.insert(meter_node_id, electricity_meter_type.meter_point_.agreements_);
                        }
                    },
                    meter::meter_agreements::Node::GasMeterType(gas_meter_type) => {
                        gas_map.insert(meter_node_id, gas_meter_type.meter_point_.agreements_);
                    },
                    _ => unreachable!()
                }
            }
            
            Ok(MeterAgreementList {
                account_number,
                export_electricity_map,
                import_electricity_map,
                gas_map,
                hash_key,
            })
        }

    fn get_in_scope(&self, meter_type: &MeterType, is_export: bool, start_date: &DateTime, end_date: &DateTime) -> Vec<i32> {
        let mut in_scope_agreements = Vec::new();

        match meter_type {
            MeterType::Gas => {
                for agreement_vec in self.gas_map.values() {
                    for agreement in agreement_vec {
                        if &agreement.valid_from_ <= end_date {
                            if let Some(valid_to) = &agreement.valid_to_ {
                                if valid_to >= start_date {
                                    in_scope_agreements.push(agreement.id_.clone());
                                }
                            }
                            else {
                                in_scope_agreements.push(agreement.id_.clone());
                            }
                        }
                    }
                }
            },
            MeterType::Electricity => {
                for agreement_vec in if is_export {self.export_electricity_map.values()} else {self.import_electricity_map.values()} {
                    for agreement in agreement_vec {
                        if &agreement.valid_from_ <= end_date {
                            if let Some(valid_to) = &agreement.valid_to_ {
                                if valid_to >= start_date {
                                    in_scope_agreements.push(agreement.id_.clone());
                                }
                            }
                            else {
                                in_scope_agreements.push(agreement.id_.clone());
                            }
                        }
                    }
                }
            },
        }

        in_scope_agreements
    }
    
    // pub fn print(&self) {
        
    //     print!("{:30} {:10} ", 
    //         "Description",
    //         "Posted"
    //     );
    //     print!("{:>10} {:>10} {:>10} {:>10} ", 
    //         "Net",
    //         "Tax", 
    //         "Total",
    //         "Balance"
    //     );
    //     print!("{:10} {:10} {:>12} ", 
    //         "From",
    //         "To",
    //         "Units"
    //     );
    //     print!("{:>12}", "p / unit");
    //     println!();

    //     let mut total_electric_charge = 0;
    //     let mut total_electric_units = Decimal::new(0, 0);

    //     for (_key, transaction) in &self.agreements {
    //     // for edge in (&statement.transactions_.edges).into_iter().rev() {
    //         let txn = transaction.as_transaction_type();

    //         if let TransactionType::Charge(charge) = &transaction {
    //             if charge.is_export_ {
    //                 print!("{} {:width$} ", txn.title_, "Export", width = 30 - txn.title_.len() - 1);
    //             }
    //             else {
    //                     print!("{:30} ", txn.title_);
    //             }
    //         }
    //         else {
    //             print!("{:30} ", txn.title_);
    //         }
    //         print!("{:10} ", 
    //                     txn.posted_date_
    //                 );

    //         if let TransactionType::Charge(charge) = &transaction {
    //             print!("{:>10} {:>10} {:>10} {:>10} ", 
    //                 as_decimal(txn.amounts_.net_, 2),
    //                 as_decimal(txn.amounts_.tax_, 2), 
    //                 as_decimal(txn.amounts_.gross_, 2),
    //                 as_decimal(txn.balance_carried_forward_, 2)
    //             );
    //             if let Some(consumption) = &charge.consumption_ {
    //                 print!("{:10} {:10} {:>12.4} ", 
    //                     consumption.start_date_,
    //                     consumption.end_date_,
    //                     consumption.quantity_
    //                 );

    //                 let rate = if consumption.quantity_.is_non_zero() {Decimal::from(txn.amounts_.gross_) / consumption.quantity_} else {Decimal::new(0, 0)};

    //                 print!("{:>12.4}", rate); //.round_dp(2));

    //                 if charge.is_export_ {
                        
    //                 }
    //                 else {
    //                         if txn.title_.eq("Electricity") {
    //                             total_electric_charge += *&txn.amounts_.gross_;
    //                             total_electric_units += consumption.quantity_;
    //                         }
    //                     }
    //             }
    //             else {
    //                 print!("{:47}","");
    //             }
    //         }
    //         else {
    //             print!("{:>10} {:>10} {:>10} {:>10} ", 
    //                 as_decimal(-txn.amounts_.net_, 2),
    //                 as_decimal(-txn.amounts_.tax_, 2), 
    //                 as_decimal(-txn.amounts_.gross_, 2),
    //                 as_decimal(txn.balance_carried_forward_, 2)
    //             );
    //             print!("{:47}","");
    //         }
    //         if let Some(note) = &txn.note_ {
    //             let note = note.trim();
    //             print!(" {}", note);
    //         }
    //         println!();

    //     }

    //     if total_electric_units.is_positive() {
    //         println!("\nTOTALS");
    //         let rate = Decimal::from(total_electric_charge) / total_electric_units;

    //         print!("{:30} {:10} ", 
    //             "Electricity Import",
    //             ""
    //         );
    //         print!("{:>10} {:>10} {:>10} {:>10} ", 
    //             "",
    //             "", 
    //             as_decimal(total_electric_charge, 2),
    //             ""
    //         );
    //         print!("{:10} {:10} {:>12.4} ", 
    //             "",
    //             "",
    //             total_electric_units
    //         );
    //         print!("{:>12.4}", rate);
    //         println!();
    //     }
    // }

    // pub async fn fetch_all(&mut self, request_manager: &RequestManager)  -> Result<(), Error> {
    //     let mut has_previous_page = self.has_previous_page;

    //     println!("fetch_all statement transactions {} in buffer", self.agreements.len());

        

    //     while has_previous_page {
    //         let mut builder = super::graphql::bill::get_statement_transactions::Variables::builder()
    //             .with_account_number(self.account_number.clone())
    //             .with_statement_id(self.statement_id.clone())
    //             .with_transactions_first(100);

    //         if let Some(end_cursor) = &self.start_cursor {
    //             builder = builder.with_transactions_before(end_cursor.clone());
    //         }
    //         let query = super::graphql::bill::get_statement_transactions::Query::from(
    //             builder.build()?
    //         );
    //         let response = request_manager.call(&query).await?;

            
    //         if let super::graphql::bill::get_statement_transactions::BillInterface::StatementType(statement) = response.account_.bill_ {
    //             println!("request for {} statement transactions after {:?} returned {} statement transactions", 100, self.start_cursor, statement.transactions_.len());

    //             self.start_cursor = statement.transactions_.page_info.start_cursor.clone();
    //             has_previous_page = statement.transactions_.page_info.has_previous_page.clone();

    //             for edge in statement.transactions_.edges.into_iter().rev() {
    //                 let sort_key = edge.cursor;
    //                 self.agreements.push((sort_key, edge.node));
    //             }
                
    //             println!("has_previous_page = {:?}", has_previous_page);
    //         }
    //     }
    //     self.has_previous_page = has_previous_page;
    //     Ok(())
    // }
}

impl meter::electricity_agreement_line_items::AgreementInterface {
    pub fn get_line_items(self) -> Vec<EdgeOf<meter::electricity_agreement_line_items::LineItemType>> {
        match self {
            meter::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
                electricity_agreement_type.line_items_.edges
                //.into_iter().collect()
            },
            meter::electricity_agreement_line_items::AgreementInterface::GasAgreementType(abstract_agreement_interface) => unreachable!(),
        }
    }


    pub fn get_page_info(&self) -> &PageInfo {
        match self {
            meter::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
                &electricity_agreement_type.line_items_.page_info
                //.into_iter().collect()
            },
            meter::electricity_agreement_line_items::AgreementInterface::GasAgreementType(abstract_agreement_interface) => unreachable!(),
        }
    }
}

pub struct AgreementLineItems {
    pub account_number: String,
    pub agreement_id: String,
    pub end_cursor: Option<String>,
    pub has_next_page: bool,
    pub line_items: Vec<(String, meter::electricity_agreement_line_items::LineItemType)>,
    hash_key: String,
    start_date: Date,
    end_date: Date,
    start_date_time: DateTime,
    end_date_time: DateTime,
}

impl AgreementLineItems {
    async fn new(cache_manager: &CacheManager, request_manager: &AuthenticatedRequestManager<OctopusTokenManager>, account_number: String, meter_type: &MeterType, agreement_id: String, date: &Date) -> Result<Self, Error> {
        let hash_key = format!("{}#{}#{}AgreementTransactions", account_number, agreement_id, meter_type);
        let mut has_next_page = true;
        let mut end_cursor: Option<String> = None;
        let mut transactions = Vec::new();

        let (start_date, end_date) = cache_manager.read_for_date(date, &hash_key, &mut transactions)?;
        let start_date_time = start_date.at_midnight();
        let end_date_time = end_date.at_midnight();

        let cached_cnt = transactions.len();

        // println!("Loaded {} rows for AgreementLineItems[{}..{}]", cached_cnt, start_date, end_date);

        if transactions.is_empty() {
            match meter_type {
                MeterType::Gas => todo!(),
                MeterType::Electricity =>{
                    let query = meter::electricity_agreement_line_items::Query::builder()
                            .with_agreement_id(agreement_id.clone())
                            .with_start_at(start_date_time.clone())
                            .with_timezone(String::from("UTC"))
                            .with_item_type(super::graphql::LineItemTypeOptions::ConsumptionCharge)
                            .with_line_item_grouping(super::graphql::LineItemGroupingOptions::None)
                            .with_first(5)
                            .build()?;
                    let response = request_manager.call(&query).await?;
                    let response_has_next_page = *&response.electricity_agreement_.get_page_info().has_next_page;

                    for edge in response.electricity_agreement_.get_line_items() {
                        if edge.node.end_at_ >= end_date_time { // have to test here before we move edge.node and break later
                            // this bucket is full
                            has_next_page = false;
                        }
                        transactions.push((edge.cursor.clone(), edge.node));
                        end_cursor = Some(edge.cursor);


                        if !has_next_page {
                            // this bucket is full
                            break; // TODO: save this in the next bucket
                        }
                    }

                    // perhaps there were no additional rows for the next bucket but no more rows for this one either
                    if has_next_page {
                        has_next_page = response_has_next_page;
                    }
                },
            }
        }
        else {
            let (cursor, final_txn) = transactions.get(transactions.len()-1).unwrap();
            if final_txn.end_at_ >= end_date_time {
                // this bucket is full
                has_next_page = false;
            }
            end_cursor = Some(cursor.clone());
        }

        let mut result = AgreementLineItems {
            account_number,
            agreement_id,
            end_cursor,
            has_next_page,
            line_items: transactions,
            hash_key,
            start_date,
            end_date,
            start_date_time: start_date_time.clone(),
            end_date_time,
        };

        if has_next_page {
            // bucket is not yet full
            result.fetch_all(request_manager, &start_date_time).await?;
        }
            
        //     let bill = response.account_.bill_;

        //     if let bill::get_statement_transactions::BillInterface::StatementType(statement) = bill {

        //         for edge in statement.transactions_.edges {
        //             let sort_key = edge.cursor; //format!("{}#{}", &edge.node.as_bill_interface().issued_date_, &edge.cursor);
        //             transactions.push((sort_key, edge.node));
        //         }
    
        //         AgreementTransactionList {
        //             account_number,
        //             statement_id: agreement_id,
        //             end_cursor: statement.transactions_.page_info.end_cursor,
        //             has_next_page: statement.transactions_.page_info.has_next_page,
        //             transactions,
        //             hash_key,
        //         }
        //     }
        //     else {
        //         return Err(Error::StringError(format!("Bill {} is not a statement", agreement_id)))
        //     }
        // }
        // else {
        //     let (end_cursor, _) = transactions.get(transactions.len() - 1).unwrap();
        //     AgreementTransactionList {
        //         account_number,
        //         statement_id: agreement_id,
        //         end_cursor: Some(end_cursor.clone()),
        //         has_next_page: true,
        //         transactions,
        //         hash_key,
        //     }
        // };

        

        if result.line_items.len() > cached_cnt {
            cache_manager.write_for_date(&result.start_date, &result.hash_key, &result.line_items, cached_cnt)?;
        }
        
        Ok(result)
    }

    pub fn print(&self) {
        let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

        println!();
        println!("{:-^20} {:-^20} {:-^10} ", "From", "To", "Charge Amount");


        for (_cursor, item) in &self.line_items {
            println!("{:20} {:20} {:10.2} {:10.3} {:10.3}", item.start_at_.format(&format).unwrap(), item.end_at_.format(&format).unwrap(), item.net_amount_, item.number_of_units_, 
                        
            if item.number_of_units_.is_non_zero() {item.net_amount_ / item.number_of_units_} else { item.number_of_units_ }  );
            // println!("Start {} End {}", item.start_at_, item.end_at_, item.number_of_units_)

            // match agreement {
            //     graphql::meters::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
            //         for edge in &electricity_agreement_type.line_items_.edges {
            //             let item = &edge.node;

            //             println!("{:20} {:20} {:10.2} {:10.3} {:10.3}", item.start_at_.format(format).unwrap(), item.end_at_.format(format).unwrap(), item.net_amount_, item.number_of_units_, 
                        
            //             if item.number_of_units_.is_non_zero() {item.net_amount_ / item.number_of_units_} else { item.number_of_units_ }  );
            //             // println!("Start {} End {}", item.start_at_, item.end_at_, item.number_of_units_)
            //         }


            //         if electricity_agreement_type.line_items_.page_info.has_next_page {
            //             if let Some(end_cursor) = &electricity_agreement_type.line_items_.page_info.end_cursor {
            //                 Some(end_cursor.clone())
            //             }
            //             else {
            //                 None
            //             }
            //         }
            //         else {
            //             None
            //         }

            //     },
            //     graphql::meters::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_) => unreachable!(),
            // }
        }
        
    //     print!("{:30} {:10} ", 
    //         "Description",
    //         "Posted"
    //     );
    //     print!("{:>10} {:>10} {:>10} {:>10} ", 
    //         "Net",
    //         "Tax", 
    //         "Total",
    //         "Balance"
    //     );
    //     print!("{:10} {:10} {:>12} ", 
    //         "From",
    //         "To",
    //         "Units"
    //     );
    //     print!("{:>12}", "p / unit");
    //     println!();

    //     let mut total_electric_charge = 0;
    //     let mut total_electric_units = Decimal::new(0, 0);

    //     for (_key, transaction) in &self.transactions {
    //     // for edge in (&statement.transactions_.edges).into_iter().rev() {
    //         let txn = transaction.as_transaction_type();

    //         if let TransactionType::Charge(charge) = &transaction {
    //             if charge.is_export_ {
    //                 print!("{} {:width$} ", txn.title_, "Export", width = 30 - txn.title_.len() - 1);
    //             }
    //             else {
    //                     print!("{:30} ", txn.title_);
    //             }
    //         }
    //         else {
    //             print!("{:30} ", txn.title_);
    //         }
    //         print!("{:10} ", 
    //                     txn.posted_date_
    //                 );

    //         if let TransactionType::Charge(charge) = &transaction {
    //             print!("{:>10} {:>10} {:>10} {:>10} ", 
    //                 as_decimal(txn.amounts_.net_, 2),
    //                 as_decimal(txn.amounts_.tax_, 2), 
    //                 as_decimal(txn.amounts_.gross_, 2),
    //                 as_decimal(txn.balance_carried_forward_, 2)
    //             );
    //             if let Some(consumption) = &charge.consumption_ {
    //                 print!("{:10} {:10} {:>12.4} ", 
    //                     consumption.start_date_,
    //                     consumption.end_date_,
    //                     consumption.quantity_
    //                 );

    //                 let rate = if consumption.quantity_.is_non_zero() {Decimal::from(txn.amounts_.gross_) / consumption.quantity_} else {Decimal::new(0, 0)};

    //                 print!("{:>12.4}", rate); //.round_dp(2));

    //                 if charge.is_export_ {
                        
    //                 }
    //                 else {
    //                         if txn.title_.eq("Electricity") {
    //                             total_electric_charge += *&txn.amounts_.gross_;
    //                             total_electric_units += consumption.quantity_;
    //                         }
    //                     }
    //             }
    //             else {
    //                 print!("{:47}","");
    //             }
    //         }
    //         else {
    //             print!("{:>10} {:>10} {:>10} {:>10} ", 
    //                 as_decimal(-txn.amounts_.net_, 2),
    //                 as_decimal(-txn.amounts_.tax_, 2), 
    //                 as_decimal(-txn.amounts_.gross_, 2),
    //                 as_decimal(txn.balance_carried_forward_, 2)
    //             );
    //             print!("{:47}","");
    //         }
    //         if let Some(note) = &txn.note_ {
    //             let note = note.trim();
    //             print!(" {}", note);
    //         }
    //         println!();

    //     }

    //     if total_electric_units.is_positive() {
    //         println!("\nTOTALS");
    //         let rate = Decimal::from(total_electric_charge) / total_electric_units;

    //         print!("{:30} {:10} ", 
    //             "Electricity Import",
    //             ""
    //         );
    //         print!("{:>10} {:>10} {:>10} {:>10} ", 
    //             "",
    //             "", 
    //             as_decimal(total_electric_charge, 2),
    //             ""
    //         );
    //         print!("{:10} {:10} {:>12.4} ", 
    //             "",
    //             "",
    //             total_electric_units
    //         );
    //         print!("{:>12.4}", rate);
    //         println!();
    //     }
    }

    pub async fn fetch_all(&mut self, request_manager: &RequestManager, start_date_time: &DateTime)  -> Result<(), Error> {
        let mut has_next_page = self.has_next_page;

        println!("fetch_all statement transactions {} in buffer", self.line_items.len());

        

        while has_next_page {
            let mut builder = meter::electricity_agreement_line_items::Query::builder()
            .with_agreement_id(self.agreement_id.clone())
                .with_start_at(start_date_time.clone())
                .with_first(100)
                .with_timezone(String::from("UTC"))
                .with_item_type(super::graphql::LineItemTypeOptions::ConsumptionCharge)
                .with_line_item_grouping(super::graphql::LineItemGroupingOptions::None)
                ;

            if let Some(end_cursor) = &self.end_cursor {
                builder = builder.with_after(end_cursor.clone());
            }
            let query = builder.build()?;
            let response = request_manager.call(&query).await?;
            let response_has_next_page = *&response.electricity_agreement_.get_page_info().has_next_page;

            for edge in response.electricity_agreement_.get_line_items() {
                if edge.node.end_at_ >= self.end_date_time { // have to test here before we move edge.node and break later
                    // this bucket is full
                    has_next_page = false;
                }
                self.line_items.push((edge.cursor.clone(), edge.node));
                self.end_cursor = Some(edge.cursor);


                if !has_next_page {
                    // this bucket is full
                    break; // TODO: save this in the next bucket
                }
            }

            // perhaps there were no additional rows for the next bucket but no more rows for this one either
            if has_next_page {
                has_next_page = response_has_next_page;
            }
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
//         let response: super::super::graphql::bill::get_bills_and_transactions::Response = serde_json::from_str(all_json).unwrap();

//         serde_json::to_string_pretty(&response);
//     }
// }