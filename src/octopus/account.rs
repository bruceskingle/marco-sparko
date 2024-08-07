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

use std::{collections::HashMap, rc::Rc};

use serde::{Deserialize, Serialize};

use super::{error::Error, token::TokenManager};


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountVar<'a> {
    pub account_number: &'a str,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountInterface {
    brand:              Option<String>,
    overdue_balance:    Option<i32>,
    billing_name:       Option<String>,
    billing_sub_name:   Option<String>,
    billing_email:   Option<String>,
}


// pub struct AccountManager  {
//     client: Client
// }

impl AccountInterface {
    // pub fn new(client: Client) -> AccountManager {
    //     AccountManager {
    //         client
    //     }
    // }

    pub async fn get_account( gql_client: &Rc<crate::gql::client::Client>,
        token_manager:  &mut TokenManager)  -> Result<AccountInterface, Error> {

        let operation_name = "getAccount";
        let query = format!(r#"query {}($accountNumber: String!)
                            {{
                                account(accountNumber: $accountNumber)
                                {{
                                    brand,
                                    overdueBalance,
                                    billingName,
                                    billingSubName,
                                    billingEmail,
                                }}
                            }}"#, operation_name);

        println!("QUERY {}", query);

        let mut headers = HashMap::new();
        // let token = String::from(self.get_authenticator().await?);
        let token = &*token_manager.get_authenticator().await?;
        headers.insert("Authorization", token);

        let href = Some(&headers);

        let variables = GetAccountVar {
            account_number: "A-B3D8B29D",
        };

        let mut response = 
        gql_client.call(
            operation_name, 
            &query, 
            &variables, 
            href
        )
        .await?;

        if let Some(result_json) = response.remove("account") {
            let account: AccountInterface = serde_json::from_value(result_json)?;

            Ok(account)


        } else {
            return Err(Error::InternalError("No result found"))
        }
    }
}