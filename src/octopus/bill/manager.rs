use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::octopus::bill::{BillDataSet, BillTransactionBreakDown, BillTransactionList};
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
    pub bills: Mutex<HashMap<String, Arc<BillDataSet>>>,
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

    pub async fn fetch_bills(&self, account_number: String) -> anyhow::Result<Arc<BillDataSet>> {
        let mut locked_map = self.bills.lock().await;
        // let mut map = &*locked_map;
        Ok((&*locked_map
                .entry(account_number.clone())
                .or_insert(
                    Arc::new(
                        // BillList::fetch(&self.cache_manager, &self.request_manager, &account_number, crate::CHECK_FOR_UPDATES).await?
                        BillDataSet::new(&account_number, crate::CHECK_FOR_UPDATES, &self.cache_manager, &self.request_manager).await?
                    )
                )
            ).clone())
    }

    pub async fn fetch_bill_transaction_breakdown(&self, account_number: String, statement_id: String, billing_timezone: &time_tz::Tz)  -> anyhow::Result<Vec<BillTransactionBreakDown>> {


        let mut result = Vec::new();
        let transactions = BillTransactionList::new(&self.cache_manager, &self.request_manager, account_number.clone(), statement_id).await?;


        for (_key, (_cursor, transaction)) in transactions.transactions {
            if let TransactionType::Charge(charge_ref) = &transaction && charge_ref.consumption_.is_some() {
                        // print the line items making up this charge
                        //println!("Get line items {:?} - {:?}",  &consumption.start_date_, &consumption.end_date_);

                if let TransactionType::Charge(charge) = transaction {
                    if let Some(consumption) = &charge.consumption_ {
                        let meter_type = match charge.as_transaction_type().title_.as_str() {
                            "Gas" => MeterType::Gas,
                            "Electricity" => MeterType::Electricity,
                            _ => panic!("Unknown consumption type")
                        };

                        let line_items = self.meter_manager.get_line_items(&account_number, &meter_type, charge.is_export_, &consumption.start_date_, &consumption.end_date_, billing_timezone).await?;

                        result.push(BillTransactionBreakDown::from_charge(charge, line_items));
                    }
                }
            }
            else  {
                result.push(BillTransactionBreakDown::from_abstract(transaction));
            }
        }

        Ok(result)
    }
}