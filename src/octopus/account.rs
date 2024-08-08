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

use std::{collections::HashMap, fmt::Display, sync::Arc};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{error::Error, token::TokenManager};


// Represents AccountUserType in the GraphQL schema
#[derive(Serialize, Deserialize, Debug)]
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
    pub fn get_field_names(accout_field_names: &str) -> String {
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
isOptedInToWof"#, accout_field_names)
    }

    pub async fn get_account_user(
        gql_client: &Arc<crate::gql::Client>,
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

impl Display for AccountUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            f.write_str(&json)?;
            Ok(())
        }
        else {
            Err(std::fmt::Error)
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

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(rename_all = "camelCase")]
// pub struct Address {
//     brand: Option<String>,
//     balance: Option<i32>,
//     overdue_balance: Option<i32>,
//     billing_name: Option<String>,
//     billing_sub_name: Option<String>,
//     billing_email: Option<String>,
// }

// #[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountInterface {
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
        gql_client: &Arc<crate::gql::Client>,
        token_manager: &mut TokenManager,
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

        let variables = GetAccountVar {
            account_number: "A-B3D8B29D",
        };

        let mut response = gql_client
            .call(operation_name, &query, &variables, href)
            .await?;

        if let Some(result_json) = response.remove("account") {
            let account: AccountInterface = serde_json::from_value(result_json)?;

            Ok(account)
        } else {
            return Err(Error::InternalError("No result found"));
        }
    }
}
