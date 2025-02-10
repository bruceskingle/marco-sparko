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



use std::collections::HashMap;

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};


use simple::gql::{ForwardPageOf, GraphQLQueryParams, GraphQLType, GraphQL, ParamBuffer, VariableBuffer};

// use sparko_graphql_derive::{GraphQLQueryParams, GraphQLType};

type ID = String;
type Date = String;
type DateTime = String;
type Int = i32;




/******************     TEST DERIVES     ******************************/
// #[derive(GraphQLElement)]
// pub struct TestPropertySimpleView {
//     pub id: String,
//     pub postcode: String
// }

// #[derive(GraphQLQueryParams)]
// pub struct TestAccountQueryParams {
//     pub active_from: Option<String>,
//     pub account_number: String,
//     pub properties: AccountPropertiesViewQueryParams
// }

// #[derive(GraphQLType, Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// #[graphql(params = "AccountQueryParams")]
// pub struct Bruce {
//     pub account: AccountPropertiesView
// }



/******************     TEST HAND WRITTEN     ******************************/

#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct PropertySimpleViewParams {
    pub active_from: Option<DateTime>
}

// impl GraphQLQueryParams for PropertySimpleViewParams {
//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {
//         params.push_formal(&prefix, "activeFrom", "DateTime");
//     }

//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String){
//         params.push_actual(&prefix, "activeFrom",);
//     }

//     fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {
//         variables.push_variable(&prefix, "activeFrom", &self.active_from)?;
//         Ok(())
//     }
// }

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct PropertySimpleView {
    pub id: String,
    pub postcode: String
}

impl GraphQLType<PropertySimpleViewParams> for PropertySimpleView {
    fn get_query_part(params: &PropertySimpleViewParams, prefix: String) -> String {
        format!(r#"
                id
                postcode
        "#
        )
    }
}

// impl GraphQLElement for PropertySimpleView {
//     fn get_field_names() -> &'static str {
//         r#"
//             id
//             postcode
//         "#
//     }
// }



#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountPropertiesViewParams {
    pub account_number: String,
    pub properties: PropertySimpleViewParams
}

// impl GraphQLQueryParams for AccountPropertiesViewParams {
//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {
//         params.push_formal(&prefix, "accountNumber", "String!");
//         self.properties.get_formal_part(params, GraphQL::prefix(&prefix, "properties"));
//     }

//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String){
//         params.push_actual(&prefix, "accountNumber",);
//     }

//     fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {

//         variables.push_variable(&prefix, "accountNumber", &self.account_number)?;
//         self.properties.get_variables_part(variables, GraphQL::prefix(&prefix, "properties"))
//     }
// }

#[derive(GraphQLType)]
#[graphql(params = "AccountPropertiesViewParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountPropertiesView {
    pub id: String,
    pub properties: Vec<PropertySimpleView>
}


// impl GraphQLType<AccountPropertiesViewParams> for AccountPropertiesView {
//     fn get_query_part(params: &AccountPropertiesViewParams, prefix: String) -> String {
//         {
//             let res = std::fmt::format(
//                 format_args!(
//                     "id\nproperties({0}){{\n    {1}\n}}\n",
//                     params.properties.get_actual(GraphQL::prefix(&prefix, "properties")),
//                     PropertySimpleView::get_query_part(
//                         &params.properties,
//                         GraphQL::prefix(&prefix, "properties"),
//                     ),
//                 ),
//             );
//             res
//         }
//     }
// }


// impl GraphQLType<AccountPropertiesViewParams> for AccountPropertiesView {
//     fn get_query_part(params: &AccountPropertiesViewParams, prefix: String) -> String {
//         let test = "AccountPropertiesViewParams";
//         let foo = "id\nproperties({}){{\n    {}\n}}\n";
//         {
//             format!(
//                     "id\nproperties({0}){{\n    {1}\n}}\n",
//                     params.properties.get_actual(GraphQL::prefix(&prefix, "properties")),
//                     PropertySimpleView::get_query_part(
//                         &params.properties,
//                         GraphQL::prefix(&prefix, "properties"),
//                     ),
//                 )
//         }
//     }
// }

// impl GraphQLType<AccountPropertiesViewParams> for AccountPropertiesView {
//     fn get_query_part(params: &AccountPropertiesViewParams, prefix: String) -> String {
//         format!(r#"
//             id
//             properties{} {{
//                 {}
//             }}
//         "#, params.properties.get_actual(GraphQL::prefix(&prefix, "properties")),
//               PropertySimpleView::get_query_part(&params.properties, GraphQL::prefix(&prefix, "properties"))
//         )
//     }
// }

#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountPropertiesQueryParams {
    account: AccountPropertiesViewParams
}



// impl GraphQLQueryParams for AccountPropertiesQueryParams {
//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {
//         self.account.get_formal_part(params, GraphQL::prefix(&prefix, "account"));
//     }

//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String) {
//     }

//     fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {
        
//         self.account.get_variables_part(variables, GraphQL::prefix(&prefix, "account"))
//     }
// }

#[derive(GraphQLType)]
#[graphql(params = "AccountPropertiesQueryParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountPropertiesQuery {
    pub account: AccountPropertiesView
}

// impl GraphQLType < AccountPropertiesQueryParams > for AccountPropertiesQuery
// {
//     fn
//     get_query_part(params : & AccountPropertiesQueryParams, prefix : String)
//     -> String
//     {
//         format!
//         ("account({}){{\n    {}\n}}\n",
//         params.account.get_actual(GraphQL :: prefix(& prefix, "account")),
//         AccountPropertiesView ::
//         get_query_part(& params.account, GraphQL ::
//         prefix(& prefix, "account")),)
//     }
// }


// impl GraphQLType < AccountPropertiesQueryParams > for AccountPropertiesQuery
// {
//     fn
//     get_query_part(params : & AccountPropertiesQueryParams, prefix : String)
//     -> String
//     {
//         format!
//         ("account({}){{\n    {}\n}}\n",
//         params."account".get_actual(GraphQL :: prefix(& prefix, "account")),
//         PropertySimpleView ::
//         get_query_part(& params."account", GraphQL ::
//         prefix(& prefix, "account")),)
//     }
// }


// impl GraphQLType<AccountPropertiesQueryParams> for AccountPropertiesQuery {
//     fn get_query_part(params: &AccountPropertiesQueryParams, prefix: String) -> String {
//         format!(r#"
//             account{} {{
//                 {}
//             }}
//         "#, params.account.get_actual(GraphQL::prefix(&prefix, "account")),
//             // params.properties.get_actual("properties_"),
//             AccountPropertiesView::get_query_part(&params.account, GraphQL::prefix(&prefix, "account"))
//         )
//     }
    
//     // fn get_request_name() -> &'static str {
//     //     "getAccountPropertiesView"
//     // }
// }


fn test_simple_properties_view() {
    let expected_query = r#"
    query get($accountNumber: String!, $properties_activeFrom: DateTime) {
        account(accountNumber: $accountNumber) {
            
            id
            properties(activeFrom: $properties_activeFrom) {
                
        id
        postcode
    
            }
    
        }
    }
    "#;

    let params = AccountPropertiesQueryParams {
        account: AccountPropertiesViewParams {
            account_number: String::from("A-B3D8B29D"),
            properties: PropertySimpleViewParams {
                active_from: Some(String::from("2024-08-12T23:00:00.000Z")),
            }
        }
    };
    let query = AccountPropertiesQuery::get_query(String::from("getAccountProperties"), &params);

    println!("query = r#\"{}\"#", query);

    // assert_eq!(query, expected_query);

    let expected_variables = r#"{
    "accountNumber": "A-B3D8B29D",
    "properties_activeFrom": "2024-08-12T23:00:00.000Z"
    }"#;

    let variables = params.get_variables().unwrap();

    println!("variables = r#\"{}\"#", variables);

    // assert_eq!(variables, expected_variables);
}









// impl GraphQLQueryParams for TransactionSimpleViewParams {
//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {
//         params.push_formal(&prefix, "first", "Int");
//     }

//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String){
//         params.push_actual(&prefix, "first",);
//     }

//     fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {
//         variables.push_variable(&prefix, "first", &self.first)?;
//         Ok(())
//     }
// }



// impl GraphQLType<TransactionSimpleViewParams> for TransactionSimpleView {
//     fn get_query_part(params: &TransactionSimpleViewParams, prefix: String) -> String {
//         format!(r#"
//                 id
//                 postedDate
//                 __typename
//         "#
//         )
//     }
// }



// impl GraphQLQueryParams for BillParams {
//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {
//         params.push_formal(&prefix, "first", "Int");
//         self.transactions.get_formal_part(params, GraphQL::prefix(&prefix, "transactions"));
//     }

//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String){
//         params.push_actual(&prefix, "first",);
//     }

//     fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {

//         variables.push_variable(&prefix, "first", &self.first)?;
//         self.transactions.get_variables_part(variables, GraphQL::prefix(&prefix, "transactions"))
//     }
// }



// impl GraphQLType<BillParams> for SimpleBillInterfaceType {
//     fn get_query_part(params: &BillParams, prefix: String) -> String {
//         format!(r#"
//                 id
//                 fromDate
//                 toDate
//         "#
//         )
//     }
// }





// impl GraphQLComponent<BillsSimpleViewParams> for BillsSimpleView {
//     fn get_query_part(params: &BillsSimpleViewParams, prefix: &str) -> String {
//         format!(r#"
//                 id
//                 fromDate
//                 toDate
//                 transactions{} {{
//                     {}
//                 }}
//         "#, 
//             params.get_actual(prefix),
//             PropertySimpleView::get_field_names()
//     )
//     }
// }



#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountBillsAndPropertiesViewParams {
    #[graphql(required)]
    pub account_number: String,
    pub properties: PropertySimpleViewParams,
    pub bills: BillParams
}

// impl GraphQLQueryParams for AccountBillsAndPropertiesViewParams {
//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {
//         params.push_formal(&prefix, "accountNumber", "String!");
//         self.properties.get_formal_part(params, GraphQL::prefix(&prefix, "properties"));
//         self.bills.get_formal_part(params, GraphQL::prefix(&prefix, "bills"));
//     }

//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String){
//         params.push_actual(&prefix, "accountNumber",);
//     }

//     fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {

//         variables.push_variable(&prefix, "accountNumber", &self.account_number)?;
//         self.properties.get_variables_part(variables, GraphQL::prefix(&prefix, "properties"))?;
//         self.bills.get_variables_part(variables, GraphQL::prefix(&prefix, "bills"))
//     }
// }

#[derive(GraphQLType)]
#[graphql(params = "AccountBillsAndPropertiesViewParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountBillsAndPropertiesView {
    pub id: String,
    pub properties: Vec<PropertySimpleView>,
    pub bills: ForwardPageOf<SimpleStatementType>
}

/*
format!
        ("
id
 properties({}){{
     {}
 }}
 bills({}){{
     pageInfo {{
         startCursor
         hasNextPage
     }}
     edges {{
         node {{
                 {}
         }}
     }}
 }}
 ",

*/

/* this is whats generated, no pagination......

impl GraphQLType < AccountBillsAndPropertiesViewParams > for
AccountBillsAndPropertiesView
{
    fn
    get_query_part(params : & AccountBillsAndPropertiesViewParams, prefix :
    String) -> String
    {
        format!
        ("id
 properties({}){{
     {}
 }}
 bills({}){{
     {}
 }}
 ",
        params.properties.get_actual(GraphQL ::
        prefix(& prefix, "properties")), PropertySimpleView ::
        get_query_part(& params.properties, GraphQL ::
        prefix(& prefix, "properties")),
        params.bills.get_actual(GraphQL :: prefix(& prefix, "bills")),
        SimpleStatementType ::
        get_query_part(& params.bills, GraphQL :: prefix(& prefix, "bills")),)
    }
}

*/



// works:
// impl GraphQLType<AccountBillsAndPropertiesViewParams> for AccountBillsAndPropertiesView {
//     fn get_query_part(params: &AccountBillsAndPropertiesViewParams, prefix: String) -> String {
//         format!(r#"
            
//                 id
//                 properties{} {{
//                     {}
//                 }}
//                 bills{} {{
//                 pageInfo
//                 {{
//                     startCursor
//                     hasNextPage
//                 }}
//                 edges
//                 {{
//                     node
//                     {{
//                         {}
//                     }}
//                 }}
//             }}
//         "#, params.properties.get_actual(GraphQL::prefix(&prefix, "properties")),
//             PropertySimpleView::get_query_part(&params.properties, GraphQL::prefix(&prefix, "properties")),
//             params.bills.get_actual(GraphQL::prefix(&prefix, "bills")),
//             SimpleStatementType::get_query_part(&params.bills, GraphQL::prefix(&prefix, "bills"))
//         )
//     }
// } 







// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct AccountBillsAndPropertiesQueryParams {
//     account: AccountBillsAndPropertiesViewParams
// }

// impl GraphQLQueryParams for AccountBillsAndPropertiesQueryParams {
//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {
//         self.account.get_formal_part(params, GraphQL::prefix(prefix, "account"));
//     }

//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String) {
//     }

//     fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {
        
//         self.account.get_variables_part(variables, GraphQL::prefix(prefix, "account"))
//     }

//     // fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {

//     //     params.push_formal(&prefix, "accountNumber", "String!");
//     //     self.account.get_formal_part(params, Self::prefix(prefix, "account"));
//     // }

//     // fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String){
//     //     params.push_actual(&prefix, "accountNumber",);
//     // }



//     // fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {

//     //     variables.push_variable(&prefix, "accountNumber", &self.account_number)?;
//     //     self.account.get_variables_part(variables, Self::prefix(prefix, "account"))
//     // }
// }

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct AccountBillsAndPropertiesQuery {
//     pub account: AccountBillsAndPropertiesView
// }

// impl GraphQLType<AccountBillsAndPropertiesQueryParams> for AccountBillsAndPropertiesQuery {
//     fn get_query_part(params: &AccountBillsAndPropertiesQueryParams, prefix: String) -> String {
//         let account_prefix = GraphQL::prefix(prefix, "account");
//         format!(r#"
//             account{} {{
//                 {}
//             }}
//         "#, params.account.get_actual(&account_prefix),
//             // params.properties.get_actual("properties_"),
//             AccountBillsAndPropertiesView::get_query_part(&params.account, account_prefix)
//         )
//     }
    
//     // fn get_request_name() -> &'static str {
//     //     "getAccountBillsAndPropertiesView"
//     // }
// }






#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountBillsAndPropertiesQueryParams {
    account: AccountBillsAndPropertiesViewParams
}

impl GraphQLQueryParams for AccountBillsAndPropertiesQueryParams {
    fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {
        self.account.get_formal_part(params, GraphQL::prefix(&prefix, "account"));
    }

    fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String) {
    }

    fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {
        
        self.account.get_variables_part(variables, GraphQL::prefix(&prefix, "account"))
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AccountBillsAndPropertiesQuery {
    pub account: AccountBillsAndPropertiesView
}

impl GraphQLType<AccountBillsAndPropertiesQueryParams> for AccountBillsAndPropertiesQuery {
    fn get_query_part(params: &AccountBillsAndPropertiesQueryParams, prefix: String) -> String {
        format!(r#"
            account{} {{
                {}
            }}
        "#, params.account.get_actual(GraphQL::prefix(&prefix, "account")),
            // params.properties.get_actual("properties_"),
            AccountBillsAndPropertiesView::get_query_part(&params.account, GraphQL::prefix(&prefix, "account"))
        )
    }
    
    // fn get_request_name() -> &'static str {
    //     "getAccountPropertiesView"
    // }
}









fn test_account_bills_and_properties_view() {


    let params = AccountBillsAndPropertiesQueryParams {
        account: AccountBillsAndPropertiesViewParams {
            account_number: String::from("A-B3D8B29D"),
            properties: PropertySimpleViewParams {
                active_from: Some(String::from("2024-08-12T23:00:00.000Z")),
            },
            bills: BillParams {
                first: 2,
                transactions: TransactionSimpleViewParams {
                    first: 3
                }
            }
        }
    };

    let expected_variables: HashMap<String, serde_json::Value> = serde_json::from_str(r#"{
        "account_bills_transactions_first": 3,
        "account_properties_activeFrom": "2024-08-12T23:00:00.000Z",
        "account_accountNumber": "A-B3D8B29D",
        "account_bills_first": 2
      }"#).unwrap();
      
          let variables = params.get_variable_map().unwrap();
      
          println!("variables = r#\"{}\"#", params.get_variables().unwrap());
      
          assert!(variables == expected_variables);



    let expected_query = r#"
            query getAccountBillsAndProperties($account_accountNumber: String!, $account_properties_activeFrom: DateTime, $account_bills_first: Int, $account_bills_transactions_first: Int) {
                
            account(accountNumber: $account_accountNumber) {
                id
properties(activeFrom: $account_properties_activeFrom){
    
                id
                postcode
        
}
bills(first: $account_bills_first){
    pageInfo {
        startCursor
        hasNextPage
    }
    edges {
        node {
                
                id
                fromDate
                toDate
                ...on StatementType {
                    transactions(first: $account_bills_transactions_first) {
                        pageInfo
                        {
                            startCursor
                            hasNextPage
                        }
                        edges
                        {
                            node
                            {
                                id
postedDate
__typename

                            }
                        }
                    }
                }
        
        }
    }
}

            }
        
            }
        "#;
    let query = AccountBillsAndPropertiesQuery::get_query(String::from("getAccountBillsAndProperties"), &params);

    println!("\n\nquery = r#\"{}\"#", query);
    println!("expected_query = r#\"{}\"#", expected_query);

    assert_eq!(query, expected_query);

    
}

fn main() {
    test_simple_properties_view();

    test_account_bills_and_properties_view();
}





























// impl GraphQLElement for TestPropertySimpleView {
//     fn get_field_names() -> &'static str {
//         "id\npost_code\n"
//     }
// }

// #[derive(GraphQLQueryParams)]
// pub struct TestAccountPropertiesViewQueryParams {
//     pub active_from: Option<String>
// }


// impl GraphQLQueryParams for TestAccountQueryParams {
//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {
//         params.push_actual(&prefix, "activeFrom");
//         params.push_actual(&prefix, "accountNumber");
//         self.properties.get_formal_part(params, Self::prefix(prefix, "properties"));
//     }
//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String) {
//         params.push_formal(&prefix, "activeFrom", "string");
//         params.push_formal(&prefix, "accountNumber", "string");
//     }
//     fn get_variables_part(
//         &self,
//         variables: &mut VariableBuffer,
//         prefix: String,
//     ) -> Result<(), serde_json::Error> {
//         variables.push_variable(&prefix, "activeFrom", &self.active_from)?;
//         variables.push_variable(&prefix, "accountNumber", &self.account_number)?;
//         self.properties
//             .get_variables_part(variables, Self::prefix(prefix, "properties"))?;
//         Ok(())
//     }
// }

/*

pub struct PancakesBuilder {
    id: Option<String>,
    postcode: Option<String>,
}

*/

// impl HelloMacro for Pancakes {
//     fn hello_macro() {
//         println!("Hello, Macro! My name is Pancakes!");
//     }
// }


// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct PropertySimpleViewQueryParams {
//     pub active_from: Option<String>
// }

// impl GraphQLQueryParams for PropertySimpleViewQueryParams {
//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {

//         params.push_formal(&prefix, "activeFrom", "DateTime");
//     }

//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String){
//         params.push_actual(&prefix, "activeFrom",);
//     }

//     fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {

//         variables.push_variable(&prefix, "activeFrom", &self.active_from)
//     }

// }




// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct AccountPropertiesViewQueryParams {
//     pub account_number: String,
//     pub properties: PropertySimpleViewQueryParams
// }

// impl GraphQLQueryParams for AccountPropertiesViewQueryParams {

//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {

//         params.push_formal(&prefix, "accountNumber", "String!");
//         self.properties.get_formal_part(params, Self::prefix(prefix, "properties"));
//     }

//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String){
//         params.push_actual(&prefix, "accountNumber",);
//     }



//     fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {

//         variables.push_variable(&prefix, "accountNumber", &self.account_number)?;
//         self.properties.get_variables_part(variables, Self::prefix(prefix, "properties"))
//     }
// }














