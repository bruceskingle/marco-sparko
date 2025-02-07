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

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Write;
use serde::{Deserialize, Serialize};

use crate::Context;

use super::error::Error;
use super::graphql::ObtainJsonWebTokenInput;

use sparko_graphql::{RequestManager, TokenManager};
use super::graphql::login::obtain_kraken_token::ObtainKrakenJsonWebToken;

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
    context:            Context,
    request_manager: Arc<RequestManager>,
    authenticator: OctopusAuthenticator,
    token: Option<OctopusToken>
}

impl OctopusTokenManager {
    pub fn builder() -> TokenManagerBuilder {
        TokenManagerBuilder::new()
    }

    fn new(context: Context,
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
            token,
        }
    }

    pub fn clone_delete_me(&self) -> OctopusTokenManager {
        OctopusTokenManager {
            context: self.context.clone(),
            request_manager: self.request_manager.clone(),
            authenticator: OctopusAuthenticator {
                api_key: self.authenticator.api_key.clone(),
                email: self.authenticator.email.clone(),
                password: self.authenticator.password.clone(),
            },
            token: if let Some(token) = &self.token {
                Some(OctopusToken {
                    token_expires: token.token_expires,
                    token: token.token.clone()
                })
            }
            else {
                None
            }
        }
    }
}

impl TokenManager for OctopusTokenManager {

    async fn get_authenticator(&mut self)  -> Result<Arc<String>, Box<dyn std::error::Error>> {
        let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;

        if let Some(token) = &self.token {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;

            if token.token_expires - GRACE_PERIOD > now {
                Ok(token.token.clone())
            }
            else {

                self.authenticate().await
            }
        } else {
            self.authenticate().await
        }
    }

    async fn authenticate(&mut self)  -> Result<Arc<String>, Box<dyn std::error::Error>> {

        let input = self.authenticator.to_obtain_json_web_token_input()?;
        let mutation = super::graphql::login::obtain_kraken_token::Mutation::new(input);
        let response = self.request_manager.call(&mutation, None).await?;
        // println!("Result {}", serde_json::to_string_pretty(&response)?);

        //     token_arc = Arc::new(response.obtain_kraken_token_.token_);
        //     token = Some(&token_arc);

        // let response = self.request_manager.mutation::<ObtainJSONWebTokenInput, ObtainKrakenJSONWebToken>("Login", "obtainKrakenToken", variables).await?;
        let token = OctopusToken::from(response.obtain_kraken_token_);

        self.context.update_cache(crate::octopus::MODULE_ID, &StoredToken::from(&token))?;

        let result = token.token.clone();

        self.token = Some(token);


        Ok(result)
    }

}

pub struct TokenManagerBuilder {
    context:            Option<Context>,
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
    
    pub fn with_context(mut self, context: Context) -> TokenManagerBuilder {
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

    pub fn build(mut self, init: bool) -> Result<OctopusTokenManager, Error> {

        if let None = self.authenticator {
            if init {
                println!("Octopus API Authentication (set OCTOPUS_API_KEY to avoid this)");
                print!("email: ");

                std::io::stdout().flush()?;

                let mut email = String::new();
                
                std::io::stdin().read_line(&mut email)?;

                let password = rpassword::prompt_password("password: ").expect("Failed to read password");

                self = self.with_password(email.trim_end().to_string(), password);
            }
            else {
                return Err(Error::StringError("No Octopus authentication credentials given, did you mean to specify --init?".to_string()))
            }
        }

        Ok(OctopusTokenManager::new(
            self.context.ok_or(Error::CallerError("Context must be provided"))?, 
            self.request_manager.ok_or(Error::CallerError("RequestManager must be provided"))?, 
            self.authenticator.ok_or(Error::CallerError("Credentials must be specified"))?
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

//     fn test_api_key(api_key: String)  -> Result<Arc<std::string::String>, Error> {
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

//     fn test_refresh_token(refresh_token: String)  -> Result<Arc<std::string::String>, Error> {
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