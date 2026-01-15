use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Write;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::MarcoSparkoContext;

use super::graphql::ObtainJsonWebTokenInput;

use sparko_graphql::{RequestManager, TokenManager};
use super::graphql::login::obtain_kraken_token::ObtainKrakenJsonWebToken;

/*
Implementation of TokenManager for the Octopus API
==================================================
*/

// // Yeah, I know. They declare a GenericScalar in fact its the JWT payload
// #[derive(GraphQLType)]
// #[graphql(params = "NoParams")]
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// struct JWT {
//     sub: String,
//     gty: String,
//     email: String,
//     token_use: String,
//     iss: String,
//     #[graphql(no_params)]
//     #[graphql(scalar)]
//     iat: u32,
//     #[graphql(no_params)]
//     #[graphql(scalar)]
//     exp: u32,
//     #[graphql(no_params)]
//     #[graphql(scalar)]
//     orig_iat: u32
//   }

const GRACE_PERIOD: u32 = 300;
// 60*60*24*10;

#[derive(Serialize, Deserialize)]
struct StoredToken {
    token_expires:      u32,
    token:              String,
    refresh_expires:    u32,
    refresh:            String,
}

impl From<&OctopusToken> for StoredToken {
    fn from(from: &OctopusToken) -> StoredToken {
        StoredToken {
            token_expires: from.token_expires,
            token: from.token.as_ref().clone(),
            refresh_expires: from.refresh_expires,
            refresh: from.refresh.as_ref().clone()
        }
    }
}

struct OctopusToken {
    token_expires:      u32,
    token:              Arc<String>,
    refresh_expires:    u32,
    refresh:            Arc<String>,
}

impl From<StoredToken> for OctopusToken {
    fn from(from: StoredToken) -> OctopusToken {
        OctopusToken {
            token_expires: from.token_expires,
            token: Arc::new(from.token),
            refresh_expires: from.refresh_expires,
            refresh: Arc::new(from.refresh)
        }
    }
}

impl From<ObtainKrakenJsonWebToken> for OctopusToken {
    fn from(from: ObtainKrakenJsonWebToken) -> OctopusToken {
        let mut expires= 0;
        
        if let Some(object) = from.payload_.as_object() {
            if let Some(exp) = object.get("exp") {
                if let Some(exp) = exp.as_u64() {
                    expires = exp as u32;
                }
            }
        }
        OctopusToken {
            token_expires: expires,
            token: Arc::new(from.token_),
            refresh_expires: from.refresh_expires_in_ as u32,
            refresh: Arc::new(from.refresh_token_),
        }
    }
}


pub enum OctopusAuthenticator {
    ApiKey(String),
    EMailPassword { email: String, password: String },
    
}

impl OctopusAuthenticator {
    pub fn from_api_key(api_key: String) -> OctopusAuthenticator {
        OctopusAuthenticator::ApiKey(api_key)
    }

    pub fn from_email_password(email: String, password: String) -> OctopusAuthenticator {
        OctopusAuthenticator::EMailPassword { email, password } 
    }

    pub fn to_obtain_json_web_token_input(&self) ->  Result<ObtainJsonWebTokenInput, sparko_graphql::Error>{
        match self {
            OctopusAuthenticator::ApiKey(api_key) => 
                ObtainJsonWebTokenInput::builder()
                    .with_apikey(api_key.clone())
                    .build(),
            OctopusAuthenticator::EMailPassword { email, password } => 
                ObtainJsonWebTokenInput::builder()
                            .with_email(email.clone())
                            .with_password(password.clone())
                            .build(),
        }
    }
}

pub struct OctopusTokenManager {
    context: Arc<MarcoSparkoContext>,
    request_manager: Arc<RequestManager>,
    authenticator: Mutex<Option<OctopusAuthenticator>>,
    token: Mutex<Option<OctopusToken>>,
}

impl OctopusTokenManager {

    pub fn new(context: Arc<MarcoSparkoContext>,
        request_manager: Arc<RequestManager>,
        authenticator: Option<OctopusAuthenticator>,
    ) -> OctopusTokenManager {
        let token: Option<OctopusToken> = if let Some(json_web_token) =  context.read_cache::<StoredToken>(crate::octopus::MODULE_ID) {
            Some(OctopusToken::from(json_web_token))
        }
        else {
            None
        };

        if let Some(token) = &token {
            println!("Loaded token from cache: {:?}", token.token);
        }
        else {
            println!("No cached token found");
        }

        OctopusTokenManager {
            context,
            request_manager,
            authenticator: Mutex::new(authenticator),
            token: Mutex::new(token),
        }
    }

    pub async fn set_authenticator(&self, authenticator: OctopusAuthenticator) {
        // let mut locked_authenticator = self.authenticator.lock().await;
        *self.authenticator.lock().await = Some(authenticator);
    }
}

impl TokenManager for OctopusTokenManager {

    async fn get_authenticator(&self, refresh: bool)  -> Result<Arc<String>, sparko_graphql::Error> {
        let mut locked_token = self.token.lock().await;

        let mut current_token = None;
        let mut refresh_token = None;

        if !refresh {
            if let Some(token) = &*locked_token {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32;

                if token.token_expires - GRACE_PERIOD > now {
                    current_token = Some(token.token.clone());
                }
                if token.refresh_expires - GRACE_PERIOD > now {
                    refresh_token = Some(token.refresh.clone());
                }
            } 
        }

        // let current_token = if refresh {
        //     None
        // }
        // else {
        //     if let Some(token) = &*locked_token {
        //         let now = SystemTime::now()
        //             .duration_since(UNIX_EPOCH)
        //             .unwrap()
        //             .as_secs() as u32;

        //         if token.token_expires - GRACE_PERIOD > now {
        //             Some(token.token.clone())
        //         }
        //         else {

        //             None
        //         }
        //     } else {
        //         None
        //     }
        // };

        if let Some(token) = current_token {
            Ok(token)
        }
        else {
            if let Some(refresh_token) = refresh_token {
                println!("Refreshing Octopus token using refresh token...");

                let input = ObtainJsonWebTokenInput::builder()
                    .with_refresh_token(refresh_token.as_ref().clone())
                    .build()?;

                let mutation = super::graphql::login::obtain_kraken_token::Mutation::new(input);
                let response: crate::octopus::graphql::login::obtain_kraken_token::Response = self.request_manager.call(&mutation, None).await?;
        
                let token = OctopusToken::from(response.obtain_kraken_token_);

                println!("Obtained new Octopus token via refresh: {:?}", token.token);
        
                if let Err(error) = self.context.update_cache(crate::octopus::MODULE_ID, &StoredToken::from(&token)) {
                    return Err(sparko_graphql::Error::InternalError(format!("Failed to update cache {}", error)))
                }
        
                let result = token.token.clone();
        
                *locked_token = Some(token);
                
                Ok(result)
            }
            else {
                let locked_authenticator = self.authenticator.lock().await;
                if let Some(authenticator) = &*locked_authenticator {
                    
                    let input = authenticator.to_obtain_json_web_token_input()?;

                    println!("Obtaining new Octopus token...{:?}", input);

                    let mutation = super::graphql::login::obtain_kraken_token::Mutation::new(input);
                    let response: crate::octopus::graphql::login::obtain_kraken_token::Response = self.request_manager.call(&mutation, None).await?;
            
                    let token = OctopusToken::from(response.obtain_kraken_token_);
            
                    if let Err(error) = self.context.update_cache(crate::octopus::MODULE_ID, &StoredToken::from(&token)) {
                        return Err(sparko_graphql::Error::InternalError(format!("Failed to update cache {}", error)))
                    }
            
                    let result = token.token.clone();
            
                    *locked_token = Some(token);
                    
                    Ok(result)
                }
                else {
                    Err(sparko_graphql::Error::MissingRequiredValueError("No authenticator provided"))
                }
            }
        }
    }

}

// pub struct ZZTokenManagerBuilder {
//     context:            Option<Arc<MarcoSparkoContext>>,
//     authenticator:      Option<OctopusAuthenticator>,
//     request_manager: Option<Arc<RequestManager>>,
// }

// impl ZZTokenManagerBuilder{
//     fn new() -> TokenManagerBuilder {
//         TokenManagerBuilder {
//             context: None,
//             authenticator: None,
//             request_manager: None
//         }
//     }
    
//     pub fn with_context(mut self, context: Arc<MarcoSparkoContext>) -> TokenManagerBuilder {
//         self.context = Some(context);
//         self
//     }

//     pub fn with_request_manager(mut self, request_manager: Arc<RequestManager>) -> TokenManagerBuilder {
//             self.request_manager = Some(request_manager);
//             self
//         }

//     pub fn with_api_key(mut self, api_key: String) -> TokenManagerBuilder {
//         self.authenticator = Some(OctopusAuthenticator::from_api_key(api_key));
//         self
//     }

//     pub fn with_password(mut self, email: String, password: String) -> TokenManagerBuilder {
//         self.authenticator = Some(OctopusAuthenticator::from_password(email, password));
//         self
//     }

//     fn trim_in_place(s: &mut String) {
//         s.truncate(s.trim_end().len());
//     }

//     pub fn build(mut self, init: bool) -> anyhow::Result<OctopusTokenManager> {

//         let context = self.context.ok_or(anyhow!("Context must be provided"))?;

//         if let None = self.authenticator {
//             if init {
//                 println!("Octopus API Authentication (set OCTOPUS_API_KEY to avoid this)");
//                 print!("email: ");

//                 std::io::stdout().flush()?;

//                 let mut email = String::new();
                
//                 std::io::stdin().read_line(&mut email)?;
//                 Self::trim_in_place(&mut email);

//                 let password = 
//                 if context.args.debug {
//                     print!("password (visible): ");
//                     std::io::stdout().flush()?;
//                     let mut password = String::new();
//                     std::io::stdin().read_line(&mut password)?;
//                     Self::trim_in_place(&mut password);
//                     password
//                 }
//                 else {
//                     rpassword::prompt_password("password: ").expect("Failed to read password")
//                 };
//                 // let mut password = String::new();
//                 // std::io::stdin().read_line(&mut password)?;

//                 // self = self.with_password(email.trim_end().to_string(), password);
//                 self.authenticator = Some(OctopusAuthenticator::from_password(email, password));
//             }
//             else {
//                 return Err(anyhow!("No Octopus authentication credentials given, did you mean to specify --init?"))
//             }
//         }

//         Ok(OctopusTokenManager::new(
//             context,
//             self.request_manager.ok_or(anyhow!("RequestManager must be provided"))?, 
//             self.authenticator.ok_or(anyhow!("Credentials must be specified"))?
//         ))
//     }
// }

// These tests actually hit the API and cause "Too many requests." errors
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_invalid_api_key() {
//         if let Err(octopus_error) = test_api_key("foo".to_string()) {

//             println!("result {:?}", octopus_error);
//             if let Error::GraphQLError(gql_error) = octopus_error {

//                 if let sparko_graphql::error::Error::GraphQLError(json_errors) = &gql_error {
//                     let x = json_errors.get(0).unwrap().extensions.error_code.as_ref().unwrap();
//                     assert_eq!(x, "KT-CT-1139");
//                 }
//                 else {
//                     panic!("Expected GraphQLError KT-CT-1139 got <{:?}>", &gql_error);
//                 }
//             }
//             else {
//                 panic!("Expected GraphQLError KT-CT-1139 got <{:?}>", octopus_error);
//             }
//             // match error {
//             //     Error::GraphQLError(_) => todo!(),
//             //     Error::IOError(_) => todo!(),
//             //     Error::JsonError(_) => todo!(),
//             //     Error::InternalError(_) => todo!(),
//             //     Error::CallerError(_) => todo!(),
//             //     Error::StringError(_) => todo!(),
//             // };
//             // assert_eq!(error., 4);
//         }
//         else {
//             panic!("Expected error, got token");
//         }
        
//     }

//     fn test_api_key(api_key: String)  -> anyhow::Result<Arc<std::string::String>> {
//         let mut octopus_client = crate::octopus::ClientBuilder::new_test()
//             .with_api_key(api_key)?
//             .do_build(false)?;
        
//         tokio_test::block_on(octopus_client.token_manager.authenticate())
//     }

//     #[test]
//     fn test_invalid_refresh() {
//         if let Err(octopus_error) = test_refresh_token("invalid_refresh_token".to_string()) {

//             println!("result {:?}", octopus_error);
//             if let Error::GraphQLError(gql_error) = octopus_error {

//                 if let sparko_graphql::error::Error::GraphQLError(json_errors) = &gql_error {
//                     let x = json_errors.get(0).unwrap().extensions.error_code.as_ref().unwrap();
//                     assert_eq!(x, "KT-CT-1135");
//                 }
//                 else {
//                     panic!("Expected GraphQLError KT-CT-1139 got <{:?}>", &gql_error);
//                 }
//             }
//             else {
//                 panic!("Expected GraphQLError KT-CT-1139 got <{:?}>", octopus_error);
//             }
//             // match error {
//             //     Error::GraphQLError(_) => todo!(),
//             //     Error::IOError(_) => todo!(),
//             //     Error::JsonError(_) => todo!(),
//             //     Error::InternalError(_) => todo!(),
//             //     Error::CallerError(_) => todo!(),
//             //     Error::StringError(_) => todo!(),
//             // };
//             // assert_eq!(error., 4);
//         }
//         else {
//             panic!("Expected error, got token");
//         }
        
//     }

//     #[test]
//     fn test_expired_refresh() {

//         /*

//     Need to fix this once this test refresh token has expired in a week.
//         {
//   "data": {
//     "obtainKrakenToken": {
//       "token": "<JWT>",
//       "possibleErrors": [
//         {
//           "code": "KT-CT-1135",
//           "type": "VALIDATION",
//           "message": "Invalid data.",
//           "description": "Please make sure the refresh token is correct."
//         },
//         {
//           "code": "KT-CT-1134",
//           "type": "VALIDATION",
//           "message": "Invalid data.",
//           "description": "The refresh token has expired."
//         }
//       ]
//     }
//   }
// }
//          */
//         if let Err(octopus_error) = test_refresh_token("1645431dbd92cd2f804fdbba89eaa52e07efe3cabfce41e590ef70a42c96f5ca".to_string()) {

//             println!("result {:?}", octopus_error);
//             if let Error::GraphQLError(gql_error) = octopus_error {

//                 if let sparko_graphql::error::Error::GraphQLError(json_errors) = &gql_error {
//                     let x = json_errors.get(0).unwrap().extensions.error_code.as_ref().unwrap();
//                      assert_eq!(x, "KT-CT-1135");
//                 }
//                 else {
//                     panic!("Expected GraphQLError KT-CT-1139 got <{:?}>", &gql_error);
//                 }
//             }
//             else {
//                 panic!("Expected GraphQLError KT-CT-1139 got <{:?}>", octopus_error);
//             }
//             // match error {
//             //     Error::GraphQLError(_) => todo!(),
//             //     Error::IOError(_) => todo!(),
//             //     Error::JsonError(_) => todo!(),
//             //     Error::InternalError(_) => todo!(),
//             //     Error::CallerError(_) => todo!(),
//             //     Error::StringError(_) => todo!(),
//             // };
//             // assert_eq!(error., 4);
//         }
//         else {
//             panic!("Expected error, got token");
//         }
        
//     }

//     fn test_refresh_token(refresh_token: String)  -> anyhow::Result<Arc<std::string::String>> {
//         let mut octopus_client = crate::octopus::ClientBuilder::new_test()
//             .with_api_key("dummy_api_key_to_prevent_password_prompt".to_string())?
//             .do_build(false)?;

//     octopus_client.token_manager.token = Some(Token {
//         token_expires: 0,
//         token:          Arc::new("dummy_token".to_string()),
//         refresh_token:  ObtainJSONWebTokenInput {
//             api_key: None,
//             email: None,
//             organization_secret_key: None,
//             password: None,
//             pre_signed_key: None,
//             refresh_token: Some(refresh_token),
//         }
//     });
        
//         tokio_test::block_on(octopus_client.token_manager.authenticate())
//     }
// }