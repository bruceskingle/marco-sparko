pub mod error;
pub mod token;
pub mod decimal;
mod account;
mod bill;
mod meter;

use std::sync::Arc;
use account::AccountManager;
use async_trait::async_trait;

use bill::BillManager;
use meter::MeterManager;
use serde::{Deserialize, Serialize};


pub use error::Error;
use sparko_graphql::types::{Date, DateTime};
use token::{OctopusTokenManager, TokenManagerBuilder};
use clap::Parser;

use crate::{CacheManager, CommandProvider, MarcoSparkoContext, Module, ModuleBuilder, ModuleConstructor, ReplCommand};

include!(concat!(env!("OUT_DIR"), "/graphql.rs"));

pub type RequestManager = sparko_graphql::AuthenticatedRequestManager<OctopusTokenManager>;

#[derive(Parser, Debug)]
pub struct OctopusArgs {
    /// The Octopus API_KEY to use
    #[arg(short, long, env)]
    octopus_api_key: Option<String>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub api_key:  Option<String>,
    #[serde(skip)]
    // #[serde(default = false)]
    pub init: bool,
}

impl Profile {
    pub fn new() -> Profile {
        Profile {
            api_key: None,
            init: false,
        }
    }
}

pub struct Client{
    context: Arc<MarcoSparkoContext>, 
    profile: Option<Profile>,
    request_manager: Arc<RequestManager>,
    // default_account: Option<Arc<graphql::summary::get_viewer_accounts::AccountInterface>>,
    account_id: String,
    cache_manager: Arc<CacheManager>,
    bill_manager: BillManager,
    meter_manager: MeterManager,
    account_manager: AccountManager,
}

const MODULE_ID: &str = "octopus";

#[async_trait(?Send)]
impl CommandProvider for Client {

    async fn exec_repl_command(&mut self, command: &str, args: std::str::SplitWhitespace<'_>) ->  Result<(), super::Error> {
        let account_id = self.account_id.clone();
        match command {
            "bills" => {
                Ok(self.bill_manager
                .bills_handler(args, &account_id)
                .await?)
            },
            "bill" => {
                Ok(self.bill_manager.bill_handler(args, &account_id, &mut self.meter_manager).await?)
            },
            _ => Err(super::Error::UserError(format!("Invalid command '{}'", command)))
        }
    }

    fn get_repl_commands(&self) -> Vec<ReplCommand> {
        vec!(
            ReplCommand {
                command:"bills",
                description: "Print a summary of all bills",
                help:
r#"
usage: bills

Print a one line summary of all bills in the account.
"#,
            },

            ReplCommand {
                command:"bill",
                description: "Print details of a bill",
                help:
r#"
usage: bill [bill_id]

Print the contents of the bill whose id is given, or the most recent bill, if none.
"#,
            }
        )
    }
}

impl Client {
    async fn new(context: Arc<MarcoSparkoContext>, profile: Option<Profile>, 
        request_manager: Arc<RequestManager>) -> Result<Client, Error> {   

        let cache_manager = context.create_cache_manager(crate::octopus::MODULE_ID)?;
        let account_manager = AccountManager::new(&cache_manager, &request_manager).await?;
        let bill_manager = BillManager::new(&cache_manager, &request_manager);
        let meter_manager = MeterManager::new(&cache_manager, &request_manager);

        Ok(Client {
            context,
            profile,
            request_manager,
            account_id: account_manager.get_default_account_id().to_string(),
            cache_manager,
            account_manager,
            bill_manager,
            meter_manager,
        })
    }

    fn get_api_key(&self) -> &Option<String> {
        &self.account_manager.viewer.viewer.viewer_.live_secret_key_
    }

    pub fn registration() -> (String, Box<ModuleConstructor>) {

        // Client::foo(Client::constructor);

        (MODULE_ID.to_string(), Box::new(Client::constructor))
    }
    
    pub fn constructor(context: Arc<MarcoSparkoContext>, 
        json_profile: Option<serde_json::Value>) -> Result<Box<dyn ModuleBuilder>, crate::Error> {
            Ok(Client::builder(context, json_profile)?)
    }

    pub fn builder(context: Arc<MarcoSparkoContext>, 
        json_profile: Option<serde_json::Value>
    ) -> Result<Box<dyn ModuleBuilder>, Error> {

        ClientBuilder::new(context, json_profile)
    }

    // pub async fn get_bill_manager(&mut self)  -> Result<&mut BillManager, Error> {

    //     if self.bill_manager.is_none() {
    //         let account = self.get_default_account().await?;
    //         let bill_manager = BillManager::new(&self.cache_manager, &self.request_manager, account.number_.clone()).await?;
    //         self.bill_manager = Some(bill_manager);
    //     }
    //     Ok(self.bill_manager.as_mut().unwrap())
    // }

    // pub async fn get_meter_manager(&mut self)  -> Result<&mut MeterManager, Error> {

    //     if self.meter_manager.is_none() {
    //         let account = self.get_default_account().await?;
    //         let meter_manager = MeterManager::new(&self.cache_manager, &self.request_manager, account.number_.clone()).await?;
    //         self.meter_manager = Some(meter_manager);
    //     }
    //     Ok(self.meter_manager.as_mut().unwrap())
    // }

    // pub async fn get_default_account(&mut self)  -> Result<Arc<graphql::summary::get_viewer_accounts::AccountInterface>, Error> {
    //     if let Some(default_account) = &self.default_account {
    //         Ok(default_account.clone())
    //     }
    //     else {
    //         let query = graphql::summary::get_viewer_accounts::Query::new();
    //         let mut response = self.request_manager.call(&query).await?;


    //         let default_account = Arc::new(response.viewer_.accounts_.remove(0));
    //         let return_value: Arc<graphql::summary::get_viewer_accounts::AccountInterface> = default_account.clone();
    //         self.default_account = Some(default_account);
    //         Ok(return_value)
    //     }
    // }

    // pub async fn get_account_user(&mut self)  -> Result<AccountUserType, Error> {
    //     let query = graphql::summary::get_account_summary::Query::new();
    //     let response = self.request_manager.call(&query).await?;

    //     Ok(response.viewer_)
    // }

    

    // // Rename this to say what it actually does, after we delete the function of the same name in account.rs
    // pub async fn get_account_properties_meters(&mut self,
    //     from: &Date,
    //     to: &Option<Date>)  -> Result<(), Error> {
    //     // let account_number = self.get_default_account().await?.number_;
    //     let query = graphql::meters::get_account_properties_meters::Query::from(graphql::meters::get_account_properties_meters::Variables::builder()
    //         .with_account_number(self.get_default_account().await?.number_.clone())
    //         .build()?
    //     );
    //     let response = self.request_manager.call(&query).await?;

    //     for property in response.account_.properties_ {
    //         for electricity_meter_point in property.electricity_meter_points_ {
    //             for electricity_meter in &electricity_meter_point.meters_ {
    //                 if let Some(_import_meter) = &electricity_meter.import_meter_ {
    //                     // println!("Export electricity meter {}", &electricity_meter.node_id_);
    //                     // export_meters.push(electricity_meter.node_id);

    //                     self.get_meter_agreements(
    //                         &electricity_meter.node_id_,
    //                         from,
    //                         to).await?;
    //                 }
    //                 else {
    //                     // println!("Import electricity meter {}", &electricity_meter.node_id_);
    //                     // import_meters.push(electricity_meter.node_id);
    //                     self.get_meter_agreements(
    //                         &electricity_meter.node_id_,
    //                         from,
    //                         to).await?;
    //                 }
    //             }
    //         }

    //         for gas_meter_point in property.gas_meter_points_ {
    //             for gas_meter in &gas_meter_point.meters_ {
    //                 // println!("Gas meter {}", &gas_meter.node_id_);
    //                 // gas_meters.push(gas_meter.node_id);
    //                 self.get_meter_agreements(
    //                     &gas_meter.node_id_,
    //                     from,
    //                     to).await?;
    //             }
    //         }
    //     }
    //     Ok(())
    // }



    // fn in_scope(from: &Date, to: &Option<Date>, valid_from: &DateTime, valid_to: &Option<DateTime>) -> bool {
    //     let end_in_scope = if let Some(to) = to {
    //         // if let Some(valid_from) = valid_from {
    //             valid_from.to_date() <= *to
    //         // }
    //         // else {
    //         //     false
    //         // }
    //     }
    //     else {
    //         true
    //     };

    //     let start_in_scope = if let Some(valid_to) = valid_to {
    //             valid_to.to_date() >= *from

    //     }
    //     else {
    //         true
    //     };
        
    //     // println!("in_scope({:?},{:?},{:?},{:?}) = {}",
    //     //     from,
    //     //     to,
    //     //     valid_from,
    //     //     valid_to,
    //     //     end_in_scope && start_in_scope
    //     // );
    //     end_in_scope && start_in_scope
    // }
 
    // pub async fn get_meter_agreements (
    //     &mut self,
    //     meter_node_id: &String,
    //     from: &Date,
    //     to: &Option<Date>,
    // ) -> Result<(), Box<dyn std::error::Error>> {

    //     let query = graphql::meters::meter_agreements::Query::from(graphql::meters::meter_agreements::Variables::builder()
    //         .with_meter_node_id(meter_node_id.clone())
    //         .with_valid_after(from.at_midnight())
    //         .build()?
    //     );
    //     let response = self.request_manager.call(&query).await?;


    //         match response.node_ {

    //             graphql::meters::meter_agreements::Node::ElectricityMeterType(electricity_meter_type) => {
    //                 for agreement in electricity_meter_type.meter_point_.agreements_
    //                 {
    //                     // println!("Agreement {:?}", &agreement);
    //                     if Self::in_scope(&from, &to, &agreement.valid_from_, &agreement.valid_to_) {
    //                         // let agreement_id = Self::unexpected_none()?.to_string();
    //                         // println!("Electricity agreement {}", &agreement.id_);

    //                         self.get_electric_line_items(format!("{}", &agreement.id_), from, to).await?;

    //                     }
    //                 } 
    //             },
    //             graphql::meters::meter_agreements::Node::GasMeterType(gas_meter_type) => {
    //                 for agreement in gas_meter_type.meter_point_.agreements_
    //                 {
    //                     if Self::in_scope(&from, &to, &agreement.valid_from_, &agreement.valid_to_) {
    //                         // let agreement_id = unexpected_none(agreement.id)?;
    //                         println!("Gas agreement {}", &agreement.id_);
    //                     }
    //                 } 
    //             },

                
    //             _ => return Err(Box::new(Error::InternalError("Unexpected node type found in agreements query")))
    //          };



    //     Ok(())
    // }


 
    // pub async fn get_electric_line_items (
    //     &mut self,
    //     agreement_id: String,
    //     from: &Date,
    //     to: &Option<Date>,
    // ) -> Result<(), Error> {
    //     let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

    //     fn handle(agreement: &graphql::meters::electricity_agreement_line_items::AgreementInterface, format: &[time::format_description::FormatItem<'_>]) -> Option<String> {

    //         match agreement {
    //             graphql::meters::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
    //                 for edge in &electricity_agreement_type.line_items_.edges {
    //                     let item = &edge.node;

    //                     println!("{:20} {:20} {:10.2} {:10.3} {:10.3}", item.start_at_.format(format).unwrap(), item.end_at_.format(format).unwrap(), item.net_amount_, item.number_of_units_, 
                        
    //                     if item.number_of_units_.is_non_zero() {item.net_amount_ / item.number_of_units_} else { item.number_of_units_ }  );
    //                     // println!("Start {} End {}", item.start_at_, item.end_at_, item.number_of_units_)
    //                 }


    //                 if electricity_agreement_type.line_items_.page_info.has_next_page {
    //                     if let Some(end_cursor) = &electricity_agreement_type.line_items_.page_info.end_cursor {
    //                         Some(end_cursor.clone())
    //                     }
    //                     else {
    //                         None
    //                     }
    //                 }
    //                 else {
    //                     None
    //                 }

    //             },
    //             graphql::meters::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_) => unreachable!(),
    //         }
    //     }
    //     println!();
    //     println!("{:-^20} {:-^20} {:-^10} ", "From", "To", "Charge Amount");
    //     let from = Date::from_calendar_date(2024, time::Month::January, 1)?;

    //     let query = graphql::meters::electricity_agreement_line_items::Query::from(graphql::meters::electricity_agreement_line_items::Variables::builder()
    //         .with_agreement_id(agreement_id.clone())
    //         .with_start_at(from.at_midnight())
    //         .with_first(100)
    //         .with_timezone(String::from("Europe/London"))
    //         .with_item_type(graphql::LineItemTypeOptions::ConsumptionCharge)
    //         .with_line_item_grouping(graphql::LineItemGroupingOptions::None)
    //         .build()?
    //     );
    //     let mut response = self.request_manager.call(&query).await?;

    //     loop {

    //         if let Some(cursor) = handle(&response.electricity_agreement_, &format) {
    //             let query = graphql::meters::electricity_agreement_line_items::Query::from(graphql::meters::electricity_agreement_line_items::Variables::builder()
    //             .with_agreement_id(agreement_id.clone())
    //             .with_start_at(from.at_midnight())
    //             .with_first(5)
    //             .with_last_cursor(cursor)
    //             .with_timezone(String::from("Europe/London"))
    //             .with_item_type(graphql::LineItemTypeOptions::ConsumptionCharge)
    //             .with_line_item_grouping(graphql::LineItemGroupingOptions::None)
    //             .build()?
    //             );
    //             response = self.request_manager.call(&query).await?;
    //         }
    //         else {
    //             break;
    //         }
            

    //     }
    //     // println!("{}", &response.electricity_agreement_);

    //     Ok(())
    // }

    async fn update_profile(&mut self)  -> Result<(), Error> {

        let api_key = if let Some(profile) = &self.profile {
            profile.api_key.clone()
        }
        else {
            None
        };

        if let Some(new_api_key) = self.get_api_key() {
            if let Some(old_profile) = &self.profile {
            
                if 
                    if let Some(old_api_key) = api_key {
                        old_api_key.ne(new_api_key)
                    }
                    else {
                        true
                    }
                {
                    // let old_octopus_config = new_profile.octopus_config;
                    let new_profile = Profile {
                        api_key: Some(new_api_key.clone()),
                        ..old_profile.clone()
                    };

                    println!("UPDATE profile <{:?}>", &new_profile);

                    self.context.update_profile(MODULE_ID, new_profile)?;
                }
            }
            else {
                let mut new_profile  = Profile::new();
                new_profile.api_key = Some(new_api_key.clone());

                println!("CREATE profile <{:?}>", &new_profile);
                self.context.update_profile(MODULE_ID, new_profile)?;
            }
        }
        Ok(())
    }

    // pub async fn handle_bill(&mut self, bill: &BillInterface) -> Result<(), crate::Error> {
    //     //println!("\n===========================\n{}\n===========================\n", result);
    //     Self::print_statement(bill);

    //     panic!("STOP");
        
    //     let abstract_bill = bill.as_bill_interface();
    //             // statement.print();

    //     println!("Energy Account Statement");
    //     println!("========================");
    //     println!("Date                 {}", abstract_bill.issued_date_);
    //     println!("Ref                  {}", abstract_bill.id_);
    //     println!("From                 {}", abstract_bill.from_date_);
    //     println!("To                   {}", abstract_bill.to_date_);

    //     if let BillInterface::StatementType(statement) = bill {
    //         let mut map = BTreeMap::new();
    //         for edge in &statement.transactions_.edges_ {
    //             map.insert(&edge.node_.as_transaction_type().posted_date_, &edge.node_);
    //         }


    //         println!();
    //         print!("{:-^20} {:-^10} ", 
    //             "Title",
    //             "Date"
    //         );
    //         print!("{:-^10} {:-^10} {:-^10} {:-^10}", 
    //             "Net", "Tax", "Gross", "c/f"
    //         );
    //         println!("{:-^10} {:-^10} {:-^10} ", "From", "To", "Charge Amount");

    //         for txn in &mut map.values() {
    //             // let txn = transaction.as_transaction_type();

    //             let title = if let TransactionType::Charge(charge) = txn {
    //                 if charge.is_export_ {
    //                     format!("{} Export", txn.as_transaction_type().title_)
    //                 }
    //                 else {
    //                     txn.as_transaction_type().title_.clone()
    //                 }
    //             }
    //             else {
    //                 txn.as_transaction_type().title_.clone()
    //             };

    //             print!("{:20} {:10} ", 
    //                 title,
    //                 txn.as_transaction_type().posted_date_
    //             );
    //             print!("{:10.2} {:10.2} {:10.2} {:10.2}", 
    //                 txn.as_transaction_type().amounts_.net_,
    //                 txn.as_transaction_type().amounts_.tax_, 
    //                 txn.as_transaction_type().amounts_.gross_,
    //                 txn.as_transaction_type().balance_carried_forward_
    //                 );
                

    //                 if let TransactionType::Charge(charge) = txn {
    //                     if let Some(consumption) = &charge.consumption_ {
    //                         print!(" {:10} {:10} {:10.2} ", 
    //                             consumption.start_date_,
    //                             consumption.end_date_,
    //                             consumption.quantity_
    //                         );

    //                         println!();
    //                         self.get_account_properties_meters(&consumption.start_date_, &Some(consumption.end_date_.clone())).await?;
    //                     }

    //                 }
    //             println!();
    //         } 
    //     }

    //     Ok(())
    // }

    // pub fn print_statement( bill: &BillInterface) {
    //     let abstract_bill = bill.as_bill_interface();

    //     println!("Energy Account Statement");
    //     println!("========================");
    //     println!("Date                 {}", abstract_bill.issued_date_);
    //     println!("Ref                  {}", abstract_bill.id_);
    //     println!("From                 {}", abstract_bill.from_date_);
    //     println!("To                   {}", abstract_bill.to_date_);
    //     println!();

    //     if let BillInterface::StatementType(statement) = bill {
    //         print!("{:20} {:10} ", 
    //             "Description",
    //             "Posted"
    //         );
    //         print!("{:>10} {:>10} {:>10} {:>10} ", 
    //             "Net",
    //             "Tax", 
    //             "Total",
    //             "Balance"
    //         );
    //         print!("{:10} {:10} {:>12} ", 
    //             "From",
    //             "To",
    //             "Units"
    //         );
    //         print!("{:>12}", "p / unit");
    //         println!();

    //         let mut total_electric_charge = 0;
    //         let mut total_electric_units = Decimal::new(0, 0);

    //         // for transaction in &mut map.values() {
    //         for edge in (&statement.transactions_.edges_).into_iter().rev() {
    //             let txn = edge.node_.as_transaction_type();

    //             if let TransactionType::Charge(charge) = &edge.node_ {
    //                 if charge.is_export_ {
    //                     print!("{} {:width$} ", txn.title_, "Export", width = 20 - txn.title_.len() - 1);
    //                 }
    //                 else {
    //                         print!("{:20} ", txn.title_);
    //                 }
    //             }
    //             else {
    //                 print!("{:20} ", txn.title_);
    //             }
    //             print!("{:10} ", 
    //                         txn.posted_date_
    //                     );
    //             print!("{:>10} {:>10} {:>10} {:>10} ", 
    //                 as_decimal(txn.amounts_.net_, 2),
    //                 as_decimal(txn.amounts_.tax_, 2), 
    //                 as_decimal(txn.amounts_.gross_, 2),
    //                 as_decimal(txn.balance_carried_forward_, 2)
    //             );
    //             if let TransactionType::Charge(charge) = &edge.node_ {
    //                 if let Some(consumption) = &charge.consumption_ {
    //                     print!("{:10} {:10} {:>12.4} ", 
    //                         consumption.start_date_,
    //                         consumption.end_date_,
    //                         consumption.quantity_
    //                     );

    //                     let rate = Decimal::from(txn.amounts_.gross_) / consumption.quantity_;

    //                     print!("{:>12.4}", rate); //.round_dp(2));

    //                     if charge.is_export_ {
                            
    //                     }
    //                     else {
    //                             if txn.title_.eq("Electricity") {
    //                                 total_electric_charge += *&txn.amounts_.gross_;
    //                                 total_electric_units += consumption.quantity_;
    //                             }
    //                         }
    //                 }
    //             }
    //             print!(" {}", txn.note_);
    //             println!();
    //         }

    //         println!("\nTOTALS");

    //         if total_electric_units.is_positive() {
    //             let rate = Decimal::from(total_electric_charge) / total_electric_units;

    //             print!("{:20} {:10} ", 
    //                 "Electricity Import",
    //                 ""
    //             );
    //             print!("{:>10} {:>10} {:>10} {:>10} ", 
    //                 "",
    //                 "", 
    //                 as_decimal(total_electric_charge, 2),
    //                 ""
    //             );
    //             print!("{:10} {:10} {:>12.4} ", 
    //                 "",
    //                 "",
    //                 total_electric_units
    //             );
    //             print!("{:>12.4}", rate);
    //             println!();
    //         }
    //     }
    // }
    // async fn do_repl(&self) -> easy_repl::anyhow::Result<()> {

    // use easy_repl::command;
    // use easy_repl::{anyhow::Context, CommandStatus};

    //     let mut repl = easy_repl::Repl::builder()
    //         .add("hello", command! {
    //             "Say hello",
    //             (name: String) => |name| {
    //                 println!("Hello {}!", name);
    //                 Ok(CommandStatus::Done)
    //             }
    //         })
    //         .add("add", command! {
    //             "Add X to Y",
    //             (X:i32, Y:i32) => |x, y| {
    //                 println!("{} + {} = {}", x, y, x + y);
    //                 Ok(CommandStatus::Done)
    //             }
    //         })
    //         .build().context("Failed to create repl")?;
    
    //     repl.run().context("Critical REPL error")?;
    
    //     Ok(())
    // }

    // async fn hello<T>(args: ArgMatches, _context: &mut T) -> reedline_repl_rs::Result<Option<String>> {
    //     Ok(Some(format!(
    //         "Hello, {}",
    //         args.get_one::<String>("who").unwrap()
    //     )))
    // }
    
    // /// Called after successful command execution, updates prompt with returned Option
    // async fn update_prompt<T>(_context: &mut T) -> reedline_repl_rs::Result<Option<String>> {
    //     Ok(Some("updated".to_string()))
    // }

    // async fn do_repl(&mut self) { //-> impl Future<Output =  reedline_repl_rs::Result<()>> + Send { //-> reedline_repl_rs::Result<()> {

    //     use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
    //     use reedline_repl_rs::{Repl, Result};

    //     let mut repl: Repl<(), reedline_repl_rs::Error> = Repl::new(())
    //         .with_name("MyApp")
    //         .with_version("v0.1.0")
    //         .with_command_async(
    //             Command::new("hello")
    //                 .arg(Arg::new("who").required(true))
    //                 .about("Greetings!"),
    //             |args, context| Box::pin(Self::hello(args, context)),
    //         )
    //         .with_on_after_command_async(|context| Box::pin(Self::update_prompt(context)));

    //     //    repl.run_async();
    //     if let Err(error) = repl.run_async().await {
    //         println!("ERROR: {}", error);
    //     }
    // }
}

// unsafe impl Send for Client {

// }


    // async fn repl(&mut self) -> Result<(), crate::Error> {
    //     self.do_repl().await;
    //     Ok(())
    // }}

#[async_trait]
impl Module for Client {

    // async fn test(&mut self) -> Result<(), crate::Error>{
    //     let user = self.get_account_user().await?;
    //     println!("get_account_user {} {} {}", user.given_name_, user.family_name_, user.email_);
    //     let account = self.get_default_account().await?;
    //     println!("get_default_account {}", account.number_);
    //     Ok(())
    // }

    // async fn summary(&mut self) -> Result<(), crate::Error>{
    //     let user = self.get_account_user().await?;
    //     println!("{}", user);
    //     Ok(())
    // }

    // async fn bill(&mut self) -> Result<(), crate::Error>{
    //     println!("DEPRECATED");
    //     // let account = self.get_default_account().await?;
    //     // // let account_number =  &account.number_;

    //     // let mut bills = bill::get_bills(&self.cache_manager, &self.request_manager, account.number_.clone()).await?;

    //     // // bills.fetch_all(&self.request_manager).await?;

    //     // bills.print_summary_lines();

    //     Ok(())
    // }
}


pub struct ClientBuilder {
    context: Arc<MarcoSparkoContext>, 
    profile: Option<Profile>,
    token_manager_builder: TokenManagerBuilder,
    url: Option<String>,
    verbose: bool,
}

impl ClientBuilder {

    fn get_profile_api_key(option_profile: &Option<Profile>) -> Result<Option<String>, Error> {

        if let Some(profile) =  option_profile {
            if let Some(api_key) = &profile.api_key {
                return Ok(Some(api_key.to_string()))
            }
        }

        Ok(None)
    }

    fn new(
            context: Arc<MarcoSparkoContext>,
            json_profile: Option<serde_json::Value>
        ) -> Result<Box<dyn ModuleBuilder>, Error> {

        let profile: Option<Profile> = if let Some(json) = json_profile {
            serde_json::from_value(json)?
        }
        else {
            None
        };

        let option_api_key = if let Some(api_key) = &context.args.octopus.octopus_api_key {
            Some(api_key.to_string())
        }
        else {
            Self::get_profile_api_key(&profile)?
        };

        let verbose = context.args.verbose;

        let builder = ClientBuilder {
            context,
            profile,
            token_manager_builder: OctopusTokenManager::builder(),
            url: None,
            verbose,
        };

        if let Some(api_key) = option_api_key {
            Ok(Box::new(builder.with_api_key(api_key)?))
        }
        else {
            Ok(Box::new(builder))
        }
        
    }

    pub fn with_url(mut self, url: String) -> Result<ClientBuilder, Error> {
        self.url = Some(url);
        Ok(self)
    }

    pub fn with_url_if_not_set(mut self, url: String) -> Result<ClientBuilder, Error> {
        if let None = self.url {
            self.url = Some(url);
        }
        Ok(self)
    }

    pub fn with_api_key(mut self, api_key: String) -> Result<ClientBuilder, Error> {
        self.token_manager_builder = self.token_manager_builder.with_api_key(api_key);
        Ok(self)
    }

    pub fn with_password(mut self, email: String, password: String) -> Result<ClientBuilder, Error> {
        self.token_manager_builder = self.token_manager_builder.with_password(email, password);
        Ok(self)
    }

    pub async fn do_build(self, init: bool) -> Result<Client, Error> {
        let option_profile = if init {
            if let Some(mut profile) = self.profile {
                profile.init = true;
                Some(profile)
            }
            else {
                let mut profile = Profile::new();
                profile.init = true;

                Some(profile)
            }
        }
        else {
            self.profile
        };

        let url = if let Some(url) = self.url {
            url
        }
        else {
            "https://api.octopus.energy/v1/graphql/".to_string()
        };

        let request_manager = Arc::new(sparko_graphql::RequestManager::new(url.clone(), self.verbose)?);

        let token_manager = self.token_manager_builder
            .with_request_manager(request_manager.clone())
            .with_context(self.context.clone())
            .build(init)?;

        let authenticated_request_manager = Arc::new(sparko_graphql::AuthenticatedRequestManager::new(request_manager, token_manager)?);
       
        let mut client = Client::new(self.context, option_profile, 
            authenticated_request_manager
        ).await?;

        if init {
            // let account_user = client.get_account_user().await?;
            // let x = client.account_manager.viewer.viewer.viewer_.live_secret_key_
            client.update_profile().await?;
        }
        
        Ok(client)
    }
}

#[async_trait]
impl ModuleBuilder for ClientBuilder {
    async fn build(self: Box<Self>, init: bool) -> Result<Box<dyn crate::Module + Send>, crate::Error> {
        Ok(Box::new(self.do_build(init).await?))
    }
}