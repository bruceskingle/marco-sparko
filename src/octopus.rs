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

use time_tz::timezones;
use token::{OctopusTokenManager, TokenManagerBuilder};
use clap::Parser;

use sparko_graphql::TokenManager;
use crate::{error::Error, CacheManager, CommandProvider, MarcoSparkoContext, Module, ModuleBuilder, ModuleConstructor, ReplCommand};

include!(concat!(env!("OUT_DIR"), "/graphql.rs"));
include!(concat!(env!("OUT_DIR"), "/crate_info.rs"));

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
    request_manager: Arc<RequestManager>,
    // default_account: Option<Arc<graphql::summary::get_viewer_accounts::AccountInterface>>,
    account_id: String,
    cache_manager: Arc<CacheManager>,
    bill_manager: BillManager,
    meter_manager: MeterManager,
    account_manager: AccountManager,
    billing_timezone: &'static time_tz::Tz,
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
                Ok(self.bill_manager.bill_handler(args, &account_id, &mut self.meter_manager, self.billing_timezone).await?)
            },
            "demand" => {
                Ok(self.meter_manager.demand_handler(args, &account_id, self.billing_timezone).await?)
            },
            _ => Err(Error::from(format!("Invalid command '{}'", command)))
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
            }
        )
    }
}

impl Client {
    async fn new(context: Arc<MarcoSparkoContext>, profile: Option<Profile>, 
        request_manager: Arc<RequestManager>, verbose: bool) -> Result<Client, Error> {   

        let billing_timezone = Self::get_billing_timezone(&profile);
        let cache_manager = context.create_cache_manager(crate::octopus::MODULE_ID, verbose)?;
        let account_manager = AccountManager::new(&cache_manager, &request_manager).await?;
        let bill_manager = BillManager::new(&cache_manager, &request_manager);
        let meter_manager = MeterManager::new(&cache_manager, &request_manager, &billing_timezone);

        Ok(Client {
            context,
            profile,
            request_manager,
            account_id: account_manager.get_default_account_id().to_string(),
            cache_manager,
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
        json_profile: Option<serde_json::Value>) -> Result<Box<dyn ModuleBuilder>, crate::Error> {
            Ok(Client::builder(context, json_profile)?)
    }

    pub fn builder(context: Arc<MarcoSparkoContext>, 
        json_profile: Option<serde_json::Value>
    ) -> Result<Box<dyn ModuleBuilder>, Error> {

        ClientBuilder::new(context, json_profile)
    }

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

                    //println!("UPDATE profile <{:?}>", &new_profile);

                    self.context.update_profile(MODULE_ID, new_profile)?;
                }
            }
            else {
                let mut new_profile  = Profile::new();
                new_profile.api_key = Some(new_api_key.clone());

                //println!("CREATE profile <{:?}>", &new_profile);
                self.context.update_profile(MODULE_ID, new_profile)?;
            }
        }
        Ok(())
    }
}


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

        let request_manager = Arc::new(sparko_graphql::RequestManager::new(url.clone(), self.verbose, CrateInfo::USER_AGENT)?);

        let token_manager = self.token_manager_builder
            .with_request_manager(request_manager.clone())
            .with_context(self.context.clone())
            .build(init)?;

        if init {
            let x = token_manager.get_authenticator(true).await;
            // println!("HERE {:?}", x);
            match x {
                Ok(token) => {
                    println!("Logged in OK");
                },
                Err(error) => {
                    if let sparko_graphql::Error::GraphQLError(graphql_errors) = &error {
                        for graphql_error in graphql_errors {
                            if let Some(error_code) = graphql_error.extensions.get("errorCode") {
                                if error_code == "KT-CT-1138" {
                                    println!("Username or password is incorrect.");
                                    return Err(Error::from(error));
                                }
                            }
                        }
                 
                    }
                    println!("Login failed {}", error);
                    return Err(Error::from(error));
                },
            }
        }

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
    async fn build(self: Box<Self>, init: bool) -> Result<Box<dyn crate::Module + Send>, crate::Error> {
        Ok(Box::new(self.do_build(init).await?))
    }
}