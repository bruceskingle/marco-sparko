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
use time::OffsetDateTime;

use crate::AuthenticatedRequestManager;
// use sparko_graphql_derive::{GraphQLQueryParams, GraphQLType};
use sparko_graphql::{types::{Boolean, Date, DateTime, ForwardPageOf, Int, ID}, GraphQL, GraphQLQueryParams, GraphQLType, NoParams, ParamBuffer, TokenManager};


use crate::octopus::{bill::{AccountBillsViewParams, BillQueryParams}, consumption_type::ConsumptionTypeQueryParams, meter::MeterQueryParams};
use crate::octopus::transaction::StatementTransactionParams;

use super::{bill::AccountBillsView, decimal::Decimal, error::Error, meter_point::{ElectricityMeterPointType, MeterPointQueryParams}, meter_point_property_view::{ElectricityMeterPointPropertyView, GasMeterPointPropertyView}, token::OctopusTokenManager};


use graphql_client::{reqwest::post_graphql as post_graphql, GraphQLQuery};


#[allow(clippy::upper_case_acronyms)]
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/octopus/octopus-schema.graphql",
    query_path = "graphql/octopus/getAccountPropertiesMeters.graphql",
    response_derives = "Debug",
)]
pub struct GetAccountPropertiesMeters;


// #[allow(clippy::upper_case_acronyms)]
// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "graphql/octopus/octopus-schema.graphql",
//     query_path = "graphql/octopus/MeterConsumption.graphql",
//     response_derives = "Debug",
// )]
// pub struct MeterConsumption;


#[allow(clippy::upper_case_acronyms)]
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/octopus/octopus-schema.graphql",
    query_path = "graphql/octopus/MeterAgreements.graphql",
    response_derives = "Debug",
)]
pub struct MeterAgreements;

#[allow(clippy::upper_case_acronyms)]
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/octopus/octopus-schema.graphql",
    query_path = "graphql/octopus/ElectricityAgreementLineItems.graphql",
    response_derives = "Debug",
)]
pub struct ElectricityAgreementLineItems;

#[allow(clippy::upper_case_acronyms)]
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/octopus/octopus-schema.graphql",
    query_path = "graphql/octopus/GasAgreementLineItems.graphql",
    response_derives = "Debug",
)]
pub struct GasAgreementLineItems;

fn missing(item: &str) -> Result<get_account_properties_meters::GetAccountPropertiesMetersAccount, Box<dyn std::error::Error>> {
    Err(Box::new(Error::StringError(format!("No {} found in result", item))))
}

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
        token_manager: &mut OctopusTokenManager,
    ) -> Result<AccountBillsView, Error> {
    let variables = AccountBillsViewParams {
        account_number: self.account_number.clone(),
        bills: BillQueryParams {
            first: Some(Int::new(1)),
            transactions: StatementTransactionParams { 
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

    let result = gql_client
        .new_call::<AccountBillsView, AccountBillsViewParams>(operation_name, "account", variables, href)
        .await?;

        println!("\nHashMap response\n===========================\n{:?}\n===========================\n", result);


        // let result: AccountBillsView = serde_json::from_value(result_json)?;

        Ok(result)
    }
 
    pub async fn get_electric_line_items (
        &self,
        authenticated_request_manager: &mut AuthenticatedRequestManager<OctopusTokenManager>,
        agreement_id: String,
        from: &Date,
        to: &Option<Date>,
    ) -> Result<get_account_properties_meters::GetAccountPropertiesMetersAccount, Box<dyn std::error::Error>> {

        let variables = electricity_agreement_line_items::Variables {
            agreement_id,
            start_at: from.at_midnight(),
            first: Some(5),
            timezone: String::from("Europe/London"),
            item_type: electricity_agreement_line_items::LineItemTypeOptions::CONSUMPTION_CHARGE,
            line_item_grouping: electricity_agreement_line_items::LineItemGroupingOptions::NONE,
            last_cursor: None,
        };

        let response = authenticated_request_manager.call::<ElectricityAgreementLineItems>(variables).await?;
        
        let x: electricity_agreement_line_items::ElectricityAgreementLineItemsElectricityAgreement = unexpected_none(response.electricity_agreement)?;

        let agreement = if let electricity_agreement_line_items::ElectricityAgreementLineItemsElectricityAgreement::ElectricityAgreementType(agreement) = x {
            agreement
        }
        else {
            return Err(Box::new(Error::StringError(String::from("Unexpected graphql response"))));
        };

        for item in unexpected_none(agreement.line_items)?.line_items.edges.into_iter().flatten() {
            let l: electricity_agreement_line_items::LineItemsEdgesNode = unexpected_none(item.node)?;
            
        }
        unimplemented!();
    }
 
    pub async fn get_meter_agreements (
        &self,
        authenticated_request_manager: &mut AuthenticatedRequestManager<OctopusTokenManager>,
        meter_node_id: String,
        from: &Date,
        to: &Option<Date>,
    ) -> Result<get_account_properties_meters::GetAccountPropertiesMetersAccount, Box<dyn std::error::Error>> {

        let variables = meter_agreements::Variables {
            meter_node_id,
            valid_after: Some(from.at_midnight())
        };

        // let from_time = from.at_midnight();
        // let to_time = if let Some(to) = to { Some(to.at_midnight())} else {None};
        let response = authenticated_request_manager.call::<MeterAgreements>(variables).await?;


        let mut meter_agreements = if let Some(node) = response.node {
            match node {

                meter_agreements::MeterAgreementsNode::ElectricityMeterType(electricity_agreement) => {
                    for agreement in electricity_agreement.meter_point.agreements
                    .into_iter()
                    .flatten() 
                    .flatten() 
                    {
                        println!("Agreement {:?}", &agreement);
                        if in_scope(&from, &to, &agreement.valid_from, &agreement.valid_to) {
                            let agreement_id = unexpected_none(agreement.id)?.to_string();
                            println!("Electricity agreement {}", agreement_id);

                            self.get_electric_line_items(authenticated_request_manager, agreement_id, from, to).await;

                        }
                    } 
                },
                meter_agreements::MeterAgreementsNode::GasMeterType(gas_agreement) => {
                    for agreement in gas_agreement.meter_point.agreements
                    .into_iter()
                    .flatten() 
                    .flatten() 
                    {
                        if in_scope(&from, &to, &agreement.valid_from, &agreement.valid_to) {
                            let agreement_id = unexpected_none(agreement.id)?;
                            println!("Gas agreement {}", agreement_id);
                        }
                    } 
                },
                _ => return Err(Box::new(Error::InternalError("Unexpected node type found in agreements query")))
             }
        }
        else {
            return missing("node");
        };



    unimplemented!();
    }



    pub async fn get_account_properties_meters(
        &self,
        authenticated_request_manager: &mut AuthenticatedRequestManager<OctopusTokenManager>,
        from: &Date,
        to: &Option<Date>,
    ) -> Result<get_account_properties_meters::GetAccountPropertiesMetersAccount, Box<dyn std::error::Error>> {


        let variables = get_account_properties_meters::Variables {
            account_number: self.account_number.clone()
        };

        let response = authenticated_request_manager.call::<GetAccountPropertiesMeters>(variables).await?;


        let mut data = if let Some(data) = response.account {
            data
        }
        else {
            return Err(Box::new(Error::InternalError("No result found")));
        };

        // node_ids of all active meters
        // let mut import_meters = Vec::new();
        // let mut export_meters = Vec::new();
        // let mut gas_meters = Vec::new();

        // let x: Option<Vec<Option<get_account_properties_meters::GetAccountPropertiesMetersAccountProperties>>> = data.properties;


        for property in data.properties.into_iter().flatten().map(|f| f.unwrap()) 
        {
            for electricity_meter_point in property.electricity_meter_points.into_iter().flatten().map(|f| f.unwrap()) {
                for electricity_meter in electricity_meter_point.meters.into_iter().flatten().map(|f| f.unwrap()) {
                    if let Some(_import_meter) = electricity_meter.import_meter {
                        println!("Export electricity meter {}", &electricity_meter.node_id);
                        // export_meters.push(electricity_meter.node_id);

                        self.get_meter_agreements(authenticated_request_manager,
                            electricity_meter.node_id,
                            from,
                            to).await?;
                    }
                    else {
                        println!("Import electricity meter {}", &electricity_meter.node_id);
                        // import_meters.push(electricity_meter.node_id);
                        self.get_meter_agreements(authenticated_request_manager,
                            electricity_meter.node_id,
                            from,
                            to).await?;
                    }
                }
            }

            for gas_meter_point in property.gas_meter_points.into_iter().flatten().map(|f| f.unwrap()) {
                for gas_meter in gas_meter_point.meters.into_iter().flatten().map(|f| f.unwrap()) {
                    println!("Gas meter {}", &gas_meter.node_id);
                    // gas_meters.push(gas_meter.node_id);
                    self.get_meter_agreements(authenticated_request_manager,
                        gas_meter.node_id,
                        from,
                        to).await?;
                }
            }
        }


    unimplemented!();

    // let variables = AccountPropertyQueryParams {
    //     account_number: self.account_number.clone(),
    //     properties: PropertyQueryParams {
    //         active_from,
    //         // electricity_meter_points: MeterPointQueryParams {
    //         //     meters: MeterQueryParams {
    //         //         id: None,
    //         //         include_inactive: Boolean::from(false),
    //         //         consumption: ConsumptionTypeQueryParams {
    //         //             start_at: DateTime::from_calendar_date(year, month, day),
    //         //             grouping: None,
    //         //             timezone: None,
    //         //             before: None,
    //         //             after: None,
    //         //             first: None,
    //         //             last: None,
    //         //         },
    //         //     },
    //         // },
    //         // gas_meter_points: MeterPointQueryParams{
    //         //     meters: MeterQueryParams {
    //         //         id: None,
    //         //         include_inactive: None,
    //         //         consumption: None,
    //         //     },
    //         // },
    //         // first: Some(Int::new(1)),
    //         // transactions: StatementTransactionParams { 
    //         //     first: Some(Int::new(100)),
    //         //     ..Default::default()
    //         // },
    //         // ..Default::default()
    //     },
    // };

    // let operation_name = "getAccountLatestBill";
    // // let query = AccountBillsView::get_query(&operation_name, &variables);
    
    
    
    // // // format!(
    // // //     r#"query {}($accountNumber: String!)
    // // //                     {{
    // // //                         account(accountNumber: $accountNumber)
    // // //                         {{
    // // //                             {}
    // // //                         }}
    // // //                     }}"#,
    // // //     operation_name, AccountBillsView::get_field_names()
    // // // );

    // // println!("QUERY {}", query);

    // let mut headers = HashMap::new();
    // // let token = String::from(self.get_authenticator().await?);
    // let token = &*token_manager.get_authenticator().await?;
    // headers.insert("Authorization", token);

    // let href = Some(&headers);


    // println!("NEW params {:?}", &variables);
    // println!("NEW params.get_actual {:?}", &variables.get_actual("TEST"));

    // let result = gql_client
    //     .new_call::<AccountPropertyView, AccountPropertyQueryParams>(operation_name, "account", variables, href)
    //     .await?;

    //     println!("\nHashMap response\n===========================\n{:?}\n===========================\n", result);


    //     // let result: AccountBillsView = serde_json::from_value(result_json)?;

    //     Ok(result)
    }
}

fn unexpected_none<T>(value: Option<T>) -> Result<T, Box<Error>> {
    value.ok_or(Box::new(Error::InternalError("Unexpected None value found")))
}

fn in_scope(from: &Date, to: &Option<Date>, valid_from: &Option<DateTime>, valid_to: &Option<DateTime>) -> bool {
    let end_in_scope = if let Some(to) = to {
        if let Some(valid_from) = valid_from {
            valid_from.to_date() <= *to
        }
        else {
            false
        }
    }
    else {
        true
    };

    let start_in_scope = if let Some(valid_to) = valid_to {
            valid_to.to_date() >= *from

    }
    else {
        true
    };
    
    println!("in_scope({:?},{:?},{:?},{:?}) = {}",
        from,
        to,
        valid_from,
        valid_to,
        end_in_scope && start_in_scope
    );
    end_in_scope && start_in_scope
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
#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountUser {
    pub id: String,
  
    // List of accounts that the user is linked to either via portfolio role or account role.
    #[graphql(no_params)]
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

    #[graphql(no_params)]
    #[graphql(scalar)]
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
        // gql_client: &Arc<sparko_graphql::Client>,
        // token_manager: &mut OctopusTokenManager,
        authenticatedRequestManager: &mut sparko_graphql::AuthenticatedRequestManager<OctopusTokenManager>
    ) -> Result<AccountUser, Error> {

        let mut response = authenticatedRequestManager.query::<NoParams, AccountUser>("GetAccountUser", "viewer", NoParams).await?;

        Ok(response)
        
        // let operation_name = "getAccountUser";
        // let query = format!(
        //     r#"query {}
        //                     {{
        //                         viewer
        //                         {{
        //                             {}
        //                         }}
        //                     }}"#,
        //     operation_name, Self::get_field_names(AccountInterface::get_field_names())
        // );

        // println!("QUERY {}", query);

        // let mut headers = HashMap::new();
        // // let token = String::from(self.get_authenticator().await?);
        // let token = &*token_manager.get_authenticator().await?;
        // headers.insert("Authorization", token);

        // let href = Some(&headers);

        // let variables =  {};

        // let mut response = gql_client
        //     .call(operation_name, &query, &variables, href)
        //     .await?;

        // if let Some(result_json) = response.remove("viewer") {
        //     let account_user: AccountUser = serde_json::from_value(result_json)?;

        //     Ok(account_user)
        // } else {
        //     return Err(Error::InternalError("No result found"));
        // }
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

// #[derive(GraphQLType)]
// #[graphql(params = "NoParams")]
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct ViewerAccounts {
//     #[graphql(no_params)]
//     pub viewer: AccountList,
// }


#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountList {
    #[graphql(no_params)]
    pub accounts: Vec<AccountInterface>
}

#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountInterface {
    pub number: String,
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
        // gql_client: &Arc<sparko_graphql::Client>,
        // token_manager: &mut OctopusTokenManager,
        authenticatedRequestManager: &mut sparko_graphql::AuthenticatedRequestManager<OctopusTokenManager>,
        account_number: String
    ) -> Result<AccountInterface, Error> {
        let mut response = authenticatedRequestManager.query::<NoParams, AccountList>("GetViewerAccounts", "account", NoParams).await?;

        Ok(response.accounts.remove(0))



        // let operation_name = "getAccount";
        // let query = format!(
        //     r#"query {}($accountNumber: String!)
        //                     {{
        //                         account(accountNumber: $accountNumber)
        //                         {{
        //                             {}
        //                         }}
        //                     }}"#,
        //     operation_name, Self::get_field_names()
        // );

        // println!("QUERY {}", query);

        // let mut headers = HashMap::new();
        // // let token = String::from(self.get_authenticator().await?);
        // let token = &*token_manager.get_authenticator().await?;
        // headers.insert("Authorization", token);

        // let href = Some(&headers);

        // let variables = AccountParams {
        //     account_number,
        // };

        // let mut response = gql_client
        //     .call(operation_name, &query, &variables.get_actual(""), href)
        //     .await?;

        // if let Some(result_json) = response.remove("account") {
        //     let account: AccountInterface = serde_json::from_value(result_json)?;

        //     Ok(account)
        // } else {
        //     return Err(Error::InternalError("No result found"));
        // }
    }

    pub async fn get_default_account(
        // gql_client: &Arc<sparko_graphql::Client>,
        // token_manager: &mut OctopusTokenManager
        authenticatedRequestManager: &mut sparko_graphql::AuthenticatedRequestManager<OctopusTokenManager>
    ) -> Result<AccountInterface, Error> {
        let mut response = authenticatedRequestManager.query::<NoParams, AccountList>("GetViewerAccounts", "viewer", NoParams).await?;

        Ok(response.accounts.remove(0))


        // let operation_name = "getDefaultAccount";
        // let query = format!(
        //     r#"query {}
        //                     {{
        //                         viewer
        //                         {{
        //                             accounts {{
        //                                 {}
        //                             }}
        //                         }}
        //                     }}"#,
        //     operation_name, Self::get_field_names()
        // );


        // println!("QUERY {}", query);

        // let mut headers = HashMap::new();
        // // let token = String::from(self.get_authenticator().await?);
        // let token = &*token_manager.get_authenticator().await?;
        // headers.insert("Authorization", token);

        // let href = Some(&headers);

        // let variables = {};

        // let mut response = gql_client
        //     .call(operation_name, &query, &variables, href)
        //     .await?;

        // if let Some(result_json) = response.remove("viewer") {
        //     let mut account_list: AccountList = serde_json::from_value(result_json)?;

        //     Ok(account_list.accounts.remove(0))
        // } else {
        //     return Err(Error::InternalError("No result found"));
        // }
    }
}


#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountPropertyQueryParams {
    #[graphql(required)]
    pub account_number: String,
    pub properties: PropertyQueryParams,
}

#[derive(GraphQLType)]
#[graphql(params = "AccountPropertyQueryParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountPropertyView {
    pub id: ID,
    pub properties: Vec<PropertyMeterView>,
}

#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct PropertyQueryParams {
    pub active_from: Option<DateTime>,
}

#[derive(GraphQLType)]
#[graphql(params = "PropertyQueryParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct PropertyMeterView {
    pub id: ID,
    pub address: Option<String>,
    pub postcode: Option<String>,
    #[graphql(no_params)]
    pub electricity_meter_points: Vec<ElectricityMeterPointPropertyView>,
    #[graphql(no_params)]
    pub gas_meter_points: Vec<GasMeterPointPropertyView>,
    #[graphql(no_params)]
    pub smart_device_networks: Vec<SmartMeterDeviceNetworkPropertyView>,
}

// #[derive(GraphQLType)]
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct ElectricityMeterPointMeterView {
//     pub id: ID,
//     pub address: Option<String>,
//     pub postcode: Option<String>,
//     #[graphql(no_params)]
//     pub electricity_meter_points: Vec<ElectricityMeterPoint>,
//     #[graphql(no_params)]
//     pub gas_meter_points: Vec<GasMeterPointType>,
//     #[graphql(no_params)]
//     pub smart_device_networks: Vec<SmartDeviceNetwork>,
// }



// #[derive(GraphQLQueryParams)]
// #[serde(rename_all = "camelCase")]
// pub struct SmartMeterDeviceNetworkQueryParams {
//     #[graphql(scalar)]
//     pub statuses: Vec<DeviceStatus>
// }

// #[derive(GraphQLType)]
// #[graphql(params = "SmartMeterDeviceNetworkQueryParams")]
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// // This is the network through which a set of SMETS2 devices communicates.
// pub struct SmartMeterDeviceNetworkType {
//   id: ID,

//   // A list of devices attached to one network.
//   smart_devices: Vec<SmartMeterDeviceType>
// }



#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
// This is the network through which a set of SMETS2 devices communicates.
pub struct SmartMeterDeviceNetworkPropertyView {
  id: ID,

  // A list of devices attached to one network.
  #[graphql(no_params)]
  smart_devices: Vec<SmartMeterDevicePropertyView>
}


#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
// This is the network through which a set of SMETS2 devices communicates.
pub struct SmartMeterDevicePropertyView {
  id: ID,
//   importElectricityMeter: ElectricityMeterType
//   exportElectricityMeter: ElectricityMeterType
//   gasMeter: GasMeterType
//   deviceNetwork: SmartMeterDeviceNetworkType
  serial_number: String,
  device_id: String,
  #[graphql(no_params)]
  #[graphql(scalar)]
  #[serde(rename = "type")]
  device_type: DeviceType,
  #[graphql(no_params)]
  #[graphql(scalar)]
  status: DeviceStatus,
  manufacturer: String,
  model: String,
  firmware_version: String,

  // The payment mode (e.g. credit or prepayment) that the device is currently operating in.
  #[graphql(no_params)]
  #[graphql(scalar)]
  payment_mode: PaymentMode,

  // The rate, in pence per week, that debt is being recovered from this device.
  weekly_debt_recovery_rate_in_pence: Int
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceType {
  // Electricity Smart Meter (ESME)
  ESME,
  // Gas Smart Meter (GSME)
  GSME,
  // Gas Proxy Function (GPF)
  GPF,
  // Communications Hub Function (CHF)
  CHF,
  // HAN Connected Auxiliary Load Control Switch (HCALCS)
  HCALCS,
  // Prepayment Interface Device (PPMID)
  PPMID,
  // In-House Display (IHD)
  IHD,
  // Consumer Access Device (CAD)
  CAD,
  // IHD or CAD (a type 2 device)
  IhdOrCad
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceStatus {
  PENDING,
  WHITELISTED,
  InstalledNotCommissioned,
  COMMISSIONED,
  DECOMMISSIONED,
  WITHDRAWN,
  SUSPENDED,
  RECOVERY,
  NotApplicable
}

//
//     The mode used by a SMETS2 meter to charge for energy consumed.
//
//     Energy consumption can either be paid for in advance (i.e. prepay / pay-as-you-go)
//     or at some time later (i.e. credit).
//
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentMode {
  PREPAY,
  CREDIT
}

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// pub enum DeviceStatuses {
//   PENDING,
//   WHITELISTED,
//   INSTALLED_NOT_COMMISSIONED,
//   COMMISSIONED,
//   DECOMMISSIONED,
//   WITHDRAWN,
//   SUSPENDED,
//   RECOVERY,
//   RECOVERED,
//   NOT_APPLICABLE
// }