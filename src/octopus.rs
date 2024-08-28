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
mod account;
mod tariff;
mod page_info;
mod decimal;
mod consumption;
mod transaction;
mod bill;

use std::sync::Arc;
use async_trait::async_trait;
use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

use account::{AccountInterface, AccountUser};
pub use error::Error;
use token::{TokenManager, TokenManagerBuilder};
use clap::Parser;

use crate::{Context, Module, ModuleBuilder, ModuleConstructor};

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

     #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
     #[serde(rename_all = "camelCase")]
     pub struct PossibleErrorType {
        message: Option<String>,
        // locations: Vec<Location>,
        // path: Vec<String>,
        // extensions: Extensions,



        // "The error code that might be returned from the query/mutation."
        code: Option<String>,
        // "The error description that might be returned from the query/mutation."
        description: Option<String>,
        // "The error message that might be returned from the query/mutation."
        
        // "The error type that might be returned from the query/mutation."
        #[serde(rename = "type")]
        type_name: Option<String>,
    }

    impl PossibleErrorType {
        pub fn to_string(errors: Vec<PossibleErrorType>) -> String {
            let mut result = String::new();

            for err in errors {
                if result.len() == 0 {
                    result.push('[');
                }
                else {
                    result.push(',');
                }

                result.push_str(&err.to_string());
            }
            result.push(']');
            result
        }
    }



    // impl fmt::Display for PossibleErrorType {
    //     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //         write!(f, "[")?;

    //         if let Some(code) = &self.code {
    //             write!(f, "code: {}", code)?
    //         }

    //         if let Some(description) = &self.description {
    //             write!(f, "description: {}", description)?
    //         }

    //         if let Some(message) = &self.message {
    //             write!(f, "message: {}", message)?
    //         }

    //         if let Some(type_name) = &self.type_name {
    //             write!(f, "type: {}", type_name)?
    //         }

    //         // Close the opened bracket and return a fmt::Result value.
    //         write!(f, "]")
    //     }
    // }


    //  self.config.get_active_profile()?.modules.octopus.clone()
// #[derive(Debug)]
pub struct Client{
    context: Context, 
    profile: Option<Profile>,
    gql_client:     Arc<crate::gql::Client>,
    pub(crate) token_manager:  TokenManager,
    default_account: Option<Arc<AccountInterface>>
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

    fn new(context: Context, profile: Option<Profile>, gql_client: Arc<crate::gql::Client>, token_manager: TokenManager) -> Client {        
        Client {
            context,
            profile,
            gql_client,
            token_manager,
            default_account: None
        }
    }

    // pub async fn authenticate(&mut self) -> Result<Arc<std::string::String>, Error>{
    //     self.token_manager.authenticate().await
    // }

    pub async fn get_default_account(&mut self)  -> Result<Arc<AccountInterface>, Error> {
        if let Some(default_account) = &self.default_account {
            Ok(default_account.clone())
        }
        else {
            let default_account = Arc::new(AccountInterface::get_default_account(&self.gql_client, &mut self.token_manager).await?);
            let return_value = default_account.clone();
            self.default_account = Some(default_account);
            Ok(return_value)
        }
    }

    pub async fn get_account_user(&mut self)  -> Result<AccountUser, Error> {
        let account_user = AccountUser::get_account_user(&self.gql_client, &mut self.token_manager).await?;
        
        self.update_profile(&account_user).await?;
        
        Ok(account_user)
    }

    async fn update_profile(&mut self, account_user: &AccountUser)  -> Result<(), Error> {

        let api_key = if let Some(profile) = &self.profile {
            profile.api_key.clone()
        }
        else {
            None
        };

        if let Some(new_api_key) = &account_user.live_secret_key {
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
}

// unsafe impl Send for Client {

// }

#[async_trait]
impl Module for Client {
    async fn summary(&mut self) -> Result<(), crate::Error>{
        let user = self.get_account_user().await?;
        println!("{}", user);
        Ok(())
    }

    async fn bill(&mut self) -> Result<(), crate::Error>{
        let account = self.get_default_account().await?;
        if let Some(account_number) =  &account.number {
            println!("{}", account_number);
            Ok(())
        }
        else {
            Err(crate::Error::InternalError(String::from("Unable to find default account number")))
        }
    }
}


pub struct ClientBuilder {
    context: Context, 
    profile: Option<Profile>,
    gql_client_builder:         crate::gql::ClientBuilder,
    token_manager_builder:      TokenManagerBuilder,
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
            gql_client_builder:     crate::gql::Client::builder(),
            token_manager_builder:  TokenManager::builder(),
        };

        if let Some(api_key) = option_api_key {
            Ok(Box::new(builder.with_api_key(api_key)?))
        }
        else {
            Ok(Box::new(builder))
        }
        
    }

    #[cfg(test)]
    fn new_test() -> ClientBuilder {
        ClientBuilder {
            context: Context::new_test(),
            profile: None,
            gql_client_builder:     crate::gql::Client::builder(),
            token_manager_builder:  TokenManager::builder(),
        }
    }

    pub fn with_url(mut self, url: String) -> Result<ClientBuilder, Error> {
        self.gql_client_builder = self.gql_client_builder.with_url(url)?;
        Ok(self)
    }

    pub fn with_url_if_not_set(mut self, url: String) -> Result<ClientBuilder, Error> {
        self.gql_client_builder = self.gql_client_builder.with_url_if_not_set(url)?;
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

        let client = Client::new(self.context, option_profile, gql_client.clone(), 
        self.token_manager_builder.with_gql_client(gql_client).build(init)?);

        Ok(client)
    }
}

impl ModuleBuilder for ClientBuilder {
    fn build(self: Box<Self>, init: bool) -> Result<Box<dyn crate::Module + Send>, crate::Error> {
        Ok(Box::new(self.do_build(init)?))
    }
}