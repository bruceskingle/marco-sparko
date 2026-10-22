

mod token;
mod decimal;
mod viewer_data_set;
mod property;
mod meter_point;
mod bill;
mod meter;
mod line_items;
mod meter_agreement;
mod measurements;
mod consumption;
mod consumption_graph;
mod stacked_bar_graph;
mod trend_graph;
mod category_filter;
mod context;
mod pages;

use std::sync::Arc;

use anyhow::anyhow;
use viewer_data_set::ViewerDataSet;
use async_trait::async_trait;

use dioxus::prelude::*;

use context::{OctopusContext, OctopusRoot};
use pages::{BillsPage, MeterPage, MeasurementsPage, PropertiesPage};
use serde::{Deserialize, Serialize};

use time_tz::timezones;
use token::{OctopusTokenManager};
use clap::Parser;

use sparko_graphql::TokenManager;
use crate::{CacheConfig, InitRequested, MarcoSparkoContext, Module, ModuleFactory, ModuleRegistration, PageInfo, octopus::token::OctopusAuthenticator};

// include!("octopus/graphql.rs");
include!(concat!(env!("OUT_DIR"), "/graphql.rs"));
include!(concat!(env!("OUT_DIR"), "/crate_info.rs"));

#[cfg(graphql_generation_error)]
compile_error!("graphql built with Errors");


pub type RequestManager = sparko_graphql::AuthenticatedRequestManager<OctopusTokenManager>;


/*
 * It is important that the names of all args here begin with octopus_
 * as only args beginning with octopus- on the command line are passed through to the module.
 * 
 * Underscores in the names of fields on this struct are translated into hyphens on the command line automatically by clap.
 */
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
    octopus_args: OctopusArgs,
    account_manager: ViewerDataSet,
    /// Shared with the pages, see OctopusRoot.
    context: Arc<OctopusContext>,
}

const MODULE_ID: &str = "octopus";


#[derive(Deserialize, Debug, Clone)]
struct LoginForm {
    email: Option<String>,
    password: Option<String>,
    api_key: Option<String>,
    login_method: String,
}

impl OctopusModule {
    async fn new(octopus_args: OctopusArgs, cache_config: Arc<CacheConfig>, profile: Profile, 
        request_manager: Arc<RequestManager>, _verbose: bool) -> anyhow::Result<OctopusModule> {   

        let billing_timezone = Self::get_billing_timezone(&profile);
        let account_manager = ViewerDataSet::new(&cache_config, &*request_manager).await?;

        let context = Arc::new(OctopusContext {
            account_id: account_manager.get_default_account_id().to_string(),
            billing_timezone,
            cache_config,
            request_manager,
        });

        Ok(OctopusModule {
            octopus_args,
            account_manager,
            context,
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

    pub fn registration() -> ModuleRegistration {
        ModuleRegistration {
            module_id: MODULE_ID.to_string(),
            constructor: Arc::new(OctopusModule::constructor),
        }
    }
    
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
}


#[async_trait]
impl Module for OctopusModule {
    fn cli_debug(&self) -> Result<(), anyhow::Error> {
        println!("Args: {:?}", self.octopus_args);
        println!("Octopus Module: account_id={}", self.context.account_id);
        Ok(())
    }

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
            },
            PageInfo {
                label: "Properties",
                path: "properties",
            },
            PageInfo {
                label: "Meters",
                path: "meters",
            },
            PageInfo {
                label: "Measurements",
                path: "measurements",
            },
        )
    }

    fn get_component(&self, page_id: &str, path: Vec<String>) -> Element {
        let page = match page_id {
            "user" => {
                // let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
                let date_format = time::format_description::parse("[year]-[month]-[day]").unwrap();
                let account_user = &self.account_manager.data_set.data.viewer_;
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
            },
            "account" => {
                let account_user = &self.account_manager.data_set.data.viewer_;
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
            },
            "bills" => {
                rsx! {
                    BillsPage { path }
                }
            },
            "properties" => {
                rsx! {
                    PropertiesPage { path }
                }
            },
            "meters" => {
                rsx! {
                    MeterPage { path }
                }
            },
            "measurements" => {
                rsx! {
                    MeasurementsPage { path }
                }
            },
            _ => {
                rsx! {
                    div { "Unknown page_id, {page_id}" }
                }
            },
        };

        // The key changes if the module is rebuilt, so that OctopusRoot is re-created and provides the new context.
        let context_key = format!("{:p}", Arc::as_ptr(&self.context));
        rsx! {
            OctopusRoot { key: "{context_key}", context: self.context.clone(), {page} }
        }
    }
}


pub struct OctopusModuleFactory {
    context: Arc<MarcoSparkoContext>,
    octopus_args: OctopusArgs,
    cache_config: Arc<CacheConfig>,
    token_manager: Arc<OctopusTokenManager>,
    request_manager: Arc<sparko_graphql::RequestManager>,
    profile: Profile,
    verbose: bool,
}

impl OctopusModuleFactory {
    pub async fn do_build(&self) -> anyhow::Result<OctopusModule> {

        let authenticated_request_manager = Arc::new(sparko_graphql::AuthenticatedRequestManager::new(self.request_manager.clone(), self.token_manager.clone())?);
       
        let client = OctopusModule::new(self.octopus_args.clone(), self.cache_config.clone(), self.profile.clone(), 
            authenticated_request_manager, self.verbose
        ).await?;

        if self.profile.init {
            crate::profile::update_profile(&self.context.profile.active_profile.name, MODULE_ID, &self.profile)?;
        }
        
        Ok(client)
    }

    async fn login(errors: &mut Vec<String>, token_manager: &Arc<OctopusTokenManager>) -> anyhow::Result<()> {

            match token_manager.get_authenticator(true).await {
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
                println!("Performing login for email: {}", email);
                token_manager.set_authenticator(
                    OctopusAuthenticator::from_email_password(email.clone(), password.clone())
                ).await;

                if let Ok(_) =  Self::login(&mut errors, &token_manager).await {
                    let init_signal = try_consume_context::<Signal<InitRequested>>();
                    if let Some(mut init_sig) = init_signal {
                        init_sig.set(InitRequested(true));
                    }
                }

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
                if let Ok(_) = Self::login(&mut errors, &token_manager).await {
                    println!("Login successful!");
                    // Store the api_key into the profile
                    let new_profile = Profile {
                        api_key: Some(values.api_key.as_ref().unwrap_or(&String::new()).trim().to_string()),
                        ..profile.clone()
                    };

                    crate::profile::update_profile(&context.profile.active_profile.name, MODULE_ID, &new_profile).unwrap_or_else(|e| println!("profile update failed: {}", e));

                    // Reset the app initialization to reload context with new profile
                    let init_signal = try_consume_context::<Signal<InitRequested>>();
                    if let Some(mut init_sig) = init_signal {
                        init_sig.set(InitRequested(true));
                    }
                }
            }
        } else {
            errors.push("Invalid login method".to_string());
        }

        error_signal.set(errors);
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

        let context: Arc<MarcoSparkoContext> = self.context.clone();
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
                        let token_manager = token_manager.clone();
                        let profile = profile.clone();
                        async move {
                            // Prevent the default browser navigation behavior
                            evt.prevent_default();

                            // Extract the form values into the LoginForm struct
                            let values: LoginForm = evt
                                .parsed_values()
                                .expect("Failed to parse form values");

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
                                button { r#type: "submit", "Log In" }
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
    octopus_args: OctopusArgs,
    profile: Profile,
    authenticator: Option<OctopusAuthenticator>,
    url: Option<String>,
    verbose: bool,
}

impl OctopusModuleFactoryBuilder {
    fn new(
            context: Arc<MarcoSparkoContext>,
            json_profile: Option<serde_json::Value>
        ) -> anyhow::Result<OctopusModuleFactoryBuilder> {

        let module_args = if let Some(args) = context.args.module_args(MODULE_ID) {
            args
        }
        else {
            Vec::new()
        };

        let octopus_args = OctopusArgs::parse_from(module_args);
        let profile = if let Some(json) = json_profile {
            serde_json::from_value(json)?
        }
        else {
            Profile::new()
        };

        let option_api_key = if let Some(api_key) = &octopus_args.octopus_api_key {
            Some(api_key.to_string())
        }
        else {
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

        let verbose = context.args.marco_sparko_args.verbose;

        if verbose {
            println!("Octopus Module Args: {:?}", octopus_args);
            println!("Octopus Module Profile: {:?}", profile);
        }  

        Ok(OctopusModuleFactoryBuilder {
            context,
            octopus_args,
            profile,
            authenticator,
            url: None,
            verbose,
        })
    }

    // pub fn with_url(mut self, url: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
    //     self.url = Some(url);
    //     Ok(self)
    // }

    // pub fn with_url_if_not_set(mut self, url: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
    //     if let None = self.url {
    //         self.url = Some(url);
    //     }
    //     Ok(self)
    // }

    // pub fn with_api_key(mut self, api_key: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
    //     self.authenticator = Some(OctopusAuthenticator::from_api_key(api_key));
    //     Ok(self)
    // }

    // pub fn with_password(mut self, email: String, password: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
    //     self.authenticator = Some(OctopusAuthenticator::from_email_password(email, password));
    //     Ok(self)
    // }

    pub fn build(self) -> anyhow::Result<OctopusModuleFactory> {
        let url = if let Some(url) = self.url {
            url
        }
        else {
            "https://api.octopus.energy/v1/graphql/".to_string()
        };

        let verbose = self.context.args.marco_sparko_args.verbose;
        let request_manager = Arc::new(sparko_graphql::RequestManager::new(url, verbose, create_info::USER_AGENT)?);
        let cache_config = self.context.create_cache_config(crate::octopus::MODULE_ID, verbose)?;

        Ok(OctopusModuleFactory {
            context: self.context.clone(),
            octopus_args: self.octopus_args,
            cache_config,
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
