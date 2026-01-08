

pub mod token;
pub mod decimal;
mod account;
mod bill;
mod meter;

use std::sync::Arc;

use anyhow::anyhow;
use account::AccountManager;
use async_trait::async_trait;

use dioxus::prelude::*;

use bill::BillManager;
use meter::MeterManager;
use serde::{Deserialize, Serialize};

use time_tz::{Tz, timezones};
use token::{OctopusTokenManager, TokenManagerBuilder};
use clap::Parser;

use sparko_graphql::TokenManager;
use crate::{CommandProvider, MarcoSparkoContext, Module, ModuleBuilder, ModuleConstructor, PageInfo, ReplCommand, octopus::{bill::BillList, bill::AbstractBill}};

// include!("octopus/graphql.rs");
include!(concat!(env!("OUT_DIR"), "/graphql.rs"));
include!(concat!(env!("OUT_DIR"), "/crate_info.rs"));

#[cfg(graphql_generation_error)]
compile_error!("graphql built with Errors");

// #[cfg(not(graphql_generation_error))]
// compile_error!("NOT graphql built with Errors");

pub type RequestManager = sparko_graphql::AuthenticatedRequestManager<OctopusTokenManager>;

#[derive(Parser, Debug, Clone, PartialEq)]
pub struct OctopusArgs {
    /// The Octopus API_KEY to use
    #[arg(short, long, env)]
    octopus_api_key: Option<String>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub api_key:  Option<String>,
    pub billing_timezone: Option<String>,
    #[serde(skip)]
    // #[serde(default = false)]
    pub init: bool,
}

impl Profile {
    pub fn new() -> Profile {
        Profile {
            api_key: None,
            billing_timezone: Some("Europe/London".to_string()),
            init: false,
        }
    }
}

pub struct Client{
    context: Arc<MarcoSparkoContext>, 
    profile: Option<Profile>,
    // request_manager: Arc<RequestManager>,
    // default_account: Option<Arc<graphql::summary::get_viewer_accounts::AccountInterface>>,
    account_id: String,
    // cache_manager: Arc<CacheManager>,
    bill_manager: Arc<BillManager>,
    meter_manager: Arc<MeterManager>,
    account_manager: AccountManager,
    billing_timezone: &'static time_tz::Tz,
}

const MODULE_ID: &str = "octopus";

#[async_trait(?Send)]
impl CommandProvider for Client {
    async fn exec_repl_command(&mut self, command: &str, args: std::str::SplitWhitespace<'_>) ->  anyhow::Result<()> {
        let account_id = self.account_id.clone();
        match command {
            "bills" => {
                Ok(self.bill_manager
                .bills_handler(args, account_id)
                .await?)
            },
            "bill" => {
                Ok(self.bill_manager.bill_handler(args, account_id, self.billing_timezone).await?)
            },
            "demand" => {
                Ok(self.meter_manager.demand_handler(args, &account_id).await?)
            },
            "consumption" => {
                Ok(self.meter_manager.consumption_handler(args, &account_id, self.billing_timezone).await?)
            },
            _ => Err(anyhow!(format!("Invalid command '{}'", command)))
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
            },

            ReplCommand {
                command:"demand",
                description: "Print electricity demand",
                help:
r#"
usage: demand

Print the current electricity demand (power imported from or exported to the grid)
"#,
            },

            ReplCommand {
                command:"consumption",
                description: "Print electricity consumption",
                help:
r#"
usage: demand

Print the current electricity consumption
"#,
            }
        )
    }
}

impl Client {
    async fn new(context: Arc<MarcoSparkoContext>, profile: Option<Profile>, 
        request_manager: Arc<RequestManager>, verbose: bool) -> anyhow::Result<Client> {   

        let billing_timezone = Self::get_billing_timezone(&profile);
        let cache_manager = context.create_cache_manager(crate::octopus::MODULE_ID, verbose)?;
        let account_manager = AccountManager::new(&cache_manager, &request_manager).await?;
        let meter_manager = Arc::new(MeterManager::new(&cache_manager, &request_manager));
        let bill_manager = Arc::new(BillManager::new(&cache_manager, &request_manager, &meter_manager));

        Ok(Client {
            context,
            profile,
            // request_manager,
            account_id: account_manager.get_default_account_id().to_string(),
            // cache_manager,
            account_manager,
            bill_manager,
            meter_manager,
            billing_timezone,
        })
    }

    fn get_billing_timezone(profile: &Option<Profile>) -> &'static time_tz::Tz {
        if let Some(profile) = profile {
            if let Some(name) = &profile.billing_timezone {
                if let Some(tz) =  timezones::get_by_name(&name) {
                    return tz;
                }
                panic!("Unable to load billing_timezone '{}'", name);
            }
        }
        return timezones::db::europe::LONDON;
    }

    fn get_api_key(&self) -> &Option<String> {
        &self.account_manager.viewer.viewer.viewer_.live_secret_key_
    }

    pub fn registration() -> (String, Box<ModuleConstructor>) {

        // Client::foo(Client::constructor);

        (MODULE_ID.to_string(), Box::new(Client::constructor))
    }
    
    pub fn constructor(context: Arc<MarcoSparkoContext>, 
        json_profile: Option<serde_json::Value>) -> anyhow::Result<Box<dyn ModuleBuilder>> {
            Ok(Client::builder(context, json_profile)?)
    }

    pub fn builder(context: Arc<MarcoSparkoContext>, 
        json_profile: Option<serde_json::Value>
    ) -> anyhow::Result<Box<dyn ModuleBuilder>> {

        ClientBuilder::new(context, json_profile)
    }

    async fn update_profile(&mut self)  -> anyhow::Result<()> {

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
                    crate::profile::update_profile(&self.context.profile.active_profile.name, MODULE_ID, &new_profile)?;
                    // self.context.update_profile(MODULE_ID, new_profile)?;
                }
            }
            else {
                let mut new_profile  = Profile::new();
                new_profile.api_key = Some(new_api_key.clone());

                println!("CREATE profile <{:?}>", &new_profile);
                crate::profile::update_profile(&self.context.profile.active_profile.name, MODULE_ID, &new_profile)?;
                // self.context.update_profile(MODULE_ID, new_profile)?;
            }
        }
        Ok(())
    }
}



fn find_bill<'a>(bill_id: &String, bills: &'a BillList) -> Option<&'a AbstractBill> {
    if let Some((_hash, bill)) = bills.bills.get(bill_id) {
        Some(bill)
    }
    else {
        None
    }
    // for (_id, bill) in &bills.bills {
    //     if bill_id == &bill.as_bill_interface().id_ {
    //         return Some(bill);
    //     }
    // }
    // None
}

#[async_trait]
impl Module for Client {
    fn module_id(&self) -> &'static str {
        MODULE_ID
    }

    fn get_page_list(&self) -> Vec<PageInfo> {
        vec!(
            PageInfo {
                label: "User", 
                path: "user",
            },
            PageInfo {
                label: "Account", 
                path: "account",
            },
            PageInfo {
                label: "Bills",
                path: "bills",
        })
    }

    fn get_component<'a>(&'a self, page_id: &'a str, path: Vec<String>) -> Box<dyn Fn() -> Element + 'a> {
        match page_id {
            "user" => {
                Box::new(|| {
                    // let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
                    let date_format = time::format_description::parse("[year]-[month]-[day]").unwrap();
                    let account_user = &self.account_manager.viewer.viewer.viewer_;
                    let dob = if let Some(date) = &account_user.date_of_birth_ {
                        date.format(&date_format).unwrap()
                    }
                    else {
                        "".to_string()
                    };

                            // tr {
                            //    th { class: "row-header","accounts"} td{"{account_user.accounts_}"}
                            // }
                    rsx! {
                        h1 { "Viewer (Current User)" }
                        table { class: "display",
                            tr {
                                th { class: "row-header", "ID" }
                                td { "{account_user.id_}" }
                            }
                            tr {
                                th { class: "row-header", "Full Name" }
                                td { "{account_user.full_name_}" }
                            }
                            tr {
                                th { class: "row-header", "Title" }
                                td { "{account_user.title_}" }
                            }
                            tr {
                                th { class: "row-header", "Preferred Name" }
                                td { "{account_user.preferred_name_}" }
                            }
                            tr {
                                th { class: "row-header", "Given Name" }
                                td { "{account_user.given_name_}" }
                            }
                            tr {
                                th { class: "row-header", "Family Name" }
                                td { "{account_user.family_name_}" }
                            }
                            tr {
                                th { class: "row-header", "Pronouns" }
                                td { {account_user.pronouns_.as_deref().unwrap_or("")} }
                            }
                            tr {
                                th { class: "row-header", "Email" }
                                td { "{account_user.email_}" }
                            }

                            tr {
                                th { class: "row-header", "Mobile" }
                                td { "{account_user.mobile_}" }
                            }
                            tr {
                                th { class: "row-header", "Landline" }
                                td { "{account_user.landline_}" }
                            }
                            tr {
                                th { class: "row-header", "API Key" }
                                td { {account_user.live_secret_key_.as_deref().unwrap_or("")} }
                            }
                            tr {
                                th { class: "row-header", "Is Deceased" }
                                td { "{account_user.is_deceased_}" }
                            }
                            tr {
                                th { class: "row-header", "Date of Birth" }
                                td { {dob} }
                            }
                            tr {
                                th { class: "row-header", "Alternative Phone Numbers" }
                                td { {format!("{:?}", account_user.alternative_phone_numbers_)} }
                            }
                            tr {
                                th { class: "row-header", "Has Family Issues" }
                                td { "{account_user.has_family_issues_}" }
                            }
                            tr {
                                th { class: "row-header", "Is in Hardship" }
                                td { "{account_user.is_in_hardship_}" }
                            }
                            tr {
                                th { class: "row-header", "Is opted in to Wheel of Fortune" }
                                td { "{account_user.is_opted_in_to_wof_}" }
                            }
                        }

                        h2 { "Accounts" }

                        for account in &account_user.accounts_ {
                            div {
                                h3 { "Account {account.number_}" }
                                table { class: "display",
                                    tr {
                                        th { class: "row-header", "Brand" }
                                        td { "{account.brand_}" }
                                    }
                                    tr {
                                        th { class: "row-header", "Overdue Balance" }
                                        td { "{account.overdue_balance_}" }
                                    }
                                    tr {
                                        th { class: "row-header", "Billing Name" }
                                        td { "{account.billing_name_}" }
                                    }
                                    tr {
                                        th { class: "row-header", "Billing Sub Name" }
                                        td { {account.billing_sub_name_.as_deref().unwrap_or("")} }
                                    }
                                    tr {
                                        th { class: "row-header", "Billing EMail" }
                                        td { {account.billing_email_.as_deref().unwrap_or("")} }
                                    }
                                }
                            }
                        }
                    }
                })
            },
            "account" => {
                Box::new(|| {
                    let account_user = &self.account_manager.viewer.viewer.viewer_;
                    // let x = account_user.full_name_;
                    let api_key = if let Some(api_key) = &account_user.live_secret_key_ {api_key} else {""};
                    rsx! {
                        table { class: "display",
                            tr {
                                th { class: "row-header", "ID" }
                                td { "{account_user.id_}" }
                            }
                            tr {
                                th { class: "row-header", "Full Name" }
                                td { "{account_user.full_name_}" }
                            }
                            tr {
                                th { class: "row-header", "API Key" }
                                td { "{api_key}" }
                            }
                        
                        }
                    }
                })
            },
            "bills" => {
                Box::new(move || {
                    // Create all the signals and actions.

                    // First the list of all bills.
                    let mut bill_list_call_signal = use_signal::<bool>(|| true);

                    let mut bill_list_action = use_action(move |args: (String, Arc<BillManager>)| async move {
                        args.1.fetch_bills(
                            args.0).await
                    });

                    // Initiate the fetch of all bills if we haven;t already done so.
                    if *bill_list_call_signal.read() {
                        bill_list_call_signal.set(false);
                        bill_list_action.call((self.account_id.clone(), self.bill_manager.clone()));
                    }

                    // Now the action to fetch all transactions for one bill
                    let mut bill_transactions_call_signal = use_signal::<Option<String>>(|| None);
                    let mut bill_transactions_action = use_action(
                        | args: (Arc<BillManager>, String, String, 
                        &'static Tz)
                        | async move {
                           let (bm, account_number, statement_id, billing_timezone) = args;

                            bm.fetch_bill_transaction_breakdown(account_number, statement_id, 
                                billing_timezone
                                // timezones::db::europe::LONDON
                            ).await
                        });




                    // Do we have the list of all bills on this account?

                    if let Some(result) = bill_list_action.value() {
                        let bills_signal = result?;
                        let bills = &*bills_signal.read();

                        // Are we looking at one bill?
                        if let Some(bill_id) = path.get(0) {

                            // Yes, have we initiated the fetch of the transactions?
                            let opt_current_bill_id =
                            if let Some(current_bill_id) = &*bill_transactions_call_signal.read() {
                                if current_bill_id != bill_id {
                                    // This is a different bill, so cancel the fetch
                                    bill_transactions_action.cancel();
                                    None
                                }
                                else {
                                    Some(current_bill_id.clone())
                                }
                            }
                            else {
                                None
                            };

                            if opt_current_bill_id.is_none() {
                                // start transaction fetch
                                bill_transactions_call_signal.set(Some(bill_id.clone()));
                                let acid: String = self.account_id.clone();

                                bill_transactions_action.call((self.bill_manager.clone(), acid, bill_id.clone(), self.billing_timezone));
                            }

                            if let Some(bill) = find_bill(bill_id, bills) {
                                if let Some(result) = bill_transactions_action.value() {
                                    let bill_transactions_signal = result?;
                                    let bill_transactions = &*bill_transactions_signal.read();
                                    
                                    bill.gui_display(bill_transactions)
                                }
                                else {
                                    rsx! {
                                        {format!("Loading transactions for bill {}...", bill_id)}
                                    }
                                }
                            }
                            else {
                                rsx! {
                                    {format!("No such bill {bill_id}")}
                                }
                            }
                        }
                        else {
                            rsx! {
                                table {
                                    {AbstractBill::gui_summary_header()?}
                                    for (_id , (_hash , bill)) in &bills.bills {
                                        {bill.gui_summary_line()?}
                                    }
                                }
                            }
                        }
                    }
                    else {
                        rsx! {
                            div { "Loading Bills for account {self.account_id}..." }
                        }
                    }
                })
            },
            _ => {
                Box::new(move || {
                    rsx! {
                        div { "Unknown page_id, {page_id}" }
                    }
                    })
            },
        }
    }
}


pub struct ClientBuilder {
    context: Arc<MarcoSparkoContext>, 
    profile: Option<Profile>,
    token_manager_builder: TokenManagerBuilder,
    url: Option<String>,
    verbose: bool,
}

impl ClientBuilder {

    fn get_profile_api_key(option_profile: &Option<Profile>) -> anyhow::Result<Option<String>> {

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
        ) -> anyhow::Result<Box<dyn ModuleBuilder>> {

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

    pub fn with_url(mut self, url: String) -> anyhow::Result<ClientBuilder> {
        self.url = Some(url);
        Ok(self)
    }

    pub fn with_url_if_not_set(mut self, url: String) -> anyhow::Result<ClientBuilder> {
        if let None = self.url {
            self.url = Some(url);
        }
        Ok(self)
    }

    pub fn with_api_key(mut self, api_key: String) -> anyhow::Result<ClientBuilder> {
        self.token_manager_builder = self.token_manager_builder.with_api_key(api_key);
        Ok(self)
    }

    pub fn with_password(mut self, email: String, password: String) -> anyhow::Result<ClientBuilder> {
        self.token_manager_builder = self.token_manager_builder.with_password(email, password);
        Ok(self)
    }

    pub async fn do_build(self, init: bool) -> anyhow::Result<Client> {
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

        let request_manager = Arc::new(sparko_graphql::RequestManager::new(url.clone(), self.verbose, create_info::USER_AGENT)?);

        let token_manager = self.token_manager_builder
            .with_request_manager(request_manager.clone())
            .with_context(self.context.clone())
            .build(init)?;

        // if init {
        //     let x = token_manager.get_authenticator(true).await;
        //     // println!("HERE {:?}", x);
        //     match x {
        //         Ok(_token) => {
        //             println!("Logged in OK");
        //         },
        //         Err(error) => {
        //             if let sparko_graphql::Error::GraphQLError(graphql_errors) = &error {
        //                 let graphql_errors = &**graphql_errors;
        //                 for graphql_error in graphql_errors {
        //                     if let Some(error_code) = graphql_error.extensions.get("errorCode") {
        //                         if error_code == "KT-CT-1138" {
        //                             println!("Username or password is incorrect.");
        //                             return Err(anyhow!(error));
        //                         }
        //                     }
        //                 }
                 
        //             }
        //             println!("Login failed {}", error);
        //             return Err(anyhow!(error));
        //         },
        //     }
        // }

        let authenticated_request_manager = Arc::new(sparko_graphql::AuthenticatedRequestManager::new(request_manager, token_manager)?);
       
        let mut client = Client::new(self.context, option_profile, 
            authenticated_request_manager, self.verbose
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
    async fn build(self: Box<Self>, init: bool) -> anyhow::Result<Box<dyn crate::Module + Send>> {
        Ok(Box::new(self.do_build(init).await?))
    }
}