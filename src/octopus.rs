

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
use dioxus_router::Navigator;

use bill::BillManager;
use meter::MeterManager;
use serde::{Deserialize, Serialize};

use time_tz::{Tz, timezones};
use token::{OctopusTokenManager};
use clap::Parser;

use sparko_graphql::TokenManager;
use crate::{CacheManager, CommandProvider, MarcoSparkoContext, Module, ModuleFactory, ModuleRegistration, PageInfo, ReplCommand, octopus::{bill::{AbstractBill, BillList}, token::OctopusAuthenticator}};

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
            init: true,
        }
    }
}

pub struct OctopusModule{
    context: Arc<MarcoSparkoContext>, 
    profile: Profile,
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
impl CommandProvider for OctopusModule {
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


#[derive(Deserialize, Debug, Clone)]
struct LoginForm {
    email: Option<String>,
    password: Option<String>,
    api_key: Option<String>,
    login_method: String,
}

impl OctopusModule {
    async fn new(context: Arc<MarcoSparkoContext>, cache_manager: Arc<CacheManager>,profile: Profile, 
        request_manager: Arc<RequestManager>, verbose: bool) -> anyhow::Result<OctopusModule> {   

        let billing_timezone = Self::get_billing_timezone(&profile);
        // let cache_manager = context.create_cache_manager(crate::octopus::MODULE_ID, verbose)?;
        let account_manager = AccountManager::new(&cache_manager, &request_manager).await?;
        let meter_manager = Arc::new(MeterManager::new(&cache_manager, &request_manager));
        let bill_manager = Arc::new(BillManager::new(&cache_manager, &request_manager, &meter_manager));

        Ok(OctopusModule {
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

    fn get_billing_timezone(profile: &Profile) -> &'static time_tz::Tz {
        // if let Some(profile) = profile {
            if let Some(name) = &profile.billing_timezone {
                if let Some(tz) =  timezones::get_by_name(&name) {
                    return tz;
                }
                panic!("Unable to load billing_timezone '{}'", name);
            }
        // }
        return timezones::db::europe::LONDON;
    }

    fn get_api_key(&self) -> &Option<String> {
        &self.account_manager.viewer.viewer.viewer_.live_secret_key_
    }

    pub fn registration() -> ModuleRegistration {

        // Client::foo(Client::constructor);

        ModuleRegistration {
            module_id: MODULE_ID.to_string(),
            constructor: Arc::new(OctopusModule::constructor),
            // init_page_provider: Box::new(Client::init_page_provider),
        }

        // (MODULE_ID.to_string(), Box::new(Client::constructor))
    }
    


    // pub fn init_page_provider(context: Arc<MarcoSparkoContext>, request_manager: Arc<sparko_graphql::RequestManager>) -> Element {
            
    
    //         let mut email = use_signal(|| String::new());
    //         let mut password = use_signal(|| String::new());
    //         let mut api_key = use_signal(|| String::new());
    //         let mut login_method = use_signal(|| "email".to_string());
    //         let mut errors: Signal<Vec<String>>   = use_signal(|| Vec::new());
    //         // let ctx = context.clone();
    //         let profile_name = context.profile.active_profile.name.clone();

    //         rsx! {
    //             for error in errors.read().iter() {
    //                 div { class: "error", "{error}" }
    //             }
    //             div {
    //                 h1 { "Octopus Login" }
    //                 form {
    //                     onsubmit: move |evt: FormEvent| {
    //                         let context = context.clone();
    //                         let request_manager = request_manager.clone();
    //                         async move {
    //                             // Prevent the default browser navigation behavior
    //                             evt.prevent_default();

    //                             println!("evt={:?}", evt);
    //                             // Extract the form values into the LoginForm struct
    //                             let values: LoginForm = evt

    //                                 // In a desktop app, you might print to console, use native APIs
    //                                 // to save to a file, or call a backend server function.
    //                                 // Perform further actions like authentication...
    //                                 .parsed_values()
    //                                 .expect("Failed to parse form values");
    //                             println!("Login attempt for user: {:?}", values);
    //                             // ctx.clone(),
    //                             Self::handle_login(
    //                                     context.clone(),
    //                                     request_manager.clone(),
    //                                     values,
    //                                     &mut errors,
    //                                 )
    //                                 .await;
    //                         }
    //                     },
    //                     table {
    //                         tr {
    //                             td { colspan: "2",
    //                                 label {
    //                                     input {
    //                                         r#type: "radio",
    //                                         name: "login_method",
    //                                         value: "email",
    //                                         checked: login_method() == "email",
    //                                         onchange: move |_| login_method.set("email".to_string()),
    //                                     }
    //                                     " Login with Email and Password"
    //                                 }
    //                             }
    //                         }
    //                         tr {
    //                             td { colspan: "2",
    //                                 label {
    //                                     input {
    //                                         r#type: "radio",
    //                                         name: "login_method",
    //                                         value: "api_key",
    //                                         checked: login_method() == "api_key",
    //                                         onchange: move |_| login_method.set("api_key".to_string()),
    //                                     }
    //                                     " Use API Key"
    //                                 }
    //                             }
    //                         }
    //                         if login_method() == "email" {
    //                             tr {
    //                                 td {
    //                                     label { "Email:" }
    //                                 }
    //                                 td {
    //                                     input {
    //                                         r#type: "email",
    //                                         id: "email",
    //                                         name: "email",
    //                                         value: "{email}",
    //                                         oninput: move |e| email.set(e.value().clone()),
    //                                     }
    //                                 }
    //                             }
    //                             tr {
    //                                 td {
    //                                     label { "Password:" }
    //                                 }
    //                                 td {
    //                                     input {
    //                                         r#type: "password",
    //                                         id: "password",
    //                                         name: "password",
    //                                         value: "{password}",
    //                                         oninput: move |e| password.set(e.value().clone()),
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                         if login_method() == "api_key" {
    //                             tr {
    //                                 td {
    //                                     label { "API Key:" }
    //                                 }
    //                                 td {
    //                                     input {
    //                                         r#type: "text",
    //                                         id: "api_key",
    //                                         name: "api_key",
    //                                         value: "{api_key}",
    //                                         oninput: move |e| api_key.set(e.value().clone()),
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                         tr {
    //                             td {
    //                                 button {
    //                                     // onclick: async move |_| {
    //                                     //     let em = (*email.read()).clone();
    //                                     //     let pw = (*password.read()).clone();
    //                                     //     Self::handle_login(context.clone(), em, pw, &mut errors).await;
    //                                     // },
    //                                     r#type: "submit", // Explicitly set as submit button
    //                                     "Log In"
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }

    // async fn handle_login(
    //     context: Arc<MarcoSparkoContext>,
    //     request_manager: Arc<sparko_graphql::RequestManager>,
    //     // profile_name: &String,
    //     //  email: String, password: String, 
    //     values: LoginForm, 
    //     error_signal: &mut Signal<Vec<String>>) {
    //     let mut errors = Vec::new();

    //     println!("Octopus login submitted for: {:?}", values);
        
    //     let login_method = values.login_method.trim().to_string();
        
    //     if login_method == "email" {
    //         let email = values.email.as_ref().unwrap_or(&String::new()).trim().to_string();
    //         let password = values.password.as_ref().unwrap_or(&String::new()).trim().to_string();
            
    //         if email.trim().is_empty() {
    //             errors.push("Email is required".to_string());
    //         }
    //         if password.trim().is_empty() {
    //             errors.push("Password is required".to_string());
    //         }

    //         if errors.is_empty() {
    //             // Perform login logic here
    //             println!("Performing login for email: {}", email);
    //             // On success, you might want to update the context or navigate to another page

    //             match OctopusTokenManager::builder()
    //                 .with_context(context)
    //                 .with_request_manager(request_manager)
    //                 .with_password(email,password)
    //                 .build(true) {
    //                     Ok(token_manager) => {
    //                         match token_manager.get_authenticator(true).await {
    //                             Ok(_authenticator) => {
    //                                 println!("Login successful!");
    //                                 // Update context or navigate as needed
    //                             },
    //                             Err(e) => {
    //                                 errors.push(format!("Authentication failed: {}", e));
    //                             }
    //                         }
    //                         // Use the authenticator for further operations
    //                     },
    //                     Err(e) => {
    //                         errors.push(format!("Failed to build token manager: {}", e));
    //                     }
    //                 }
    //         }
    //     } else if login_method == "api_key" {
    //         let api_key = values.api_key.as_ref().unwrap_or(&String::new()).trim().to_string();
            
    //         if api_key.trim().is_empty() {
    //             errors.push("API Key is required".to_string());
    //         }

    //         if errors.is_empty() {
    //             println!("Performing login with API key");
                
    //             match OctopusTokenManager::builder()
    //                 .with_api_key(api_key)
    //                 .build(true) {
    //                     Ok(token_manager) => {
    //                         match token_manager.get_authenticator(true).await {
    //                             Ok(_authenticator) => {
    //                                 println!("Login successful!");
    //                                 // Update context or navigate as needed
    //                             },
    //                             Err(e) => {
    //                                 errors.push(format!("Authentication failed: {}", e));
    //                             }
    //                         }
    //                     },
    //                     Err(e) => {
    //                         errors.push(format!("Failed to build token manager: {}", e));
    //                     }
    //                 }
    //         }
    //     } else {
    //         errors.push("Invalid login method".to_string());
    //     }

    //     error_signal.set(errors);
    //     // For now, store the api_key into the active profile if appropriate, or trigger auth flow.
    //     // Example placeholder behaviour:
    //     // let new_profile = Profile {
    //     //     api_key: Some(new_api_key.clone()),
    //     //     ..old_profile.clone()
    //     // };
    //     // crate::profile::update_profile(&context.profile.active_profile.name, MODULE_ID, &new_profile).unwrap_or_else(|e| println!("profile update failed: {}", e));
    // }

    pub fn constructor(context: Arc<MarcoSparkoContext>, 
        json_profile: Option<serde_json::Value>) -> anyhow::Result<Arc<dyn ModuleFactory>> {

            let builder =OctopusModule::builder(context, json_profile)?;

            Ok(Arc::new(builder.build()?))
    }

    pub fn builder(context: Arc<MarcoSparkoContext>, 
        json_profile: Option<serde_json::Value>
    ) -> anyhow::Result<OctopusModuleFactoryBuilder> {

        OctopusModuleFactoryBuilder::new(context, json_profile)
    }

    // async fn update_profile(&mut self)  -> anyhow::Result<()> {

    //     let api_key = if let Some(profile) = &self.profile {
    //         profile.api_key.clone()
    //     }
    //     else {
    //         None
    //     };

    //     if let Some(new_api_key) = self.get_api_key() {
    //         if let Some(old_profile) = &self.profile {
            
    //             if 
    //                 if let Some(old_api_key) = api_key {
    //                     old_api_key.ne(new_api_key)
    //                 }
    //                 else {
    //                     true
    //                 }
    //             {
    //                 // let old_octopus_config = new_profile.octopus_config;
    //                 let new_profile = Profile {
    //                     api_key: Some(new_api_key.clone()),
    //                     ..old_profile.clone()
    //                 };

    //                 println!("UPDATE profile <{:?}>", &new_profile);
    //                 crate::profile::update_profile(&self.context.profile.active_profile.name, MODULE_ID, &new_profile)?;
    //                 // self.context.update_profile(MODULE_ID, new_profile)?;
    //             }
    //         }
    //         else {
    //             let mut new_profile  = Profile::new();
    //             new_profile.api_key = Some(new_api_key.clone());

    //             println!("CREATE profile <{:?}>", &new_profile);
    //             crate::profile::update_profile(&self.context.profile.active_profile.name, MODULE_ID, &new_profile)?;
    //             // self.context.update_profile(MODULE_ID, new_profile)?;
    //         }
    //     }
    //     Ok(())
    // }
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
impl Module for OctopusModule {
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

pub struct OctopusModuleFactory {
    context: Arc<MarcoSparkoContext>,
    cache_manager: Arc<CacheManager>,
    // json_profile: Option<serde_json::Value>,
    token_manager: Arc<OctopusTokenManager>,
    request_manager: Arc<sparko_graphql::RequestManager>,
    profile: Profile,
    verbose: bool,
}

impl OctopusModuleFactory {
    pub async fn do_build(&self) -> anyhow::Result<OctopusModule> {
        // let option_profile = if init {
        //     if let Some(mut profile) = self.profile {
        //         profile.init = true;
        //         Some(profile)
        //     }
        //     else {
        //         let mut profile = Profile::new();
        //         profile.init = true;

        //         Some(profile)
        //     }
        // }
        // else {
        //     self.profile
        // };

        // let url = if let Some(url) = self.url {
        //     url
        // }
        // else {
        //     "https://api.octopus.energy/v1/graphql/".to_string()
        // };

        // let request_manager = Arc::new(sparko_graphql::RequestManager::new(url.clone(), self.verbose, create_info::USER_AGENT)?);

        // let token_manager = self.token_manager_builder
        //     .with_request_manager(request_manager.clone())
        //     .with_context(self.context.clone())
        //     .build(init)?;

        // // if init {
        // //     let x = token_manager.get_authenticator(true).await;
        // //     // println!("HERE {:?}", x);
        // //     match x {
        // //         Ok(_token) => {
        // //             println!("Logged in OK");
        // //         },
        // //         Err(error) => {
        // //             if let sparko_graphql::Error::GraphQLError(graphql_errors) = &error {
        // //                 let graphql_errors = &**graphql_errors;
        // //                 for graphql_error in graphql_errors {
        // //                     if let Some(error_code) = graphql_error.extensions.get("errorCode") {
        // //                         if error_code == "KT-CT-1138" {
        // //                             println!("Username or password is incorrect.");
        // //                             return Err(anyhow!(error));
        // //                         }
        // //                     }
        // //                 }
                 
        // //             }
        // //             println!("Login failed {}", error);
        // //             return Err(anyhow!(error));
        // //         },
        // //     }
        // // }

        let authenticated_request_manager = Arc::new(sparko_graphql::AuthenticatedRequestManager::new(self.request_manager.clone(), self.token_manager.clone())?);
       
        let mut client = OctopusModule::new(self.context.clone(), self.cache_manager.clone(), self.profile.clone(), 
            authenticated_request_manager, self.verbose
        ).await?;

        if self.profile.init {
            // let account_user = client.get_account_user().await?;
            // let x = client.account_manager.viewer.viewer.viewer_.live_secret_key_
            crate::profile::update_profile(&self.context.profile.active_profile.name, MODULE_ID, &self.profile)?;
            // client.update_profile().await?;
        }
        
        Ok(client)
    }

    async fn login(errors: &mut Vec<String>, token_manager: &Arc<OctopusTokenManager>) -> anyhow::Result<()> {

            let x = token_manager.get_authenticator(true).await;
            // println!("HERE {:?}", x);
            match x {
                Ok(_token) => {
                    println!("Logged in OK");
                    Ok(())
                },
                Err(error) => {
                    if let sparko_graphql::Error::GraphQLError(graphql_errors) = &error {
                        let graphql_errors = &**graphql_errors;
                        for graphql_error in graphql_errors {
                            if let Some(error_code) = graphql_error.extensions.get("errorCode") {
                                if error_code == "KT-CT-1138" {
                                    errors.push(format!("Username or password is incorrect."));
                                    return Err(anyhow!(error));
                                }
                                if error_code == "KT-CT-1139" {
                                    errors.push(format!("API KEY is incorrect."));
                                    return Err(anyhow!(error));
                                }
                            }
                        }
                 
                    }
                    errors.push(format!("Login failed {}", error));
                    return Err(anyhow!(error));
                },
            }
    }

    async fn handle_login(
        token_manager: Arc<OctopusTokenManager>,
        profile: Profile,
        context: Arc<MarcoSparkoContext>,
        values: LoginForm, 
        error_signal: &mut Signal<Vec<String>>) {
        let mut errors = Vec::new();

        println!("Octopus login submitted for: {:?}", values);
        
        let login_method = values.login_method.trim().to_string();
        
        if login_method == "email" {
            let email = values.email.as_ref().unwrap_or(&String::new()).trim().to_string();
            let password = values.password.as_ref().unwrap_or(&String::new()).trim().to_string();
            
            if email.is_empty() {
                errors.push("Email is required".to_string());
            }
            if password.is_empty() {
                errors.push("Password is required".to_string());
            }

            if errors.is_empty() {
                // Perform login logic here
                println!("Performing login for email: {}", email);
                // On success, you might want to update the context or navigate to another page
                token_manager.set_authenticator(
                    OctopusAuthenticator::from_email_password(email.clone(), password.clone())
                ).await;

                let _ = Self::login(&mut errors, &token_manager).await;

                // Reset the app initialization to reload context with new profile
                let init_signal = try_consume_context::<Signal<bool>>();
                if let Some(mut init_sig) = init_signal {
                    init_sig.set(true);
                }
                // match token_manager.get_authenticator(true).await {
                //     Ok(_authenticator) => {
                //         println!("Login successful!");
                //     },
                //     Err(e) => {
                //         errors.push(format!("Authentication failed: {}", e));
                //     }
                // }
            }
        } else if login_method == "api_key" {
            let api_key = values.api_key.as_ref().unwrap_or(&String::new()).trim().to_string();
            
            if api_key.is_empty() {
                errors.push("API Key is required".to_string());
            }

            if errors.is_empty() {
                println!("Performing login with API key");
                token_manager.set_authenticator(
                    OctopusAuthenticator::from_api_key(api_key.clone())
                ).await;
                // match token_manager.get_authenticator(true).await {
                match Self::login(&mut errors, &token_manager).await {
                    Ok(_authenticator) => {
                        println!("Login successful!");
                        // Store the api_key into the profile
                        let new_profile = Profile {
                            api_key: Some(values.api_key.as_ref().unwrap_or(&String::new()).trim().to_string()),
                            ..profile.clone()
                        };

                        crate::profile::update_profile(&context.profile.active_profile.name, MODULE_ID, &new_profile).unwrap_or_else(|e| println!("profile update failed: {}", e));

                        // Reset the app initialization to reload context with new profile
                        let mut init_signal = try_consume_context::<Signal<bool>>();
                        if let Some(mut init_sig) = init_signal {
                            init_sig.set(true);
                        }

                        // // Navigate to the module page
                        // navigator.push(format!("/blog/{}", MODULE_ID));

                        // //aaaaaa
                        // let api_key = if let Some(profile) = &self.profile {
                        //     profile.api_key.clone()
                        // }
                        // else {
                        //     None
                        // };

                        // if let Some(new_api_key) = self.get_api_key() {
                        //     if let Some(old_profile) = &self.profile {
                            
                        //         if 
                        //             if let Some(old_api_key) = api_key {
                        //                 old_api_key.ne(new_api_key)
                        //             }
                        //             else {
                        //                 true
                        //             }
                        //         {
                        //             // let old_octopus_config = new_profile.octopus_config;
                        //             let new_profile = Profile {
                        //                 api_key: Some(new_api_key.clone()),
                        //                 ..old_profile.clone()
                        //             };

                        //             println!("UPDATE profile <{:?}>", &new_profile);
                        //             crate::profile::update_profile(&self.context.profile.active_profile.name, MODULE_ID, &new_profile)?;
                        //             // self.context.update_profile(MODULE_ID, new_profile)?;
                        //         }
                        //     }
                        //     else {
                        //         let mut new_profile  = Profile::new();
                        //         new_profile.api_key = Some(new_api_key.clone());

                        //         println!("CREATE profile <{:?}>", &new_profile);
                        //         crate::profile::update_profile(&self.context.profile.active_profile.name, MODULE_ID, &new_profile)?;
                        //         // self.context.update_profile(MODULE_ID, new_profile)?;
                        //     }
                        // }
                        // //sssss
                    },
                    Err(e) => {
                        // errors.push(format!("Authentication failed: {}", e));
                    }
                }
            }
        } else {
            errors.push("Invalid login method".to_string());
        }

        error_signal.set(errors);
        // For now, store the api_key into the active profile if appropriate, or trigger auth flow.
        // Example placeholder behaviour:
        // let new_profile = Profile {
        //     api_key: Some(new_api_key.clone()),
        //     ..old_profile.clone()
        // };
        // crate::profile::update_profile(&context.profile.active_profile.name, MODULE_ID, &new_profile).unwrap_or_else(|e| println!("profile update failed: {}", e));
    }
}

#[async_trait]
impl ModuleFactory for OctopusModuleFactory {

    async fn is_ready(&self) -> anyhow::Result<bool> {
        if let Ok(_token) = self.token_manager.get_authenticator(false).await {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn init_page(&self) -> Element {
        let mut email = use_signal(|| String::new());
        let mut password = use_signal(|| String::new());
        let mut api_key = use_signal(|| String::new());
        let mut login_method = use_signal(|| "email".to_string());
        let mut errors: Signal<Vec<String>>   = use_signal(|| Vec::new());
        // let ctx = context.clone();
        // let profile_name = self.context.profile.active_profile.name.clone();

        let context: Arc<MarcoSparkoContext> = self.context.clone();
        let request_manager: Arc<sparko_graphql::RequestManager> = self.request_manager.clone();
        let token_manager: Arc<OctopusTokenManager> = self.token_manager.clone();
        let profile: Profile = self.profile.clone();
        rsx! {
            for error in errors.read().iter() {
                div { class: "error", "{error}" }
            }
            div {
                h1 { "Octopus Login" }
                form {
                    onsubmit: move |evt: FormEvent| {
                        let context = context.clone();
                        // let request_manager = request_manager.clone();
                        let token_manager = token_manager.clone();
                        let profile = profile.clone();
                        async move {
                            // Prevent the default browser navigation behavior
                            evt.prevent_default();

                            println!("evt={:?}", evt);
                            // Extract the form values into the LoginForm struct
                            let values: LoginForm = evt

                                // In a desktop app, you might print to console, use native APIs
                                // to save to a file, or call a backend server function.
                                // Perform further actions like authentication...
                                .parsed_values()
                                .expect("Failed to parse form values");
                            println!("Login attempt for user: {:?}", values);
                            // ctx.clone(),
                            Self::handle_login(
                                    token_manager,
                                    profile,
                                    context.clone(),
                                    values,
                                    &mut errors,
                                )
                                .await;
                        }
                    },
                    table {
                        tr {
                            td { colspan: "2",
                                label {
                                    input {
                                        r#type: "radio",
                                        name: "login_method",
                                        value: "email",
                                        checked: login_method() == "email",
                                        onchange: move |_| login_method.set("email".to_string()),
                                    }
                                    " Login with Email and Password"
                                }
                            }
                        }
                        tr {
                            td { colspan: "2",
                                label {
                                    input {
                                        r#type: "radio",
                                        name: "login_method",
                                        value: "api_key",
                                        checked: login_method() == "api_key",
                                        onchange: move |_| login_method.set("api_key".to_string()),
                                    }
                                    " Use API Key"
                                }
                            }
                        }
                        if login_method() == "email" {
                            tr {
                                td {
                                    label { "Email:" }
                                }
                                td {
                                    input {
                                        r#type: "email",
                                        id: "email",
                                        name: "email",
                                        value: "{email}",
                                        oninput: move |e| email.set(e.value().clone()),
                                    }
                                }
                            }
                            tr {
                                td {
                                    label { "Password:" }
                                }
                                td {
                                    input {
                                        r#type: "password",
                                        id: "password",
                                        name: "password",
                                        value: "{password}",
                                        oninput: move |e| password.set(e.value().clone()),
                                    }
                                }
                            }
                        }
                        if login_method() == "api_key" {
                            tr {
                                td {
                                    label { "API Key:" }
                                }
                                td {
                                    input {
                                        r#type: "text",
                                        id: "api_key",
                                        name: "api_key",
                                        value: "{api_key}",
                                        oninput: move |e| api_key.set(e.value().clone()),
                                    }
                                }
                            }
                        }
                        tr {
                            td {
                                button {
                                    // onclick: async move |_| {
                                    //     let em = (*email.read()).clone();
                                    //     let pw = (*password.read()).clone();
                                    //     Self::handle_login(context.clone(), em, pw, &mut errors).await;
                                    // },
                                    r#type: "submit", // Explicitly set as submit button
                                    "Log In"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    async fn build(&self) -> anyhow::Result<Box<dyn crate::Module + Send>> {
        Ok(Box::new(self.do_build().await?))
    }
}

pub struct OctopusModuleFactoryBuilder {
    context: Arc<MarcoSparkoContext>, 
    profile: Profile,
    authenticator: Option<OctopusAuthenticator>,
    url: Option<String>,
    verbose: bool,
}

impl OctopusModuleFactoryBuilder {

    // fn get_profile_api_key(option_profile: &Option<Profile>) -> anyhow::Result<Option<String>> {

    //     if let Some(profile) =  option_profile {
    //         if let Some(api_key) = &profile.api_key {
    //             return Ok(Some(api_key.to_string()))
    //         }
    //     }

    //     Ok(None)
    // }

    fn new(
            context: Arc<MarcoSparkoContext>,
            json_profile: Option<serde_json::Value>
        ) -> anyhow::Result<OctopusModuleFactoryBuilder> {

        let profile = if let Some(json) = json_profile {
            serde_json::from_value(json)?
        }
        else {
            Profile::new()
        };

        let option_api_key = if let Some(api_key) = &context.args.octopus.octopus_api_key {
            Some(api_key.to_string())
        }
        else {
            // Self::get_profile_api_key(&profile)?
            if let Some(api_key) = &profile.api_key {
                Some(api_key.to_string())
            }
            else {
                None
            }
        };

        let authenticator = if let Some(api_key) = option_api_key {
            Some(OctopusAuthenticator::from_api_key(api_key))
        }
        else {
            None
        };

        let verbose = context.args.verbose;

        Ok(OctopusModuleFactoryBuilder {
            context,
            profile,
            authenticator,
            url: None,
            verbose,
        })

        // if let Some(api_key) = option_api_key {
        //     Ok(Arc::new(builder.with_api_key(api_key)?))
        // }
        // else {
        //     Ok(Arc::new(builder))
        // }
        
    }

    pub fn with_url(mut self, url: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
        self.url = Some(url);
        Ok(self)
    }

    pub fn with_url_if_not_set(mut self, url: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
        if let None = self.url {
            self.url = Some(url);
        }
        Ok(self)
    }

    pub fn with_api_key(mut self, api_key: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
        self.authenticator = Some(OctopusAuthenticator::from_api_key(api_key));
        Ok(self)
    }

    pub fn with_password(mut self, email: String, password: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
        self.authenticator = Some(OctopusAuthenticator::from_email_password(email, password));
        Ok(self)
    }

    pub fn build(self) -> anyhow::Result<OctopusModuleFactory> {
        let url = if let Some(url) = self.url {
            url
        }
        else {
            "https://api.octopus.energy/v1/graphql/".to_string()
        };

        let verbose = self.context.args.verbose;
        let request_manager = Arc::new(sparko_graphql::RequestManager::new(url, self.context.args.verbose, create_info::USER_AGENT)?);
        let cache_manager = self.context.create_cache_manager(crate::octopus::MODULE_ID, verbose)?;

        Ok(OctopusModuleFactory {
            context: self.context.clone(),
            cache_manager,
            token_manager: Arc::new(OctopusTokenManager::new(
                self.context,
                 request_manager.clone(),
                 self.authenticator)),
            request_manager,
            profile: self.profile,
            verbose: self.verbose,
        })
    }
}
