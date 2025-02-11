pub mod error;
pub mod token;
pub mod decimal;

use std::{collections::BTreeMap, sync::Arc};
use async_trait::async_trait;
use display_json::DisplayAsJsonPretty;
use graphql::{latest_bill::get_account_latest_bill::{BillInterface, TransactionType}, summary::get_account_summary::AccountUserType};
use serde::{Deserialize, Serialize};

pub use error::Error;
use sparko_graphql::types::{Date, DateTime};
use token::{OctopusTokenManager, TokenManagerBuilder};
use clap::Parser;

use crate::{Context, Module, ModuleBuilder, ModuleConstructor};

include!(concat!(env!("OUT_DIR"), "/graphql.rs"));


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


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct Location {
line: i32,
column: i32,
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct ValidationError {
    message: String,
    input_path: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct Extensions {
error_type: String,
error_code: String,
error_description: String,
error_class: String,
validation_errors: Vec<ValidationError>
}



//  self.config.get_active_profile()?.modules.octopus.clone()
// #[derive(Debug)]
pub struct Client{
    context: Context, 
    profile: Option<Profile>,
    gql_authenticated_request_manager: sparko_graphql::AuthenticatedRequestManager<OctopusTokenManager>,
    default_account: Option<Arc<graphql::summary::get_viewer_accounts::AccountInterface>>
}

const MODULE_ID: &str = "octopus";

impl Client {
    pub fn registration() -> (String, Box<ModuleConstructor>) {

        // Client::foo(Client::constructor);

        (MODULE_ID.to_string(), Box::new(Client::constructor))
    }
    
    pub fn constructor(context: Box<&Context>, 
        json_profile: Option<serde_json::Value>) -> Result<Box<dyn ModuleBuilder>, crate::Error> {
            Ok(Client::builder(&context, json_profile)?)
    }

    pub fn builder(context: &Context, 
        json_profile: Option<serde_json::Value>
    ) -> Result<Box<dyn ModuleBuilder>, Error> {

        ClientBuilder::new(context, json_profile)
    }

    fn new(context: Context, profile: Option<Profile>, 
        gql_authenticated_request_manager: sparko_graphql::AuthenticatedRequestManager<OctopusTokenManager>) -> Client {        

        Client {
            context,
            profile,
            gql_authenticated_request_manager,
            default_account: None
        }
    }

    pub async fn get_default_account(&mut self)  -> Result<Arc<graphql::summary::get_viewer_accounts::AccountInterface>, Error> {
        if let Some(default_account) = &self.default_account {
            Ok(default_account.clone())
        }
        else {
            let query = graphql::summary::get_viewer_accounts::Query::new();
            let mut response = self.gql_authenticated_request_manager.call(&query).await?;


            let default_account = Arc::new(response.viewer_.accounts_.remove(0));
            let return_value = default_account.clone();
            self.default_account = Some(default_account);
            Ok(return_value)
        }
    }

    pub async fn get_account_user(&mut self)  -> Result<AccountUserType, Error> {
        let query = graphql::summary::get_account_summary::Query::new();
        let response = self.gql_authenticated_request_manager.call(&query).await?;

        Ok(response.viewer_)
    }

    pub async fn get_latest_bill(&mut self)  -> Result<graphql::latest_bill::get_account_latest_bill::BillConnectionTypeConnection, Error> {
        // let account_number = self.get_default_account().await?.number_;
        let query = graphql::latest_bill::get_account_latest_bill::Query::from(graphql::latest_bill::get_account_latest_bill::Variables::builder()
            .with_account_number(self.get_default_account().await?.number_.clone())
            .with_bills_first(1)
            .with_bills_transactions_first(100)
            .build()?
        );
        let response = self.gql_authenticated_request_manager.call(&query).await?;

        Ok(response.account_.bills_)
    }

    // Rename this to say what it actually does, after we delete the function of the same name in account.rs
    pub async fn get_account_properties_meters(&mut self,
        from: &Date,
        to: &Option<Date>)  -> Result<(), Error> {
        // let account_number = self.get_default_account().await?.number_;
        let query = graphql::meters::get_account_properties_meters::Query::from(graphql::meters::get_account_properties_meters::Variables::builder()
            .with_account_number(self.get_default_account().await?.number_.clone())
            .build()?
        );
        let response = self.gql_authenticated_request_manager.call(&query).await?;

        for property in response.account_.properties_ {
            for electricity_meter_point in property.electricity_meter_points_ {
                for electricity_meter in &electricity_meter_point.meters_ {
                    if let Some(_import_meter) = &electricity_meter.import_meter_ {
                        // println!("Export electricity meter {}", &electricity_meter.node_id_);
                        // export_meters.push(electricity_meter.node_id);

                        self.get_meter_agreements(
                            &electricity_meter.node_id_,
                            from,
                            to).await?;
                    }
                    else {
                        // println!("Import electricity meter {}", &electricity_meter.node_id_);
                        // import_meters.push(electricity_meter.node_id);
                        self.get_meter_agreements(
                            &electricity_meter.node_id_,
                            from,
                            to).await?;
                    }
                }
            }

            for gas_meter_point in property.gas_meter_points_ {
                for gas_meter in &gas_meter_point.meters_ {
                    // println!("Gas meter {}", &gas_meter.node_id_);
                    // gas_meters.push(gas_meter.node_id);
                    self.get_meter_agreements(
                        &gas_meter.node_id_,
                        from,
                        to).await?;
                }
            }
        }
        Ok(())
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
 
    pub async fn get_meter_agreements (
        &mut self,
        meter_node_id: &String,
        from: &Date,
        to: &Option<Date>,
    ) -> Result<(), Box<dyn std::error::Error>> {

        let query = graphql::meters::meter_agreements::Query::from(graphql::meters::meter_agreements::Variables::builder()
            .with_meter_node_id(meter_node_id.clone())
            .with_valid_after(from.at_midnight())
            .build()?
        );
        let response = self.gql_authenticated_request_manager.call(&query).await?;


            match response.node_ {

                graphql::meters::meter_agreements::Node::ElectricityMeterType(electricity_meter_type) => {
                    for agreement in electricity_meter_type.meter_point_.agreements_
                    {
                        // println!("Agreement {:?}", &agreement);
                        if Self::in_scope(&from, &to, &agreement.valid_from_, &agreement.valid_to_) {
                            // let agreement_id = Self::unexpected_none()?.to_string();
                            // println!("Electricity agreement {}", &agreement.id_);

                            self.get_electric_line_items(format!("{}", &agreement.id_), from, to).await?;

                        }
                    } 
                },
                graphql::meters::meter_agreements::Node::GasMeterType(gas_meter_type) => {
                    for agreement in gas_meter_type.meter_point_.agreements_
                    {
                        if Self::in_scope(&from, &to, &agreement.valid_from_, &agreement.valid_to_) {
                            // let agreement_id = unexpected_none(agreement.id)?;
                            println!("Gas agreement {}", &agreement.id_);
                        }
                    } 
                },

                
                _ => return Err(Box::new(Error::InternalError("Unexpected node type found in agreements query")))
             };



        Ok(())
    }


 
    pub async fn get_electric_line_items (
        &mut self,
        agreement_id: String,
        from: &Date,
        to: &Option<Date>,
    ) -> Result<(), Error> {
        let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

        fn handle(agreement: &graphql::meters::electricity_agreement_line_items::AgreementInterface, format: &[time::format_description::FormatItem<'_>]) -> Option<String> {

            match agreement {
                graphql::meters::electricity_agreement_line_items::AgreementInterface::ElectricityAgreementType(electricity_agreement_type) => {
                    for edge in &electricity_agreement_type.line_items_.edges_ {
                        let item = &edge.node_;

                        println!("{:20} {:20} {:8.2} {:7.3} {:7.3}", item.start_at_.format(format).unwrap(), item.end_at_.format(format).unwrap(), item.net_amount_, item.number_of_units_, 
                        
                        if item.number_of_units_.is_non_zero() {item.net_amount_ / item.number_of_units_} else { item.number_of_units_ }  );
                        // println!("Start {} End {}", item.start_at_, item.end_at_, item.number_of_units_)
                    }


                    if electricity_agreement_type.line_items_.page_info_.has_next_page_ {
                        Some(electricity_agreement_type.line_items_.page_info_.end_cursor_.clone())
                    }
                    else {
                        None
                    }

                },
                graphql::meters::electricity_agreement_line_items::AgreementInterface::GasAgreementType(_) => unreachable!(),
            }
        }

        let from = Date::from_calendar_date(2024, time::Month::January, 1)?;

        let query = graphql::meters::electricity_agreement_line_items::Query::from(graphql::meters::electricity_agreement_line_items::Variables::builder()
            .with_agreement_id(agreement_id.clone())
            .with_start_at(from.at_midnight())
            .with_first(5)
            .with_timezone(String::from("Europe/London"))
            .with_item_type(graphql::LineItemTypeOptions::ConsumptionCharge)
            .with_line_item_grouping(graphql::LineItemGroupingOptions::None)
            .build()?
        );
        let mut response = self.gql_authenticated_request_manager.call(&query).await?;

        loop {

            if let Some(cursor) = handle(&response.electricity_agreement_, &format) {
                let query = graphql::meters::electricity_agreement_line_items::Query::from(graphql::meters::electricity_agreement_line_items::Variables::builder()
                .with_agreement_id(agreement_id.clone())
                .with_start_at(from.at_midnight())
                .with_first(5)
                .with_last_cursor(cursor)
                .with_timezone(String::from("Europe/London"))
                .with_item_type(graphql::LineItemTypeOptions::ConsumptionCharge)
                .with_line_item_grouping(graphql::LineItemGroupingOptions::None)
                .build()?
                );
                response = self.gql_authenticated_request_manager.call(&query).await?;
            }
            else {
                break;
            }
            

        }
        // println!("{}", &response.electricity_agreement_);

        Ok(())
    }

    async fn update_profile(&mut self, account_user: &AccountUserType)  -> Result<(), Error> {

        let api_key = if let Some(profile) = &self.profile {
            profile.api_key.clone()
        }
        else {
            None
        };

        if let Some(new_api_key) = &account_user.live_secret_key_ {
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

    pub async fn handle_bill(&mut self, bill: &BillInterface) -> Result<(), crate::Error> {
        //println!("\n===========================\n{}\n===========================\n", result);
        
        
        let abstract_bill = bill.as_bill_interface();
                // statement.print();

        println!("Energy Account Statement");
        println!("========================");
        println!("Date                 {}", abstract_bill.issued_date_);
        println!("Ref                  {}", abstract_bill.id_);
        println!("From                 {}", abstract_bill.from_date_);
        println!("To                   {}", abstract_bill.to_date_);

        if let BillInterface::StatementType(statement) = bill {
            let mut map = BTreeMap::new();
            for edge in &statement.transactions_.edges_ {
                map.insert(&edge.node_.as_transaction_type().posted_date_, &edge.node_);
            }


            println!();
            print!("{:20} {:10} ", 
                "Title",
                "Date"
            );
            print!("{:10} {:10} {:10} {:10}", 
                "Net", "Tax", "Gross", "Balance c/f"
            );

            println!();

            for txn in &mut map.values() {
                // let txn = transaction.as_transaction_type();

                let title = if let TransactionType::Charge(charge) = txn {
                    if charge.is_export_ {
                        format!("{} Export", txn.as_transaction_type().title_)
                    }
                    else {
                        txn.as_transaction_type().title_.clone()
                    }
                }
                else {
                    txn.as_transaction_type().title_.clone()
                };

                print!("{:20} {:10} ", 
                    title,
                    txn.as_transaction_type().posted_date_
                );
                print!("{:8.2} {:8.2} {:8.2} {:8.2}", 
                    txn.as_transaction_type().amounts_.net_,
                    txn.as_transaction_type().amounts_.tax_, 
                    txn.as_transaction_type().amounts_.gross_,
                    txn.as_transaction_type().balance_carried_forward_
                    );
                print!(" {:10} {:10} {:10} ", "from", "to", "Charge Amount");

                    if let TransactionType::Charge(charge) = txn {
                        if let Some(consumption) = &charge.consumption_ {
                            print!(" {:10} {:10} {:8.2} ", 
                                consumption.start_date_,
                                consumption.end_date_,
                                consumption.quantity_
                            );

                            println!();
                            self.get_account_properties_meters(&consumption.start_date_, &Some(consumption.end_date_.clone())).await?;
                        }

                    }
                println!();
            } 
        }

        Ok(())
    }

    // pub fn get_latest_bill(&self) -> Result<Bill, Error> {
    //     let query = graphql::latest_bill::get_account_summary::Query::new();
    //     let response = self.gql_authenticated_request_manager.call(&query).await?;

    //     Ok(response.viewer_)
    // }



    // pub fn print_statement(&self) {
    //     println!("Energy Account Statement");
    //     println!("========================");
    //     println!("Date                 {}", self.bill.issued_date);
    //     println!("Ref                  {}", self.bill.id);
    //     println!("From                 {}", self.bill.from_date);
    //     println!("To                   {}", self.bill.to_date);
    //     println!();

    //     // let mut map = BTreeMap::new();
    //     // for edge in &self.transactions.edges {
    //     //     let txn = edge.node.as_transaction();

    //     //     map.insert(&txn.posted_date, &edge.node);
    //     // }

    //     print!("{:20} {:10} ", 
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

    //     let mut total_electric_charge = Int::new(0); //0;
    //     let mut total_electric_units = Decimal::new(0, 0);

    //     // for transaction in &mut map.values() {
    //     for edge in (&self.transactions.edges).into_iter().rev() {
    //         let transaction = &edge.node;
    //         let txn = transaction.as_transaction();
    //         if let Transaction::Charge(charge) = &transaction {
    //             if *charge.is_export {
    //                 print!("{} {:width$} ", txn.title, "Export", width = 20 - txn.title.len() - 1);
    //             }
    //             else {
    //                 print!("{} {:width$} ", txn.title, "Import",width =  20 - txn.title.len() - 1);

    //                 if txn.title.eq("Electricity") {
    //                     total_electric_charge += *&txn.amounts.gross;
    //                     total_electric_units += charge.consumption.quantity;
    //                 }
    //             }
    //         }
    //         else {
    //             print!("{:20} ", txn.title);
    //         }
    //         print!("{:10} ", 
    //                     txn.posted_date
    //                 );
    //         print!("{:>10} {:>10} {:>10} {:>10} ", 
    //             txn.amounts.net.as_decimal(2),
    //             txn.amounts.tax.as_decimal(2), 
    //             txn.amounts.gross.as_decimal(2),
    //             txn.balance_carried_forward.as_decimal(2)
    //         );
    //         if let Transaction::Charge(charge) = &transaction {
    //             print!("{:10} {:10} {:>12.4} ", 
    //                 charge.consumption.start_date,
    //                 charge.consumption.end_date,
    //                 charge.consumption.quantity
    //             );

    //             let rate = Decimal::from_int(&txn.amounts.gross) / charge.consumption.quantity;

    //             print!("{:>12.4}", rate); //.round_dp(2));
    //         }
    //         println!();
    //     }

    //     println!("\nTOTALS");

    //     if total_electric_units.is_positive() {
    //         let rate = Decimal::from_int(&total_electric_charge) / total_electric_units;

    //         print!("{:20} {:10} ", 
    //             "Electricity Import",
    //             ""
    //         );
    //         print!("{:>10} {:>10} {:>10} {:>10} ", 
    //             "",
    //             "", 
    //             total_electric_charge.as_decimal(2),
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
}

// unsafe impl Send for Client {

// }

#[async_trait]
impl Module for Client {
    async fn test(&mut self) -> Result<(), crate::Error>{
        let user = self.get_account_user().await?;
        println!("get_account_user {} {} {}", user.given_name_, user.family_name_, user.email_);
        let account = self.get_default_account().await?;
        println!("get_default_account {}", account.number_);
        Ok(())
    }

    async fn summary(&mut self) -> Result<(), crate::Error>{
        let user = self.get_account_user().await?;
        println!("{}", user);
        Ok(())
    }

    async fn bill(&mut self) -> Result<(), crate::Error>{
        let account = self.get_default_account().await?;
        let account_number =  &account.number_; {
            println!("{}", account_number);

            let result = self.get_latest_bill().await?;
            // account_manager.get_latest_bill(&self.gql_client, &mut self.token_manager).await?;

            let bill =  &result.edges_[0].node_;
            self.handle_bill(bill).await?;


            Ok(())
        }
        // else {
        //     Err(crate::Error::InternalError(String::from("Unable to find default account number")))
        // }
    }
}


pub struct ClientBuilder {
    context: Context, 
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
            context: &Context,
            json_profile: Option<serde_json::Value>
        ) -> Result<Box<dyn ModuleBuilder>, Error> {

        let profile: Option<Profile> = if let Some(json) = json_profile {
            serde_json::from_value(json)?
        }
        else {
            None
        };

        let option_api_key= if let Some(args) =  context.args() {
            if let Some(api_key) = &args.octopus.octopus_api_key {
                Some(api_key.to_string())
            }
            else {
                Self::get_profile_api_key(&profile)?
            }
        }
        else {
            Self::get_profile_api_key(&profile)?
        };

        let verbose = if let Some(args) =  context.args() {
            args.verbose
        }
        else {
            false
        };

        let builder = ClientBuilder {
            context: context.clone(),
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

    #[cfg(test)]
    // fn new_test() -> ClientBuilder {
    //     ClientBuilder {
    //         context: Context::new_test(),
    //         profile: None,
    //         gql_client_builder:     sparko_graphql::Client::builder(),
    //         token_manager_builder:  TokenManager::builder(),
    //     }
    // }

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

    pub fn do_build(self, init: bool) -> Result<Client, Error> {
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

        let gql_request_manager = Arc::new(sparko_graphql::RequestManager::new(url.clone(), self.verbose)?);

        let token_manager = self.token_manager_builder
            .with_request_manager(gql_request_manager.clone())
            .with_context(self.context.clone())
            .build(init)?;

        let gql_authenticated_request_manager = sparko_graphql::AuthenticatedRequestManager::new(gql_request_manager, token_manager)?;
       
        let client = Client::new(self.context, option_profile, 
            gql_authenticated_request_manager
          );

        Ok(client)
    }
}

impl ModuleBuilder for ClientBuilder {
    fn build(self: Box<Self>, init: bool) -> Result<Box<dyn crate::Module + Send>, crate::Error> {
        Ok(Box::new(self.do_build(init)?))
    }
}