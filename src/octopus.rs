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

use std::{collections::BTreeMap, sync::Arc};
use async_trait::async_trait;
use bill::BillResults;
use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

use account::{AccountInterface, AccountManager, AccountUser};
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

    fn handle_bill(result: BillResults) -> Result<(), crate::Error> {
        //println!("\n===========================\n{}\n===========================\n", result);

        match &result.bills.edges[0].node {
            bill::Bill::Statement(statement) => {
                statement.print();

                // println!("Energy Account Statement");
                // println!("========================");
                // println!("Date                {}", statement.issued_date);
                // println!("Ref                 {}", statement.id);
                // println!("From                {}", statement.from_date);
                // println!("To                  {}", statement.to_date);

                // let mut map = BTreeMap::new();
                // for edge in &statement.transactions.edges {
                //     let txn = edge.node.as_transaction();

                //     map.insert(&txn.posted_date, &edge.node);
                // }


                // for transaction in &mut map.values() {
                //     let txn = transaction.as_transaction();

                //     print!("{:20} {:10} ", 
                //                 txn.title,
                //                 txn.posted_date
                //             );
                //     print!("{:10} {:10} {:10} {:10}", 
                //         txn.amounts.net,
                //         txn.amounts.tax, 
                //         txn.amounts.gross,
                //         txn.balance_carried_forward
                //         );

                //     if let transaction::Transaction::Charge(charge) = &transaction {
                            
                //         print!("{}-{} {:10} ", 
                //             charge.consumption.start_date,
                //             charge.consumption.end_date,
                //             charge.consumption.quantity
                //         );
                //         if *charge.is_export {
                //             print!("export ");
                //         }
                //         else {
                //             print!("import ");
                //         }
                //     }
                //     println!();
                // } 
            },
        };



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

            let account_manager = AccountManager::new(account_number);

            let result = account_manager.get_latest_bill(&self.gql_client, &mut self.token_manager).await?;

            Self::handle_bill(result)
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_bill_total() {
        let json = r#"
{
  "status": "ACTIVE",
  "number": "A-B3D8B29D",
  "balance": 39305,
  "bills": {
    "pageInfo": {
      "startCursor": "YXJyYXljb25uZWN0aW9uOjA=",
      "hasNextPage": true
    },
    "edges": [
      {
        "node": {
          "billType": "STATEMENT",
          "id": "236646425",
          "fromDate": "2024-07-22",
          "toDate": "2024-08-21",
          "issuedDate": "2024-08-22",
          "closingBalance": 39303,
          "openingBalance": 17791,
          "isExternalBill": false,
          "transactions": {
            "pageInfo": {
              "startCursor": "YXJyYXljb25uZWN0aW9uOjA=",
              "hasNextPage": false
            },
            "edges": [
              {
                "node": {
                  "__typename": "Charge",
                  "id": "-1871040199",
                  "postedDate": "2024-08-20",
                  "createdAt": "2024-08-21T21:36:10.492186Z",
                  "accountNumber": "A-B3D8B29D",
                  "amounts": {
                    "net": 2711,
                    "tax": 136,
                    "gross": 2847
                  },
                  "balanceCarriedForward": 39303,
                  "isHeld": false,
                  "isIssued": true,
                  "title": "Gas",
                  "billingDocumentIdentifier": "236646425",
                  "isReversed": false,
                  "hasStatement": true,
                  "note": "",
                  "consumption": {
                    "startDate": "2024-07-21",
                    "endDate": "2024-08-20",
                    "quantity": "360.7100",
                    "unit": "kWh",
                    "usageCost": 0,
                    "supplyCharge": 0
                  },
                  "isExport": false
                }
              },
              {
                "node": {
                  "__typename": "Charge",
                  "id": "-1871043601",
                  "postedDate": "2024-08-20",
                  "createdAt": "2024-08-21T21:32:19.902722Z",
                  "accountNumber": "A-B3D8B29D",
                  "amounts": {
                    "net": -2716,
                    "tax": 0,
                    "gross": -2716
                  },
                  "balanceCarriedForward": 42150,
                  "isHeld": false,
                  "isIssued": true,
                  "title": "Electricity",
                  "billingDocumentIdentifier": "236646425",
                  "isReversed": false,
                  "hasStatement": true,
                  "note": "",
                  "consumption": {
                    "startDate": "2024-08-13",
                    "endDate": "2024-08-20",
                    "quantity": "181.500",
                    "unit": "kWh",
                    "usageCost": 0,
                    "supplyCharge": 0
                  },
                  "isExport": true
                }
              },
              {
                "node": {
                  "__typename": "Charge",
                  "id": "-1871044025",
                  "postedDate": "2024-08-20",
                  "createdAt": "2024-08-21T21:32:01.991119Z",
                  "accountNumber": "A-B3D8B29D",
                  "amounts": {
                    "net": 2854,
                    "tax": 143,
                    "gross": 2997
                  },
                  "balanceCarriedForward": 39434,
                  "isHeld": false,
                  "isIssued": true,
                  "title": "Electricity",
                  "billingDocumentIdentifier": "236646425",
                  "isReversed": false,
                  "hasStatement": true,
                  "note": "",
                  "consumption": {
                    "startDate": "2024-08-08",
                    "endDate": "2024-08-20",
                    "quantity": "334.7100",
                    "unit": "kWh",
                    "usageCost": 0,
                    "supplyCharge": 0
                  },
                  "isExport": false
                }
              },
              {
                "node": {
                  "__typename": "Credit",
                  "id": "-1896251302",
                  "postedDate": "2024-08-14",
                  "createdAt": "2024-08-15T11:55:19.400763Z",
                  "accountNumber": "A-B3D8B29D",
                  "amounts": {
                    "net": 478,
                    "tax": 24,
                    "gross": 502
                  },
                  "balanceCarriedForward": 42431,
                  "isHeld": false,
                  "isIssued": true,
                  "title": "Powerups Reward",
                  "billingDocumentIdentifier": "236646425",
                  "isReversed": false,
                  "hasStatement": true,
                  "note": ""
                }
              },
              {
                "node": {
                  "__typename": "Charge",
                  "id": "-1871043620",
                  "postedDate": "2024-08-12",
                  "createdAt": "2024-08-21T21:32:19.073366Z",
                  "accountNumber": "A-B3D8B29D",
                  "amounts": {
                    "net": -2407,
                    "tax": 0,
                    "gross": -2407
                  },
                  "balanceCarriedForward": 41929,
                  "isHeld": false,
                  "isIssued": true,
                  "title": "Electricity",
                  "billingDocumentIdentifier": "236646425",
                  "isReversed": false,
                  "hasStatement": true,
                  "note": "",
                  "consumption": {
                    "startDate": "2024-07-21",
                    "endDate": "2024-08-12",
                    "quantity": "300.8200",
                    "unit": "kWh",
                    "usageCost": 0,
                    "supplyCharge": 0
                  },
                  "isExport": true
                }
              },
              {
                "node": {
                  "__typename": "Charge",
                  "id": "-1871044052",
                  "postedDate": "2024-08-07",
                  "createdAt": "2024-08-21T21:32:01.008991Z",
                  "accountNumber": "A-B3D8B29D",
                  "amounts": {
                    "net": 4104,
                    "tax": 205,
                    "gross": 4309
                  },
                  "balanceCarriedForward": 39522,
                  "isHeld": false,
                  "isIssued": true,
                  "title": "Electricity",
                  "billingDocumentIdentifier": "236646425",
                  "isReversed": false,
                  "hasStatement": true,
                  "note": "",
                  "consumption": {
                    "startDate": "2024-07-21",
                    "endDate": "2024-08-07",
                    "quantity": "322.5100",
                    "unit": "kWh",
                    "usageCost": 0,
                    "supplyCharge": 0
                  },
                  "isExport": false
                }
              },
              {
                "node": {
                  "__typename": "Payment",
                  "id": "-1949392858",
                  "postedDate": "2024-07-29",
                  "createdAt": "2024-08-01T03:09:50.202838Z",
                  "accountNumber": "A-B3D8B29D",
                  "amounts": {
                    "net": 24790,
                    "tax": 0,
                    "gross": 0
                  },
                  "balanceCarriedForward": 43831,
                  "isHeld": false,
                  "isIssued": true,
                  "title": "Direct debit",
                  "billingDocumentIdentifier": "236646425",
                  "isReversed": false,
                  "hasStatement": true,
                  "note": null
                }
              },
              {
                "node": {
                  "__typename": "Credit",
                  "id": "-1973989678",
                  "postedDate": "2024-07-24",
                  "createdAt": "2024-07-25T10:53:30.897903Z",
                  "accountNumber": "A-B3D8B29D",
                  "amounts": {
                    "net": 543,
                    "tax": 28,
                    "gross": 571
                  },
                  "balanceCarriedForward": 19041,
                  "isHeld": false,
                  "isIssued": true,
                  "title": "Powerups Reward",
                  "billingDocumentIdentifier": "236646425",
                  "isReversed": false,
                  "hasStatement": true,
                  "note": ""
                }
              },
              {
                "node": {
                  "__typename": "Credit",
                  "id": "-1974036696",
                  "postedDate": "2024-07-24",
                  "createdAt": "2024-07-25T10:43:02.33929Z",
                  "accountNumber": "A-B3D8B29D",
                  "amounts": {
                    "net": 177,
                    "tax": 9,
                    "gross": 186
                  },
                  "balanceCarriedForward": 18470,
                  "isHeld": false,
                  "isIssued": true,
                  "title": "Powerups Reward",
                  "billingDocumentIdentifier": "236646425",
                  "isReversed": false,
                  "hasStatement": true,
                  "note": ""
                }
              },
              {
                "node": {
                  "__typename": "Credit",
                  "id": "-1974103763",
                  "postedDate": "2024-07-24",
                  "createdAt": "2024-07-25T10:17:07.255688Z",
                  "accountNumber": "A-B3D8B29D",
                  "amounts": {
                    "net": 469,
                    "tax": 24,
                    "gross": 493
                  },
                  "balanceCarriedForward": 18284,
                  "isHeld": false,
                  "isIssued": true,
                  "title": "Powerups Reward",
                  "billingDocumentIdentifier": "236646425",
                  "isReversed": false,
                  "hasStatement": true,
                  "note": ""
                }
              }
            ]
          },
          "userId": 3235447,
          "toAddress": "bruce@skingle.org",
          "paymentDueDate": "2024-09-06",
          "consumptionStartDate": null,
          "consumptionEndDate": null,
          "reversalsAfterClose": "NONE",
          "status": "CLOSED",
          "heldStatus": {
            "isHeld": false,
            "reason": null
          },
          "totalCharges": {
            "netTotal": 4546,
            "taxTotal": 484,
            "grossTotal": 5030
          },
          "totalCredits": {
            "netTotal": 1667,
            "taxTotal": 85,
            "grossTotal": 1752
          }
        }
      }
    ]
  }
}
        "#;


        let result: bill::BillResults = serde_json::from_str(json).unwrap();
        
        Client::handle_bill(result);

        panic!("DEV");
    }

}
