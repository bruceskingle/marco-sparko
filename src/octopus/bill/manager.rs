use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::octopus::bill::{BillList, BillTransactionBreakDown, BillTransactionList};
// use anyhow::anyhow;
use crate::octopus::meter::MeterType;
use crate::CacheManager;

use super::super::graphql::bill;
use super::super::meter::MeterManager;
use bill::get_statement_transactions::TransactionType;
use super::super::RequestManager;

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


    pub async fn bill_handler(&self, mut args: std::str::SplitWhitespace<'_>, account_number: String, billing_timezone: &time_tz::Tz) ->  anyhow::Result<()> {
        // let one_hundred = Decimal::new(100, 0);
        // let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
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