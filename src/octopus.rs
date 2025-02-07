/*****************************************************************************
MIT License

Copyright (c) 2024 Bruce Skingle

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
******************************************************************************/

pub mod error;
pub mod token;
pub mod account;
pub mod tariff;
pub mod decimal;
pub mod consumption;
pub mod consumption_type;
pub mod transaction;
pub mod bill;
pub mod meter;
pub mod meter_property_view;
pub mod meter_point;
pub mod meter_point_property_view;
pub mod account_property_meters;

use std::{collections::BTreeMap, sync::Arc};
use async_trait::async_trait;
use bill::AccountBillsView;
// use bill::BillResults;
use display_json::DisplayAsJsonPretty;
use graphql::{latest_bill::get_account_latest_bill::{BillInterface, TransactionType}, summary::get_account_summary::AccountUserType};
use serde::{Deserialize, Serialize};

use account::AccountManager;
pub use error::Error;
use sparko_graphql::types::Date;
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
    DEPRECATED_authenticated_request_manager: crate::AuthenticatedRequestManager<OctopusTokenManager>,
    gql_client: Arc<sparko_graphql::Client>,
    pub(crate) token_manager:  OctopusTokenManager,
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
        gql_authenticated_request_manager: sparko_graphql::AuthenticatedRequestManager<OctopusTokenManager>,
        DEPRECATED_authenticated_request_manager: crate::AuthenticatedRequestManager<OctopusTokenManager>,
        gql_client: Arc<sparko_graphql::Client>, token_manager: OctopusTokenManager) -> Client {        

        Client {
            context,
            profile,
            gql_authenticated_request_manager,
            DEPRECATED_authenticated_request_manager,
            gql_client,
            token_manager,
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

    pub fn handle_bill(result: &graphql::latest_bill::get_account_latest_bill::BillConnectionTypeConnection) -> Result<(), crate::Error> {
        //println!("\n===========================\n{}\n===========================\n", result);
        
        let bill =  &result.edges_[0].node_;
        let abstract_bill = bill.as_bill_interface();
                // statement.print();

        println!("Energy Account Statement");
        println!("========================");
        println!("Date                {}", abstract_bill.issued_date_);
        println!("Ref                 {}", abstract_bill.id_);
        println!("From                {}", abstract_bill.from_date_);
        println!("To                  {}", abstract_bill.to_date_);

        if let BillInterface::StatementType(statement) = bill {
            let mut map = BTreeMap::new();
            for edge in &statement.transactions_.edges_ {
                map.insert(&edge.node_.as_transaction_type().posted_date_, &edge.node_);
            }


            for txn in &mut map.values() {
                // let txn = transaction.as_transaction_type();

                print!("{:20} {:10} ", 
                            txn.as_transaction_type().title_,
                            txn.as_transaction_type().posted_date_
                        );
                print!("{:10} {:10} {:10} {:10}", 
                    txn.as_transaction_type().amounts_.net_,
                    txn.as_transaction_type().amounts_.tax_, 
                    txn.as_transaction_type().amounts_.gross_,
                    txn.as_transaction_type().balance_carried_forward_
                    );

                    if let TransactionType::Charge(charge) = txn {
                        if let Some(consumption) = &charge.consumption_ {
                            print!(" {} {} {:10} ", 
                                consumption.start_date_,
                                consumption.end_date_,
                                consumption.quantity_
                            );
                            if charge.is_export_ {
                                print!("export ");
                            }
                            else {
                                print!("import ");
                            }
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

            let account_manager = AccountManager::new(account_number);

            let result = self.get_latest_bill().await?;
            // account_manager.get_latest_bill(&self.gql_client, &mut self.token_manager).await?;

            Self::handle_bill(&result)?;

            // let statement =  &result.edges_[0].node_;


            

            // // if let  bill::Bill::Statement(statement) = &result.edges_[0].node_ {

            //     // let with_effect_from = None; // Some(statement.bill.from_date)
            //     // let meter_result = account_manager.get_account_properties_meters(
            //     //     &self.gql_client, &mut self.token_manager, with_effect_from).await?;

            //     let foo = &statement.consumption_start_date_;

            //     println!("consumption_start_date={:?}", foo);

            //     let start_date = Date::from_calendar_date(2024, time::Month::October, 30).unwrap();

            //     // if let Some(start_date) = &statement.consumption_start_date 
            //     {
            //         let meters = account_manager.get_account_properties_meters(
            //             &mut self.DEPRECATED_authenticated_request_manager,
            //             &start_date,
            //             &statement.consumption_end_date_).await?;
            //         }

            // // }

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
    gql_client_builder:         sparko_graphql::ClientBuilder,
    token_manager_builder:      TokenManagerBuilder,
    url:                        Option<String>,
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

        

        let builder = ClientBuilder {
            context: context.clone(),
            profile,
            gql_client_builder:     sparko_graphql::Client::builder(),
            token_manager_builder:  OctopusTokenManager::builder(),
            url:                    None,
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
        self.gql_client_builder = self.gql_client_builder.with_url(url.clone())?;
        self.url = Some(url);
        Ok(self)
    }

    pub fn with_url_if_not_set(mut self, url: String) -> Result<ClientBuilder, Error> {
        self.gql_client_builder = self.gql_client_builder.with_url_if_not_set(url.clone())?;

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

        let gql_client = Arc::new(
            self.gql_client_builder
                .with_url_if_not_set(String::from("https://api.octopus.energy/v1/graphql/"))?
                .build()?);
        
        let url = if let Some(url) = self.url {
            url
        }
        else {
            "https://api.octopus.energy/v1/graphql/".to_string()
        };

        let gql_request_manager = Arc::new(sparko_graphql::RequestManager::new(url.clone())?);

        let token_manager = self.token_manager_builder
            .with_request_manager(gql_request_manager.clone())
            .with_context(self.context.clone())
            .build(init)?;

        let cloned_token_manager = token_manager.clone_delete_me();
        let cloned2_token_manager = token_manager.clone_delete_me();

        let gql_authenticated_request_manager = sparko_graphql::AuthenticatedRequestManager::new(gql_request_manager, token_manager)?;

        let request_manager = Arc::new(crate::RequestManager::new(url)?);
       
        let DEPRECATED_authenticated_request_manager = crate::AuthenticatedRequestManager::new(request_manager, cloned2_token_manager)?;

        let client = Client::new(self.context, option_profile, 
            gql_authenticated_request_manager,
            DEPRECATED_authenticated_request_manager,
            gql_client.clone(), 
            cloned_token_manager
          );

        Ok(client)
    }
}

impl ModuleBuilder for ClientBuilder {
    fn build(self: Box<Self>, init: bool) -> Result<Box<dyn crate::Module + Send>, crate::Error> {
        Ok(Box::new(self.do_build(init)?))
    }
}