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

use std::{collections::HashMap, sync::Arc};

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};
use sparko_graphql_derive::{GraphQLQueryParams};
use time::OffsetDateTime;

use sparko_graphql::{types::Int, GraphQLQuery, ParamBuffer, VariableBuffer};
use sparko_graphql::GraphQLQueryParams;

use crate::octopus::bill::{AccountBillsQuery, AccountBillsQueryParams, AccountBillsViewParams, BillQueryParams};

use super::{bill::AccountBillsView, error::Error, token::TokenManager};




pub struct AccountManager {
    pub account_number: String
}

impl AccountManager {
    pub fn new(account_number: &str) -> AccountManager {
        AccountManager {
            account_number: account_number.to_string()
        }
    }

    pub async fn get_latest_bill(
        &self,
        gql_client: &Arc<sparko_graphql::Client>,
        token_manager: &mut TokenManager,
    ) -> Result<AccountBillsView, Error> {
    let variables = AccountBillsViewParams {
        account_number: self.account_number.clone(),
        bills: BillQueryParams {
            first: Some(Int::new(1)),
            transactions: crate::octopus::bill::StatementTransactionParams { 
                first: Some(Int::new(100)),
                ..Default::default()
            },
            ..Default::default()
        },
    };

    let operation_name = "getAccountLatestBill";
    // let query = AccountBillsView::get_query(&operation_name, &variables);
    
    
    
    // // format!(
    // //     r#"query {}($accountNumber: String!)
    // //                     {{
    // //                         account(accountNumber: $accountNumber)
    // //                         {{
    // //                             {}
    // //                         }}
    // //                     }}"#,
    // //     operation_name, AccountBillsView::get_field_names()
    // // );

    // println!("QUERY {}", query);

    let mut headers = HashMap::new();
    // let token = String::from(self.get_authenticator().await?);
    let token = &*token_manager.get_authenticator().await?;
    headers.insert("Authorization", token);

    let href = Some(&headers);


    println!("NEW params {:?}", &variables);
    println!("NEW params.get_actual {:?}", &variables.get_actual("TEST"));

    let mut result = gql_client
        .new_call::<AccountBillsView, AccountBillsViewParams>(operation_name, "account", variables, href)
        .await?;

        println!("\nHashMap response\n===========================\n{:?}\n===========================\n", result);


        // let result: AccountBillsView = serde_json::from_value(result_json)?;

        Ok(result)
    }
}


#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountParams {
   account_number: String,
}

// #[derive(GraphQLType)]
// #[graphql(params = "AccountBillsViewParams")]
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct AccountBillsView {
//     pub id: String,
//     pub bills: ForwardPageOf<Bill>
// }





// Represents AccountUserType in the GraphQL schema
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountUser {
    pub id: String,
  
    // List of accounts that the user is linked to either via portfolio role or account role.
    pub accounts: Vec<AccountInterface>,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
    pub mobile: String,
    pub landline: String,
    pub title: Option<String>,
  
    // The user's pronouns e.g. 'she/her', 'he/him', 'they/them'.
    pub pronouns: Option<String>,
  
    // Designates whether this user is deceased.
    pub is_deceased: bool,
  
    // The user's secret key to access the Developer API.
    pub live_secret_key: Option<String>,
  
    // // List of portfolios that the user is linked to via their portfolio roles.
    // portfolios(
    //   // Optionally filter the user's portfolios to only return those linked to specified brands.
    //   allowedBrandCodes: [BrandChoices]
  
    //   // Optionally restrict the user portfolios to only return those linked to public facing brands.
    //   restrictToPublicFacingBrands: Boolean
    //   before: String
    //   after: String
    //   first: Int
    //   last: Int
    // ): PortfolioConnectionTypeConnection
  
    // AccountUser's date of birth.
    #[serde(with = "time::serde::iso8601::option")]
    pub date_of_birth: Option<OffsetDateTime>,
  
    // // List of details linked to this user.
    // details: [AccountUserDetailType]
  
    // The user's full name.
    pub full_name: Option<String>,
  
    // The user's preferred name.
    pub preferred_name: Option<String>,
  
      
    // // List of portfolio ids that the user is linked to via their portfolio roles.
    // portfolioIds(
    //   // Optionally filter the user's portfolios to only return those linked to specified brands.
    //   allowedBrandCodes: [BrandChoices]
  
    //   // Optionally restrict the user portfolios to only return those linked to public facing brands.
    //   restrictToPublicFacingBrands: Boolean
    // ): [ID]
    // specialCircumstances: SpecialCircumstancesType
    // preferences: AccountUserCommsPreferences
  
    // List of alternative phone numbers for the account user.
    pub alternative_phone_numbers: Vec<String>,
  
    // Whether there are family issues.
    pub has_family_issues: Option<bool>,
  
    // True if user is linked to an account with an active hardship agreement.
    pub is_in_hardship: Option<bool>,
  
    // // List of roles a user has for each account they're linked to.
    // accountUserRoles(
    //   // Optionally filter the user's account roles to only return those linked to specific accounts.
    //   accountNumber: String
    // ): [AccountUserRoleType]
  
    // // List of roles a user has for each portfolio they're linked to.
    // portfolioUserRoles(
    //   // Optionally filter the portfolio's user roles to only return those linked to a specific portfolio.
    //   portfolioNumber: String
  
    //   // Return the user portfolio roles for this account's portfolio.
    //   accountNumber: String
    // ): [PortfolioUserRoleType]
  
    // List of hold music options.
    // holdMusicChoices: [TrackOptionType]
    pub is_opted_in_to_wof: Option<bool>,
  }

  impl AccountUser {
    pub fn get_field_names(account_field_names: &str) -> String {
        format!(r#"id
accounts {{
{}
    }}
givenName
familyName
email
mobile
landline
title
pronouns
isDeceased
liveSecretKey
dateOfBirth
fullName
preferredName
alternativePhoneNumbers
hasFamilyIssues
isInHardship
isOptedInToWof"#, account_field_names)
    }

    pub async fn get_account_user(
        gql_client: &Arc<sparko_graphql::Client>,
        token_manager: &mut TokenManager,
    ) -> Result<AccountUser, Error> {
        let operation_name = "getAccountUser";
        let query = format!(
            r#"query {}
                            {{
                                viewer
                                {{
                                    {}
                                }}
                            }}"#,
            operation_name, Self::get_field_names(AccountInterface::get_field_names())
        );

        println!("QUERY {}", query);

        let mut headers = HashMap::new();
        // let token = String::from(self.get_authenticator().await?);
        let token = &*token_manager.get_authenticator().await?;
        headers.insert("Authorization", token);

        let href = Some(&headers);

        let variables =  {};

        let mut response = gql_client
            .call(operation_name, &query, &variables, href)
            .await?;

        if let Some(result_json) = response.remove("viewer") {
            let account_user: AccountUser = serde_json::from_value(result_json)?;

            Ok(account_user)
        } else {
            return Err(Error::InternalError("No result found"));
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountVar<'a> {
    pub account_number: &'a str,
}

// enum AccountStatus {
//     // A pending account is one that has been created but no registrations have started.
//     PENDING,
  
//     // Account requires processes to be completed before supply can be set up
//     INCOMPLETE,
  
//     // Withdrawn before supply started
//     WITHDRAWN,
  
//     // Supply could have started, be ongoing or ended.
//     ACTIVE,
  
//     // An error occurred when we tried to enroll a meter point. This may be deprecated in future in favour of exposing this through enrollment property of a meter point.
//     ENROLMENT_ERROR,
  
//     // Meter point enrollment was rejected. This may be deprecated in future in favour of exposing this through enrollment property of a meter point.
//     ENROLMENT_REJECTED,
  
//     // Dormant. Users should not be able to log into dormant accounts.
//     DORMANT,
  
//     // Void. Account created in error.
//     VOID
//   }

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct Address {
//     brand: Option<String>,
//     balance: Option<i32>,
//     overdue_balance: Option<i32>,
//     billing_name: Option<String>,
//     billing_sub_name: Option<String>,
//     billing_email: Option<String>,
// }

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct AccountSummaryView {
//     brand: Option<String>,
//     balance: i32,
//     overdue_balance: Option<i32>,
//     billing_name: Option<String>,
//     billing_sub_name: Option<String>,
//     billing_email: Option<String>,
//     billing_address: Option<String>,
//     billing_address_postcode: Option<String>,
//     account_type: Option<AccountTypeChoices>,
//     preferred_language_for_comms: Option<String>,
//     status: AccountStatus,
// }

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountList {
    pub accounts: Vec<AccountInterface>
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountInterface {
    pub number: Option<String>,
    pub brand: Option<String>,
    pub overdue_balance: Option<i32>,
    pub billing_name: Option<String>,
    pub billing_sub_name: Option<String>,
    pub billing_email: Option<String>,
}

// pub struct AccountManager  {
//     client: Client
// }

impl AccountInterface {
    pub fn get_field_names() -> &'static str {
        r#"
number,
brand,
overdueBalance,
billingName,
billingSubName,
billingEmail
"#
    }
    // pub fn new(client: Client) -> AccountManager {
    //     AccountManager {
    //         client
    //     }
    // }

    pub async fn get_account(
        gql_client: &Arc<sparko_graphql::Client>,
        token_manager: &mut TokenManager,
        account_number: String
    ) -> Result<AccountInterface, Error> {
        let operation_name = "getAccount";
        let query = format!(
            r#"query {}($accountNumber: String!)
                            {{
                                account(accountNumber: $accountNumber)
                                {{
                                    {}
                                }}
                            }}"#,
            operation_name, Self::get_field_names()
        );

        println!("QUERY {}", query);

        let mut headers = HashMap::new();
        // let token = String::from(self.get_authenticator().await?);
        let token = &*token_manager.get_authenticator().await?;
        headers.insert("Authorization", token);

        let href = Some(&headers);

        let variables = AccountParams {
            account_number,
        };

        let mut response = gql_client
            .call(operation_name, &query, &variables.get_actual(""), href)
            .await?;

        if let Some(result_json) = response.remove("account") {
            let account: AccountInterface = serde_json::from_value(result_json)?;

            Ok(account)
        } else {
            return Err(Error::InternalError("No result found"));
        }
    }

    pub async fn get_default_account(
        gql_client: &Arc<sparko_graphql::Client>,
        token_manager: &mut TokenManager
    ) -> Result<AccountInterface, Error> {
        let operation_name = "getDefaultAccount";
        let query = format!(
            r#"query {}
                            {{
                                viewer
                                {{
                                    accounts {{
                                        {}
                                    }}
                                }}
                            }}"#,
            operation_name, Self::get_field_names()
        );


        println!("QUERY {}", query);

        let mut headers = HashMap::new();
        // let token = String::from(self.get_authenticator().await?);
        let token = &*token_manager.get_authenticator().await?;
        headers.insert("Authorization", token);

        let href = Some(&headers);

        let variables = {};

        let mut response = gql_client
            .call(operation_name, &query, &variables, href)
            .await?;

        if let Some(result_json) = response.remove("viewer") {
            let mut account_list: AccountList = serde_json::from_value(result_json)?;

            Ok(account_list.accounts.remove(0))
        } else {
            return Err(Error::InternalError("No result found"));
        }
    }
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct Request<'a, T>
    where T: Serialize
{
    query:          &'a str,
    variables:      T,
    operation_name:  &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_bills_query() {

    let params = AccountBillsQueryParams {
        account: AccountBillsViewParams {
            account_number: "A-B1D2C34D".to_string(),
            bills: BillQueryParams {
                first: Some(Int::new(1)),
                ..Default::default()
            },
        }
    };
   

    let operation_name = "getAccountLatestBill";
    let query = AccountBillsQuery::get_query(&operation_name, &params);
    let variables = params.get_variables().unwrap();

    let payload = Request {
        query: &query,
        variables: &variables,
        operation_name,
    };

    let serialized = serde_json::to_string(&payload).unwrap();

    println!("payload {}", &serialized);




    

    let expected_variables = r#"{
  "account_bills_last": null,
  "account_bills_includeOpenStatements": false,
  "account_bills_includeBillsWithoutPDF": false,
  "account_bills_includeHistoricStatements": true,
  "account_bills_issuedFromDate": null,
  "account_bills_before": null,
  "account_bills_transactions_before": null,
  "account_bills_transactions_first": null,
  "account_bills_issuedToDate": null,
  "account_bills_onlyCurrentEmail": false,
  "account_bills_toDate": null,
  "account_bills_offset": null,
  "account_bills_transactions_last": null,
  "account_bills_includeHeldStatements": false,
  "account_bills_fromDate": null,
  "account_bills_after": null,
  "account_bills_first": 1,
  "account_bills_transactions_after": null,
  "account_accountNumber": "A-B1D2C34D"
}"#;

/*
assertion `left == right` failed
  left: "\n            query getAccountLatestBill($account_accountNumber: String!, $account_bills_includeBillsWithoutPDF: Boolean, $account_bills_includeOpenStatements: Boolean, $account_bills_includeHeldStatements: Boolean, $account_bills_includeHistoricStatements: Boolean, $account_bills_onlyCurrentEmail: Boolean, $account_bills_fromDate: Date, $account_bills_toDate: Date, $account_bills_issuedFromDate: Date, $account_bills_issuedToDate: Date, $account_bills_offset: Int, $account_bills_before: String, $account_bills_after: String, $account_bills_first: Int, $account_bills_last: Int, $account_bills_transactions_before: String, $account_bills_transactions_after: String, $account_bills_transactions_first: Int, $account_bills_transactions_last: Int) {\n                getAccountLatestBill: account(accountNumber: $account_accountNumber) { #get_query_part\n  id\nstatus\nnumber\nbalance\nbills(includeBillsWithoutPDF: $account_bills_includeBillsWithoutPDF, includeOpenStatements: $account_bills_includeOpenStatements, includeHeldStatements: $account_bills_includeHeldStatements, includeHistoricStatements: $account_bills_includeHistoricStatements, onlyCurrentEmail: $account_bills_onlyCurrentEmail, fromDate: $account_bills_fromDate, toDate: $account_bills_toDate, issuedFromDate: $account_bills_issuedFromDate, issuedToDate: $account_bills_issuedToDate, offset: $account_bills_offset, before: $account_bills_before, after: $account_bills_after, first: $account_bills_first, last: $account_bills_last)\n  # pageOf \"bills\"\n  { # pageOf\n    pageInfo {\n        startCursor\n        hasNextPage\n    }\n    edges {\n  # node \"bills\"\n        node { #get_query_part\n  \n      #bill\n              \n          id\nfromDate\ntoDate\nissuedDate\n\n          ...on StatementType {\n            closingBalance\n            openingBalance\n            isExternalBill\n            transactions(before: $account_bills_transactions_before, after: $account_bills_transactions_after, first: $account_bills_transactions_first, last: $account_bills_transactions_last) {\n                pageInfo {\n                    startCursor\n                    hasNextPage\n                }\n                edges {\n                    node { #get_query_part\n  \n      # transaction\n              \n      # charge\n        id\npostedDate\ncreatedAt\naccountNumber\namounts\n  # object \"amounts\"\n    { #get_query_part\n  net\ntax\ngross\n\n} #/get_query_part\n\n  # /object \"amounts\"\nbalanceCarriedForward\nisHeld\nisIssued\ntitle\nbillingDocumentIdentifier\nisReversed\nhasStatement\nnote\n\n        ...on Charge {\n          consumption\n            { #get_query_part\n  startDate\nendDate\nquantity\nunit\nusageCost\nsupplyCharge\n\n} #/get_query_part\n\n          isExport\n        }\n        # /charge\n      \n      # /transaction\n      \n} #/get_query_part\n\n                }\n            }\n            userId\n            toAddress\n            paymentDueDate\n            consumptionStartDate\n            consumptionEndDate\n            reversalsAfterClose\n            status\n            heldStatus\n                { #get_query_part\n  isHeld\nreason\n\n} #/get_query_part\n\n            totalCharges\n                { #get_query_part\n  netTotal\ntaxTotal\ngrossTotal\n\n} #/get_query_part\n\n            totalCredits\n                { #get_query_part\n  netTotal\ntaxTotal\ngrossTotal\n\n} #/get_query_part\n\n          }\n        \n        #/bill\n      \n} #/get_query_part\n\n  # /node \"bills\"\n    } # /edges\n  } # /pageOf\n  # /pageOf \"bills\"\n\n} #/get_query_part\n\n            }\n        "
 right: "\n            query getAccountLatestBill($account_accountNumber: String!, $account_bills_includeBillsWithoutPDF: Boolean, $account_bills_includeOpenStatements: Boolean, $account_bills_includeHeldStatements: Boolean, $account_bills_includeHistoricStatements: Boolean, $account_bills_onlyCurrentEmail: Boolean, $account_bills_fromDate: Date, $account_bills_toDate: Date, $account_bills_issuedFromDate: Date, $account_bills_issuedToDate: Date, $account_bills_offset: Int, $account_bills_before: String, $account_bills_after: String, $account_bills_first: Int, $account_bills_last: Int, $account_bills_transactions_before: String, $account_bills_transactions_after: String, $account_bills_transactions_first: Int, $account_bills_transactions_last: Int) {\n                getAccountLatestBill: account(accountNumber: $account_accountNumber) { #get_query_part\n  id\nstatus\nnumber\nbalance\nbills(includeBillsWithoutPDF: $account_bills_includeBillsWithoutPDF, includeOpenStatements: $account_bills_includeOpenStatements, includeHeldStatements: $account_bills_includeHeldStatements, includeHistoricStatements: $account_bills_includeHistoricStatements, onlyCurrentEmail: $account_bills_onlyCurrentEmail, fromDate: $account_bills_fromDate, toDate: $account_bills_toDate, issuedFromDate: $account_bills_issuedFromDate, issuedToDate: $account_bills_issuedToDate, offset: $account_bills_offset, before: $account_bills_before, after: $account_bills_after, first: $account_bills_first, last: $account_bills_last)\n  # pageOf \"bills\"\n  { # pageOf\n    pageInfo {\n        startCursor\n        hasNextPage\n    }\n    edges {\n  # node \"bills\"\n        node { #get_query_part\n  \n      #bill\n              \n          id\nfromDate\ntoDate\nissuedDate\n\n          ...on StatementType {\n            closingBalance\n            openingBalance\n            isExternalBill\n            transactions(before: $account_bills_transactions_before, after: $account_bills_transactions_after, first: $account_bills_transactions_first, last: $account_bills_transactions_last) {\n                pageInfo {\n                    startCursor\n                    hasNextPage\n                }\n                edges {\n                    node { #get_query_part\n  \n      # transaction\n              \n      # charge\n        id\npostedDate\ncreatedAt\naccountNumber\namounts\n  # object \"amounts\"\n    { #get_query_part\n  net\ntax\ngross\n\n} #/get_query_part\n\n  # /object \"amounts\"\nbalanceCarriedForward\nisHeld\nisIssued\ntitle\nbillingDocumentIdentifier\nisReversed\nhasStatement\nnote\n\n        ...on Charge {\n          consumption\n            { #get_query_part\n  startDate\nendDate\nquantity\nunit\nusageCost\nsupplyCharge\n\n} #/get_query_part\n\n          isExport\n        }\n        # /charge\n      \n      # /transaction\n      \n} #/get_query_part\n\n                }\n            }\n            userId\n            toAddress\n            paymentDueDate\n            consumptionStartDate\n            consumptionEndDate\n            reversalsAfterClose\n            status\n            heldStatus\n                { #get_query_part\n  isHeld\nreason\n\n} #/get_query_part\n\n            totalCharges\n                { #get_query_part\n  netTotal\ntaxTotal\ngrossTotal\n\n} #/get_query_part\n\n            totalCredits\n                { #get_query_part\n  netTotal\ntaxTotal\ngrossTotal\n\n} #/get_query_part\n\n          }\n        \n        #/bill\n      \n} #/get_query_part\n\n  # /node \"bills\"\n    } # /edges\n  } # /pageOf\n  # /pageOf \"bills\"\n\n} #/get_query_part\n\n            }\n        "
stack
*/
    let expected_query = r#"
            query getAccountLatestBill($account_accountNumber: String!, $account_bills_includeBillsWithoutPDF: Boolean, $account_bills_includeOpenStatements: Boolean, $account_bills_includeHeldStatements: Boolean, $account_bills_includeHistoricStatements: Boolean, $account_bills_onlyCurrentEmail: Boolean, $account_bills_fromDate: Date, $account_bills_toDate: Date, $account_bills_issuedFromDate: Date, $account_bills_issuedToDate: Date, $account_bills_offset: Int, $account_bills_before: String, $account_bills_after: String, $account_bills_first: Int, $account_bills_last: Int, $account_bills_transactions_before: String, $account_bills_transactions_after: String, $account_bills_transactions_first: Int, $account_bills_transactions_last: Int) {
                getAccountLatestBill: account(accountNumber: $account_accountNumber) { #get_query_part
  id
status
number
balance
bills(includeBillsWithoutPDF: $account_bills_includeBillsWithoutPDF, includeOpenStatements: $account_bills_includeOpenStatements, includeHeldStatements: $account_bills_includeHeldStatements, includeHistoricStatements: $account_bills_includeHistoricStatements, onlyCurrentEmail: $account_bills_onlyCurrentEmail, fromDate: $account_bills_fromDate, toDate: $account_bills_toDate, issuedFromDate: $account_bills_issuedFromDate, issuedToDate: $account_bills_issuedToDate, offset: $account_bills_offset, before: $account_bills_before, after: $account_bills_after, first: $account_bills_first, last: $account_bills_last)
  # pageOf "bills"
  { # pageOf
    pageInfo {
        startCursor
        hasNextPage
    }
    edges {
  # node "bills"
        node { #get_query_part
  
      #bill
              
          id
fromDate
toDate
issuedDate

          ...on StatementType {
            closingBalance
            openingBalance
            isExternalBill
            transactions(before: $account_bills_transactions_before, after: $account_bills_transactions_after, first: $account_bills_transactions_first, last: $account_bills_transactions_last) {
                pageInfo {
                    startCursor
                    hasNextPage
                }
                edges {
                    node { #get_query_part
  
      # transaction
              
      # charge
        id
postedDate
createdAt
accountNumber
amounts
  # object "amounts"
    { #get_query_part
  net
tax
gross

} #/get_query_part

  # /object "amounts"
balanceCarriedForward
isHeld
isIssued
title
billingDocumentIdentifier
isReversed
hasStatement
note

        ...on Charge {
          consumption
            { #get_query_part
  startDate
endDate
quantity
unit
usageCost
supplyCharge

} #/get_query_part

          isExport
        }
        # /charge
      
      # /transaction
      
} #/get_query_part

                }
            }
            userId
            toAddress
            paymentDueDate
            consumptionStartDate
            consumptionEndDate
            reversalsAfterClose
            status
            heldStatus
                { #get_query_part
  isHeld
reason

} #/get_query_part

            totalCharges
                { #get_query_part
  netTotal
taxTotal
grossTotal

} #/get_query_part

            totalCredits
                { #get_query_part
  netTotal
taxTotal
grossTotal

} #/get_query_part

          }
        
        #/bill
      
} #/get_query_part

  # /node "bills"
    } # /edges
  } # /pageOf
  # /pageOf "bills"

} #/get_query_part

            }
        "#;

    println!("<QUERY>{}</QUERY>\n", &query);
    println!("<VARIABLES>{}</VARIABLES>\n", &variables);

    // assert_eq!(variables, expected_variables);

    assert_eq!(query, expected_query);
    }

    #[test]
    fn test_parse_account_bills_query() {

        let json = r#"{
            "account": {
                "id": "3403670",
                "status": "ACTIVE",
                "number": "A-B1C2D34E",
                "balance": 52020,
                "bills": {
                  "pageInfo": {
                    "startCursor": "YXJyYXljb25uZWN0aW9uOjA=",
                    "hasNextPage": true
                  },
                  "edges": [
                    {
                      "node": {
                        "id": "236646425",
                        "billType": "STATEMENT",
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
                                "id": "-1871040199",
                                "postedDate": "2024-08-20",
                                "createdAt": "2024-08-21T21:36:10.492186+00:00",
                                "accountNumber": "A-B1C2D34E",
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
                                "__typename": "Charge",
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
                                "id": "-1871043601",
                                "postedDate": "2024-08-20",
                                "createdAt": "2024-08-21T21:32:19.902722+00:00",
                                "accountNumber": "A-B1C2D34E",
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
                                "__typename": "Charge",
                                "consumption": {
                                  "startDate": "2024-08-13",
                                  "endDate": "2024-08-20",
                                  "quantity": "181.0500",
                                  "unit": "kWh",
                                  "usageCost": 0,
                                  "supplyCharge": 0
                                },
                                "isExport": true
                              }
                            },
                            {
                              "node": {
                                "id": "-1871044025",
                                "postedDate": "2024-08-20",
                                "createdAt": "2024-08-21T21:32:01.991119+00:00",
                                "accountNumber": "A-B1C2D34E",
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
                                "__typename": "Charge",
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
                                "id": "-1896251302",
                                "postedDate": "2024-08-14",
                                "createdAt": "2024-08-15T11:55:19.400763+00:00",
                                "accountNumber": "A-B1C2D34E",
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
                                "note": "",
                                "__typename": "Credit"
                              }
                            },
                            {
                              "node": {
                                "id": "-1871043620",
                                "postedDate": "2024-08-12",
                                "createdAt": "2024-08-21T21:32:19.073366+00:00",
                                "accountNumber": "A-B1C2D34E",
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
                                "__typename": "Charge",
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
                                "id": "-1871044052",
                                "postedDate": "2024-08-07",
                                "createdAt": "2024-08-21T21:32:01.008991+00:00",
                                "accountNumber": "A-B1C2D34E",
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
                                "__typename": "Charge",
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
                                "id": "-1949392858",
                                "postedDate": "2024-07-29",
                                "createdAt": "2024-08-01T03:09:50.202838+00:00",
                                "accountNumber": "A-B1C2D34E",
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
                                "note": null,
                                "__typename": "Payment"
                              }
                            },
                            {
                              "node": {
                                "id": "-1973989678",
                                "postedDate": "2024-07-24",
                                "createdAt": "2024-07-25T10:53:30.897903+00:00",
                                "accountNumber": "A-B1C2D34E",
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
                                "note": "",
                                "__typename": "Credit"
                              }
                            },
                            {
                              "node": {
                                "id": "-1974036696",
                                "postedDate": "2024-07-24",
                                "createdAt": "2024-07-25T10:43:02.339290+00:00",
                                "accountNumber": "A-B1C2D34E",
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
                                "note": "",
                                "__typename": "Credit"
                              }
                            },
                            {
                              "node": {
                                "id": "-1974103763",
                                "postedDate": "2024-07-24",
                                "createdAt": "2024-07-25T10:17:07.255688+00:00",
                                "accountNumber": "A-B1C2D34E",
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
                                "note": "",
                                "__typename": "Credit"
                              }
                            }
                          ]
                        },
                        "userId": 3235447,
                        "toAddress": "dan@archer.org",
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
          }"#;

        /*
        assertion `left == right` failed
  left: "\n            query getAccountLatestBill($account_accountNumber: String!, $account_bills_includeBillsWithoutPDF: Boolean, $account_bills_includeOpenStatements: Boolean, $account_bills_includeHeldStatements: Boolean, $account_bills_includeHistoricStatements: Boolean, $account_bills_onlyCurrentEmail: Boolean, $account_bills_fromDate: Date, $account_bills_toDate: Date, $account_bills_issuedFromDate: Date, $account_bills_issuedToDate: Date, $account_bills_offset: Int, $account_bills_before: String, $account_bills_after: String, $account_bills_first: Int, $account_bills_last: Int, $account_bills_transactions_before: String, $account_bills_transactions_after: String, $account_bills_transactions_first: Int, $account_bills_transactions_last: Int) {\n                getAccountLatestBill: account(accountNumber: $account_accountNumber) { #get_query_part\n  id\nstatus\nnumber\nbalance\nbills(includeBillsWithoutPDF: $account_bills_includeBillsWithoutPDF, includeOpenStatements: $account_bills_includeOpenStatements, includeHeldStatements: $account_bills_includeHeldStatements, includeHistoricStatements: $account_bills_includeHistoricStatements, onlyCurrentEmail: $account_bills_onlyCurrentEmail, fromDate: $account_bills_fromDate, toDate: $account_bills_toDate, issuedFromDate: $account_bills_issuedFromDate, issuedToDate: $account_bills_issuedToDate, offset: $account_bills_offset, before: $account_bills_before, after: $account_bills_after, first: $account_bills_first, last: $account_bills_last)\n  # pageOf \"bills\"\n  { # pageOf\n    pageInfo {\n        startCursor\n        hasNextPage\n    }\n    edges {\n  # node \"bills\"\n        node { #get_query_part\n  \n      #bill\n              \n          id\nfromDate\ntoDate\nissuedDate\n\n          ...on StatementType {\n            closingBalance\n            openingBalance\n            isExternalBill\n            transactions(before: $account_bills_transactions_before, after: $account_bills_transactions_after, first: $account_bills_transactions_first, last: $account_bills_transactions_last) {\n                pageInfo {\n                    startCursor\n                    hasNextPage\n                }\n                edges {\n                    node { #get_query_part\n  \n      # transaction\n              \n      # charge\n        id\npostedDate\ncreatedAt\naccountNumber\namounts\n  # object \"amounts\"\n    { #get_query_part\n  net\ntax\ngross\n\n} #/get_query_part\n\n  # /object \"amounts\"\nbalanceCarriedForward\nisHeld\nisIssued\ntitle\nbillingDocumentIdentifier\nisReversed\nhasStatement\nnote\n\n        ...on Charge {\n          consumption\n            { #get_query_part\n  startDate\nendDate\nquantity\nunit\nusageCost\nsupplyCharge\n\n} #/get_query_part\n\n          isExport\n        }\n        # /charge\n      \n      # /transaction\n      \n} #/get_query_part\n\n                }\n            }\n            userId\n            toAddress\n            paymentDueDate\n            consumptionStartDate\n            consumptionEndDate\n            reversalsAfterClose\n            status\n            heldStatus\n                { #get_query_part\n  isHeld\nreason\n\n} #/get_query_part\n\n            totalCharges\n                { #get_query_part\n  netTotal\ntaxTotal\ngrossTotal\n\n} #/get_query_part\n\n            totalCredits\n                { #get_query_part\n  netTotal\ntaxTotal\ngrossTotal\n\n} #/get_query_part\n\n          }\n        \n        #/bill\n      \n} #/get_query_part\n\n  # /node \"bills\"\n    } # /edges\n  } # /pageOf\n  # /pageOf \"bills\"\n\n} #/get_query_part\n\n            }\n        "
 right: "\n            query getAccountLatestBill($account_accountNumber: String!, $account_bills_includeBillsWithoutPDF: Boolean, $account_bills_includeOpenStatements: Boolean, $account_bills_includeHeldStatements: Boolean, $account_bills_includeHistoricStatements: Boolean, $account_bills_onlyCurrentEmail: Boolean, $account_bills_fromDate: Date, $account_bills_toDate: Date, $account_bills_issuedFromDate: Date, $account_bills_issuedToDate: Date, $account_bills_offset: Int, $account_bills_before: String, $account_bills_after: String, $account_bills_first: Int, $account_bills_last: Int, $account_bills_transactions_before: String, $account_bills_transactions_after: String, $account_bills_transactions_first: Int, $account_bills_transactions_last: Int) {\n                getAccountLatestBill: account(accountNumber: $account_accountNumber) { #get_query_part\n  id\nstatus\nnumber\nbalance\nbills(includeBillsWithoutPDF: $account_bills_includeBillsWithoutPDF, includeOpenStatements: $account_bills_includeOpenStatements, includeHeldStatements: $account_bills_includeHeldStatements, includeHistoricStatements: $account_bills_includeHistoricStatements, onlyCurrentEmail: $account_bills_onlyCurrentEmail, fromDate: $account_bills_fromDate, toDate: $account_bills_toDate, issuedFromDate: $account_bills_issuedFromDate, issuedToDate: $account_bills_issuedToDate, offset: $account_bills_offset, before: $account_bills_before, after: $account_bills_after, first: $account_bills_first, last: $account_bills_last)\n  # pageOf \"bills\"\n  { # pageOf\n    pageInfo {\n        startCursor\n        hasNextPage\n    }\n    edges {\n  # node \"bills\"\n        node { #get_query_part\n  \n      #bill\n              \n          id\nbillType\nfromDate\ntoDate\nissuedDate\n\n          ...on StatementType {\n            closingBalance\n            openingBalance\n            isExternalBill\n            transactions(before: $account_bills_transactions_before, after: $account_bills_transactions_after, first: $account_bills_transactions_first, last: $account_bills_transactions_last) {\n                pageInfo {\n                    startCursor\n                    hasNextPage\n                }\n                edges {\n                    node { #get_query_part\n  \n      # transaction\n              \n      # charge\n        id\npostedDate\ncreatedAt\naccountNumber\namounts\n  # object \"amounts\"\n    { #get_query_part\n  net\ntax\ngross\n\n} #/get_query_part\n\n  # /object \"amounts\"\nbalanceCarriedForward\nisHeld\nisIssued\ntitle\nbillingDocumentIdentifier\nisReversed\nhasStatement\nnote\n__typename\n\n        ...on Charge {\n          consumption\n            { #get_query_part\n  startDate\nendDate\nquantity\nunit\nusageCost\nsupplyCharge\n\n} #/get_query_part\n\n          isExport\n        }\n        # /charge\n      \n      # /transaction\n      \n} #/get_query_part\n\n                }\n            }\n            userId\n            toAddress\n            paymentDueDate\n            consumptionStartDate\n            consumptionEndDate\n            reversalsAfterClose\n            status\n            heldStatus\n                { #get_query_part\n  isHeld\nreason\n\n} #/get_query_part\n\n            totalCharges\n                { #get_query_part\n  netTotal\ntaxTotal\ngrossTotal\n\n} #/get_query_part\n\n            totalCredits\n                { #get_query_part\n  netTotal\ntaxTotal\ngrossTotal\n\n} #/get_query_part\n\n          }\n        \n        #/bill\n      \n} #/get_query_part\n\n  # /node \"bills\"\n    } # /edges\n  } # /pageOf\n  # /pageOf \"bills\"\n\n} #/get_query_part\n\n            }\n        "
stack backtrace:
         */

        // let bill = Bill::from_json(json).unwrap();
        let result: AccountBillsQuery = serde_json::from_str(json).unwrap();
        
        //assert_eq!(result.account.balance, Int::new(52020)
        assert_eq!(result.account.balance, Int::new(52020));
        
    }
}

/*
assertion `left == right` failed
  left: "{\n  \"account_bills_first\": 1,\n  \"account_bills_fromDate\": null,\n  \"account_bills_includeHeldStatements\": false,\n  \"account_bills_issuedFromDate\": null,\n  \"account_bills_issuedToDate\": null,\n  \"account_bills_after\": null,\n  \"account_bills_transactions_last\": null,\n  \"account_bills_includeOpenStatements\": false,\n  \"account_bills_last\": null,\n  \"account_bills_transactions_after\": null,\n  \"account_bills_transactions_first\": null,\n  \"account_bills_includeHistoricStatements\": true,\n  \"account_bills_before\": null,\n  \"account_bills_transactions_before\": null,\n  \"account_accountNumber\": \"A-B1D2C34D\",\n  \"account_bills_onlyCurrentEmail\": false,\n  \"account_bills_toDate\": null,\n  \"account_bills_offset\": null,\n  \"account_bills_includeBillsWithoutPDF\": false\n}"
 right: "{\n  \"account_bills_last\": null,\n  \"account_bills_includeOpenStatements\": false,\n  \"account_bills_includeBillsWithoutPDF\": false,\n  \"account_bills_includeHistoricStatements\": true,\n  \"account_bills_issuedFromDate\": null,\n  \"account_bills_before\": null,\n  \"account_bills_transactions_before\": null,\n  \"account_bills_transactions_first\": null,\n  \"account_bills_issuedToDate\": null,\n  \"account_bills_onlyCurrentEmail\": false,\n  \"account_bills_toDate\": null,\n  \"account_bills_offset\": null,\n  \"account_bills_transactions_last\": null,\n  \"account_bills_includeHeldStatements\": false,\n  \"account_bills_fromDate\": null,\n  \"account_bills_after\": null,\n  \"account_bills_first\": 1,\n  \"account_bills_transactions_after\": null,\n  \"account_accountNumber\": \"A-B1D2C34D\"\n}"
*/