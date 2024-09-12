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

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

use crate::Context;

use super::{error::Error, PossibleErrorType};

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct JWT {
    sub: String,
    gty: String,
    email: String,
    token_use: String,
    iss: String,
    iat: u32,
    exp: u32,
    orig_iat: u32
  }

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct ObtainKrakenJSONWebToken {
    // "The body payload of the Kraken Token. The same information can be obtained by using JWT decoding tools on the value of the token field."
    //payload: GenericScalar,
    errors: Option<Vec<PossibleErrorType>>,
    // "A token that can be used in a subsequent call to obtainKrakenToken to get a new Kraken Token with the same access conditions after the previous one has expired."
    refresh_token: Option<String>,
    // "A Unix timestamp representing the point in time at which the refresh token will expire."
    refresh_expires_in: Option<u32>,
    // "The Kraken Token. Can be used in the Authorization header for subsequent calls to the API to access protected resources."
    token: String,
    payload: JWT
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct ObtainKrakenJSONWebTokenWrapper {
    obtain_kraken_token: Option<ObtainKrakenJSONWebToken>
}

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// struct ObtainKrakenJSONWebTokenResponse {
//     errors: Option<Vec<PossibleErrorType>>,
//     data:   ObtainKrakenJSONWebTokenWrapper,
// }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObtainJSONWebTokenInput {
    // "API key of the account user. Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "APIKey")]
    api_key: Option<String>,
    // "Email address of the account user. Use with 'password' field."
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    // // "Live secret key of an third-party organization. Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    organization_secret_key: Option<String>,
    // // "Password of the account user. Use with 'email' field."
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    // // "Short-lived, temporary key (that's pre-signed). Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    pre_signed_key: Option<String>,
    // // "The refresh token that can be used to extend the expiry claim of a Kraken token. Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Variables<'a> {
   input:  &'a ObtainJSONWebTokenInput,
}

const GRACE_PERIOD: u32 = 300;
// 60*60*24*10;

struct Token {
    token_expires:      u32,
    refresh_expires:    u32,
    token:              Arc<String>,
    refresh_token:      ObtainJSONWebTokenInput,
}

impl Token {
    fn new(token: ObtainKrakenJSONWebToken) -> Token {
        Token {
            refresh_expires: token.refresh_expires_in.unwrap(),
            token_expires: token.payload.exp,
            token:          Arc::new(token.token),
            refresh_token:  ObtainJSONWebTokenInput {
                api_key: None,
                email: None,
                organization_secret_key: None,
                password: None,
                pre_signed_key: None,
                refresh_token: Some(token.refresh_token.unwrap()),
            }
        }
    }
}

pub struct TokenManager {
    context:        Context,
    gql_client:     Arc<sparko_graphql::Client>,
    authenticator:  ObtainJSONWebTokenInput,
    token:          Option<Token>
}

impl TokenManager{
    pub fn builder() -> TokenManagerBuilder {
        TokenManagerBuilder::new()
    }

    fn new(context: Context, gql_client: Arc<sparko_graphql::Client>, authenticator:  ObtainJSONWebTokenInput) -> TokenManager {
        let token = if let Some(json_web_token) =  context.read_cache(crate::octopus::MODULE_ID) {
            Some(Token::new(json_web_token))
        }
        else {
            None
        };

        TokenManager {
            context,
            gql_client,
            authenticator,
            token,
        }
    }

    pub async fn get_authenticator(&mut self)  -> Result<Arc<String>, Error> {
        let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;
        println!("now {}", now);

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

    pub async fn authenticate(&mut self)  -> Result<Arc<String>, Error> {
        let query_name = "obtainKrakenToken";
        let query = format!(r#"mutation {}($input: ObtainJSONWebTokenInput!) {{
                obtainKrakenToken(input: $input) {{
                    refreshToken
                    refreshExpiresIn
                    token
                    payload
    }}
    }}"#, query_name);

        println!("QUERY {}", query);


        let variables = Variables {
            input: if let Some(token) = &self.token { &token.refresh_token } else { &self.authenticator }
        };

        let mut response = self.gql_client.call(query_name, &query, &variables, None).await?;

        if let Some(result_json) = response.remove(query_name) {
            let token: ObtainKrakenJSONWebToken = serde_json::from_value(result_json)?;

                if let Some(errors) = token.errors {
                    return Err(Error::StringError(PossibleErrorType::to_string(errors)))
                }

                self.context.update_cache(crate::octopus::MODULE_ID, &token)?;

                self.token = Some(Token::new(token));
                 
        } else {
            return Err(Error::InternalError("No result found"));
        }

        let r = &self.token.as_ref().unwrap().token;

        Ok(r.clone())
    }

}

pub struct TokenManagerBuilder {
    context:            Option<Context>,
    gql_client:         Option<Arc<sparko_graphql::Client>>,
    authenticator:      Option<ObtainJSONWebTokenInput>,

}

impl TokenManagerBuilder{
    fn new() -> TokenManagerBuilder {
        TokenManagerBuilder {
            context: None,
            gql_client:     None,
            authenticator:  None
        }
    }
    
    pub fn with_context(mut self, context: Context) -> TokenManagerBuilder {
        self.context = Some(context);
        self
    }
    
    pub fn with_gql_client(mut self, gql_client: Arc<sparko_graphql::Client>) -> TokenManagerBuilder {
        self.gql_client = Some(gql_client);
        self
    }

    pub fn with_api_key(mut self, api_key: String) -> TokenManagerBuilder {
        self.authenticator = 
            Some(ObtainJSONWebTokenInput {
                api_key: Some(api_key),
                email: None,
                organization_secret_key: None,
                password: None,
                pre_signed_key: None,
                refresh_token: None,
            });
            // Some(Authenticator::ApiKey { key: api_key });
        self
    }

    pub fn with_password(mut self, email: String, password: String) -> TokenManagerBuilder {
        self.authenticator = 
            Some(ObtainJSONWebTokenInput {
                api_key: None,
                email: Some(email),
                organization_secret_key: None,
                password: Some(password),
                pre_signed_key: None,
                refresh_token: None,
            });
        // Some(Authenticator::EmailPassword { email: email, password: password });
        self
    }

    pub fn build(mut self, init: bool) -> Result<TokenManager, Error> {

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

        Ok(TokenManager::new(
            self.context.ok_or(Error::CallerError("Context must be provided"))?, 
            self.gql_client.ok_or(Error::CallerError("GQL Client must be provided"))?, 
            self.authenticator.ok_or(Error::CallerError("Credentials must be specified"))?)
        )
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

//     Need to fix this once this test refresh token has exired in a week.
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