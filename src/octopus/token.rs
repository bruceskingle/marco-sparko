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
}

impl From<&OctopusToken> for StoredToken {
    fn from(from: &OctopusToken) -> StoredToken {
        StoredToken {
            token_expires: from.token_expires,
            token: from.token.as_ref().clone()
        }
    }
}

struct OctopusToken {
    token_expires:      u32,
    token:              Arc<String>,
}

impl From<StoredToken> for OctopusToken {
    fn from(from: StoredToken) -> OctopusToken {
        OctopusToken {
            token_expires: from.token_expires,
            token: Arc::new(from.token)
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
        }
    }
}

pub struct OctopusAuthenticator {
    api_key:            Option<String>,
    email:              Option<String>,
    password:           Option<String>,
}

impl OctopusAuthenticator {
    fn from_api_key(api_key: String) -> OctopusAuthenticator {
        OctopusAuthenticator {
            api_key: Some(api_key),
            email: None,
            password: None
        }
    }

    fn from_password(email: String, password: String) -> OctopusAuthenticator {
        OctopusAuthenticator {
            api_key: None,
            email: Some(email),
            password: Some(password)
        }
    }

    fn to_obtain_json_web_token_input(&self) ->  Result<ObtainJsonWebTokenInput, sparko_graphql::Error>{

        
        if let Some(api_key) = &self.api_key {
            ObtainJsonWebTokenInput::builder()
            .with_apikey(api_key.clone())
            .build()
        }
        else {
            if let Some(email) = &self.email {
                if let Some(password) = &self.password {
                    ObtainJsonWebTokenInput::builder()
                        .with_email(email.clone())
                        .with_password(password.clone())
                        .build()
                }
                else {
                    panic!("Unreachable");
                }
            }
            else {
                panic!("Unreachable");
            }
        }
    }
}

pub struct OctopusTokenManager {
    context: Arc<MarcoSparkoContext>,
    request_manager: Arc<RequestManager>,
    authenticator: OctopusAuthenticator,
    token: Mutex<Option<OctopusToken>>,
}

impl OctopusTokenManager {
    pub fn builder() -> TokenManagerBuilder {
        TokenManagerBuilder::new()
    }

    fn new(context: Arc<MarcoSparkoContext>,
        request_manager: Arc<RequestManager>,
        authenticator: OctopusAuthenticator,
    ) -> OctopusTokenManager {
        let token: Option<OctopusToken> = if let Some(json_web_token) =  context.read_cache::<StoredToken>(crate::octopus::MODULE_ID) {
            Some(OctopusToken::from(json_web_token))
        }
        else {
            None
        };

        OctopusTokenManager {
            context,
            request_manager,
            authenticator,
            token: Mutex::new(token),
        }
    }
}

impl TokenManager for OctopusTokenManager {

    async fn get_authenticator(&self, refresh: bool)  -> Result<Arc<String>, sparko_graphql::Error> {
        let mut locked_token = self.token.lock().await;

        let current_token = if refresh {
            None
        }
        else {
            if let Some(token) = &*locked_token {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32;

                if token.token_expires - GRACE_PERIOD > now {
                    Some(token.token.clone())
                }
                else {

                    None
                }
            } else {
                None
            }
        };

        if let Some(token) = current_token {
            Ok(token)
        }
        else {
            let input = self.authenticator.to_obtain_json_web_token_input()?;
            let mutation = super::graphql::login::obtain_kraken_token::Mutation::new(input);
            let response = self.request_manager.call(&mutation, None).await?;
    
            let token = OctopusToken::from(response.obtain_kraken_token_);
    
            if let Err(error) = self.context.update_cache(crate::octopus::MODULE_ID, &StoredToken::from(&token)) {
                return Err(sparko_graphql::Error::InternalError(format!("Failed to update cache {}", error)))
            }
    
            let result = token.token.clone();
    
            *locked_token = Some(token);
            
            Ok(result)
        }
    }

}

pub struct TokenManagerBuilder {
    context:            Option<Arc<MarcoSparkoContext>>,
    authenticator:      Option<OctopusAuthenticator>,
    request_manager: Option<Arc<RequestManager>>,
}

impl TokenManagerBuilder{
    fn new() -> TokenManagerBuilder {
        TokenManagerBuilder {
            context: None,
            authenticator: None,
            request_manager: None
        }
    }
    
    pub fn with_context(mut self, context: Arc<MarcoSparkoContext>) -> TokenManagerBuilder {
        self.context = Some(context);
        self
    }

    pub fn with_request_manager(mut self, request_manager: Arc<RequestManager>) -> TokenManagerBuilder {
            self.request_manager = Some(request_manager);
            self
        }

    pub fn with_api_key(mut self, api_key: String) -> TokenManagerBuilder {
        self.authenticator = Some(OctopusAuthenticator::from_api_key(api_key));
        self
    }

    pub fn with_password(mut self, email: String, password: String) -> TokenManagerBuilder {
        self.authenticator = Some(OctopusAuthenticator::from_password(email, password));
        self
    }

    pub fn build(mut self, init: bool) -> anyhow::Result<OctopusTokenManager> {

        if let None = self.authenticator {
            if init {
                println!("Octopus API Authentication (set OCTOPUS_API_KEY to avoid this)");
                print!("email: ");

                std::io::stdout().flush()?;

                let mut email = String::new();
                
                std::io::stdin().read_line(&mut email)?;

                let password = rpassword::prompt_password("password: ").expect("Failed to read password");
                // let mut password = String::new();
                // std::io::stdin().read_line(&mut password)?;

                self = self.with_password(email.trim_end().to_string(), password);
            }
            else {
                return Err(anyhow!("No Octopus authentication credentials given, did you mean to specify --init?"))
            }
        }

        Ok(OctopusTokenManager::new(
            self.context.ok_or(anyhow!("Context must be provided"))?, 
            self.request_manager.ok_or(anyhow!("RequestManager must be provided"))?, 
            self.authenticator.ok_or(anyhow!("Credentials must be specified"))?
        ))
    }
}

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