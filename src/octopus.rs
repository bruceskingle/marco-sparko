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

use std::rc::Rc;
use std::io::{self, Write};

use account::AccountInterface;
use error::Error;
use token::{TokenManager, TokenManagerBuilder};


pub mod error;
pub mod token;
mod account;


// #[derive(Debug)]
pub struct Client{
    gql_client:     Rc<crate::gql::client::Client>,
    token_manager:  TokenManager,
}


impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    fn new(gql_client: Rc<crate::gql::client::Client>, token_manager: TokenManager) -> Client {        
        Client {
            gql_client,
            token_manager
        }
    }

    pub async fn get_account(&mut self)  -> Result<AccountInterface, Error> {
        AccountInterface::get_account(&self.gql_client, &mut self.token_manager).await
    }

    //     let operation_name = "getAccount";
    //     let query = format!(r#"query {}($accountNumber: String!)
    //                         {{
    //                             account(accountNumber: $accountNumber)
    //                             {{
    //                                 brand,
    //                                 overdueBalance,
    //                                 billingName,
    //                                 billingSubName,
    //                                 billingEmail,
    //                             }}
    //                         }}"#, operation_name);

    //     println!("QUERY {}", query);

    //     let mut headers = HashMap::new();
    //     // let token = String::from(self.get_authenticator().await?);
    //     let token = &*self.token_manager.get_authenticator().await?;
    //     headers.insert("Authorization", token);

    //     let href = Some(&headers);

    //     let variables = account::GetAccountVar {
    //         accountNumber: "A-B3D8B29D",
    //     };

    //     let mut response = 
    //     self.gql_client.call(
    //         operation_name, 
    //         &query, 
    //         &variables, 
    //         href
    //     )
    //     .await?;

    //     if let Some(result_json) = response.remove("account") {
    //         let account: AccountInterface = serde_json::from_value(result_json)?;

    //         Ok(account)


    //     } else {
    //         return Err(Error::InternalError("No result found"))
    //     }

    //     // let parse = true;
            
    //     //     if parse {
    //     //         println!("\n\nHERE1\n\n");
    //     //         let result: HashMap<String, HashMap<String, AccountInterface>> = response.json().await?;

    //     //         println!("\n\nHERE2\n\n");
    //     //         println!("result {:?}", result);

    //     //     } else {
    //     //         let string_value = &response.text().await?;
            
    //     //         println!("\nstring_value:   {:?}", string_value);

    //     //         // let obj = serde_json::from_str(string_value).unwrap();
    //     //         let obj: HashMap<String, HashMap<String,Value>> = serde_json::from_str(string_value).unwrap();
    //     //         println!("\n JSON {}", serde_json::to_string_pretty(&obj).unwrap());
    //     //     }
    //     //     println!("\n\nHERE3\n\n");
    //     //     Ok(())

    //     // let input = r#""
    //     // {\"query\":\"mutation obtainKrakenToken($input: ObtainJSONWebTokenInput!) {\\n  obtainKrakenToken(input: $input) {\\n    refreshToken\\n    refreshExpiresIn\\n    token\\n  }\\n}\",\"variables\":{\"input\":{\"APIKey\":\"SECRET\"}},\"operationName\":\"obtainKrakenToken\"}
    //     // "#;
    // }
}


pub struct ClientBuilder {
    gql_client_builder:         crate::gql::client::ClientBuilder,
    token_manager_builder:      TokenManagerBuilder,
}

impl ClientBuilder {

    pub fn new() -> ClientBuilder {
        ClientBuilder {
            gql_client_builder:     crate::gql::client::Client::builder(),
            token_manager_builder:  TokenManager::builder()
        }
    }
    
    
    pub fn authenticate(self) -> Result<ClientBuilder, Error> {
        if let Ok(api_key) = std::env::var("OCTOPUS_API_KEY") {
            self.with_api_key(api_key)
        }
        else {
            println!("Octopus API Authentication (set OCTOPUS_API_KEY to avoid this)");
            print!("email: ");

            std::io::stdout().flush()?;

            let mut email = String::new();
            
            std::io::stdin().read_line(&mut email)?;

            let password = rpassword::prompt_password("password: ").expect("Failed to read password");

            self.with_password(email.trim_end().to_string(), password)

        }
    }

    pub fn with_url(mut self, url: String) -> Result<ClientBuilder, Error> {
        self.gql_client_builder.with_url(url);
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

    pub fn build(self) -> Result<Client, Error> {
        let gql_client = Rc::new(self.gql_client_builder.build());

        Ok(Client::new(gql_client.clone(), 
            self.token_manager_builder.with_gql_client(gql_client).build()?))
    }
}