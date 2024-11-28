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

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

use sparko_graphql::NoParams;
// use sparko_graphql_derive::{GraphQLQueryParams, GraphQLType};

use sparko_graphql::{types::{Boolean, Date, DateTime, Int, ID}, GraphQLQueryParams, GraphQLType, GraphQL, ParamBuffer, VariableBuffer};
use super::consumption::Consumption;




// #[derive(GraphQLQueryParams)]
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct TransactionSimpleViewParams {
//     pub first: Int
// }

// #[derive(GraphQLType)]
// #[graphql(params = "TransactionSimpleViewParams")]
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct TransactionSimpleView {
//     pub id: String,
//     pub posted_date: String,
//     pub __typename: String
// }



// PROTOTYPE FOR PAGINATION

// #[derive(GraphQLQueryParams)]
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct StatementTransactionParams {
//     pub before: String,
//     pub after: String,
//     pub first: Int,
//     pub last: Int,
// }

// #[derive(GraphQLQueryParams)]
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct TransactionParams {
//   pub amounts: NoParams,
//   pub consumption: NoParams,
// }

#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatementTransactionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<Int>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<Int>
}

#[derive(GraphQLType)]
#[graphql(params = "StatementTransactionParams")]
#[graphql(super_type = ["TransactionTypeInterface"])]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(tag = "__typename")]
pub enum Transaction {
  Charge(Charge),
  Credit(AbstractTransaction),
  Payment(AbstractTransaction),
  Refund(AbstractTransaction)
}

impl Transaction {
  pub fn as_transaction(&self) -> &TransactionTypeInterface {
    match self {
      Transaction::Charge(txn) => &txn.transaction,
      Transaction::Credit(txn) => &txn.transaction,
      Transaction::Payment(txn) => &txn.transaction,
      Transaction::Refund(txn) => &txn.transaction,
    }
  }
}

#[derive(GraphQLType)]
#[graphql(params = "StatementTransactionParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTypeInterface {
    pub id: ID,
    pub posted_date: Date,
    pub created_at: DateTime,
    pub account_number: String,
    #[graphql(no_params)]
    pub amounts: TransactionAmountType,
    pub balance_carried_forward: Int,
    pub is_held: Boolean,
    pub is_issued: Boolean,
    pub title: String,
    pub billing_document_identifier: ID,
    pub is_reversed: Boolean,
    pub has_statement: Boolean,
    pub note: Option<String>
}

// Several variants have no additional fields so this implements for all of them

#[derive(GraphQLType)]
#[graphql(params = "StatementTransactionParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AbstractTransaction{
  #[serde(flatten)]
  pub transaction: TransactionTypeInterface
}

#[derive(GraphQLType)]
#[graphql(params = "StatementTransactionParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct Charge {
    #[serde(flatten)]
    pub transaction: TransactionTypeInterface,
    #[graphql(no_params)]
    pub consumption: Consumption,
    pub is_export: Boolean
}


#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct TransactionAmountType {
    pub net: Int,
    pub tax: Int,
    pub gross: Int,
}

#[cfg(test)]
mod tests {
  use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};
use sparko_graphql::types::{Boolean, Date, Int};

    #[test]
    fn test_tagged_type() {
      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      #[serde(tag = "__typename")]
      pub enum Transaction {
        Charge(Charge),
        Credit(AbstractTransaction),
        // Payment(AbstractTransaction),
        // Refund(AbstractTransaction)
      }


      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      #[serde(rename_all = "camelCase")]
      pub struct TransactionTypeInterface {
          pub posted_date: Date,
          pub account_number: String,
          // pub tag: String,
      }

      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      #[serde(rename_all = "camelCase")]
      pub struct AbstractTransaction{
        pub posted_date: Date,
        pub account_number: String,
        // pub tag: String,
      }

      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      #[serde(rename_all = "camelCase")]
      pub struct Charge {
        
        pub posted_date: Date,
        pub account_number: String,
        // pub tag: String,
        pub consumption: Int,
        pub is_export: Boolean
      }

      let charge = Charge {
        posted_date: Date::from_calendar_date(2024,time::Month::April,12).unwrap(),
        account_number: "1234".to_string(),
        // tag: "Charge".to_string(),
        consumption: Int::new(4),
        is_export: false.into(),
      };

      let input_transaction: Transaction = Transaction::Charge(charge);

      let serialized = serde_json::to_string(&input_transaction).unwrap();

      println!("<Serialized>{}</Serialized>\n\n", &serialized);

      

      let abstract_transaction: AbstractTransaction = serde_json::from_str(&serialized).unwrap();

      println!("<AbstractTransaction>{}</AbstractTransaction>\n\n", serde_json::to_string_pretty(&abstract_transaction).unwrap());
      
      let charge: Charge = serde_json::from_str(&serialized).unwrap();

      println!("<Charge>{}</Charge>\n\n", serde_json::to_string_pretty(&charge).unwrap());

      let transaction_type_interface: TransactionTypeInterface = serde_json::from_str(&serialized).unwrap();

      println!("<TransactionTypeInterface>{}</TransactionTypeInterface>\n\n", serde_json::to_string_pretty(&transaction_type_interface).unwrap());

      fn deserialize(name: &str, input: &str) {
        let transaction: Transaction = serde_json::from_str(input).unwrap();

        println!("<{}>{}</{}>\n\n", name, serde_json::to_string_pretty(&transaction).unwrap(), name);
      }
      
      deserialize("Transaction", &serialized);
      deserialize("PreCamelTransaction", r#"{"postedDate":"2024-04-12","accountNumber":"1234","__typename":"Charge","consumption":4,"isExport":false}"#);

    }



    #[test]
    fn test_get_attributes() {
      let params = StatementTransactionParams {
        before: None,
        after: None,
        first: None,
        last: None,
    };
      let result = Transaction::get_query_attributes(&params, "");
      let expected = r#"
      # transaction
              __typename
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
# flattened transaction
consumption
  # object "consumption"
    { #get_query_part
  startDate
endDate
quantity
unit
usageCost
supplyCharge

} #/get_query_part

  # /object "consumption"
isExport

}

      # /transaction
      "#;

      let expected = r#"__typename
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

  # enum variant charge
  ...on Charge {
# flattened transaction
consumption
  # object "consumption"
    { #get_query_part
  startDate
endDate
quantity
unit
usageCost
supplyCharge

} #/get_query_part

  # /object "consumption"
isExport

}

  # /enum variant charge
  # enum variant credit
  # flattened transaction

  # /enum variant credit
  # enum variant payment
  # flattened transaction

  # /enum variant payment
  # enum variant refund
  # flattened transaction

  # /enum variant refund
"#;

      assert_eq!(expected, result);
    }



    #[test]
    fn test_flat_tagged_type() {
      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      #[serde(tag = "__typename")]
      pub enum Transaction {
        Charge(Charge),
        Credit(AbstractTransaction),
        // Payment(AbstractTransaction),
        // Refund(AbstractTransaction)
      }


      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      #[serde(rename_all = "camelCase")]
      pub struct TransactionTypeInterface {
          pub posted_date: Date,
          pub account_number: String,
      }

      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      #[serde(rename_all = "camelCase")]
      pub struct AbstractTransaction{
        #[serde(flatten)]
        pub transaction: TransactionTypeInterface
      }

      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      #[serde(rename_all = "camelCase")]
      pub struct Charge {
        #[serde(flatten)]
        pub transaction: TransactionTypeInterface,
        pub consumption: Int,
        pub is_export: Boolean
      }

      let charge = Charge {
        transaction: TransactionTypeInterface {
          posted_date: Date::from_calendar_date(2024,time::Month::April,12).unwrap(),
          account_number: "1234".to_string(),
        },
        consumption: Int::new(4),
        is_export: false.into(),
      };

      let input_transaction: Transaction = Transaction::Charge(charge);

      let serialized = serde_json::to_string(&input_transaction).unwrap();

      println!("<Serialized>{}</Serialized>\n\n", &serialized);

      

      let abstract_transaction: AbstractTransaction = serde_json::from_str(&serialized).unwrap();

      println!("<AbstractTransaction>{}</AbstractTransaction>\n\n", serde_json::to_string_pretty(&abstract_transaction).unwrap());
      
      let charge: Charge = serde_json::from_str(&serialized).unwrap();

      println!("<Charge>{}</Charge>\n\n", serde_json::to_string_pretty(&charge).unwrap());

      let transaction_type_interface: TransactionTypeInterface = serde_json::from_str(&serialized).unwrap();

      println!("<TransactionTypeInterface>{}</TransactionTypeInterface>\n\n", serde_json::to_string_pretty(&transaction_type_interface).unwrap());

      fn deserialize(name: &str, input: &str) {
        let transaction: Transaction = serde_json::from_str(input).unwrap();

        println!("<{}>{}</{}>\n\n", name, serde_json::to_string_pretty(&transaction).unwrap(), name);
      }
      
      deserialize("Transaction", &serialized);
      // deserialize("PreSnakeTransaction", r#"{"posted_date":"2024-04-12","account_number":"1234","__typename":"Charge","consumption":4,"is_export":false}"#);
      deserialize("PreCamelTransaction", r#"{"postedDate":"2024-04-12","accountNumber":"1234","__typename":"Charge","consumption":4,"isExport":false}"#);

    }
    use sparko_graphql::types::ForwardPageOf;

    use crate::octopus::consumption::ConsumptionUnit;

    use super::*;

    #[test]
    fn test_parse_transaction_amount() {
        let json = r#"
        {
            "net": 1667,
            "tax": 85,
            "gross": 1752
        }
        "#;


        let value: TransactionAmountType = serde_json::from_str(json).unwrap();
        
        assert_eq!(value.net, Int::new(1667));
        assert_eq!(value.tax, Int::new(85));
        assert_eq!(value.gross, Int::new(1752));
    }






//     #[test]
//     fn test_parse_payment_transaction() {
//         let json = r#"
//     {
//       "postedDate": "2024-07-29",
//       "createdAt": "2024-08-01T03:09:50.202838+00:00",
//       "amounts": {
//         "net": 24790,
//         "tax": 0,
//         "gross": 0
//       },
//       "balanceCarriedForward": 43831,
//       "isHeld": false,
//       "isIssued": true,
//       "title": "Direct debit",
//       "isReversed": false,
//       "hasStatement": true,
//       "note": null,
//       "__typename": "Payment"
//     }
//     "#;

//     // let bill = Bill::from_json(json).unwrap();
//     let bill: Bill = serde_json::from_str(json).unwrap();
    
//     if let Bill::Statement(statement) = bill {
//         assert_eq!(statement.total_credits.net_total, Int::new(1667));
//         assert_eq!(statement.total_credits.tax_total, Int::new(85));
//         assert_eq!(statement.total_credits.gross_total, Int::new(1752));
//     }
//     else {
//         panic!("Expected Statement not {:?}", bill);
//     }
    
// }

#[test]
fn test_parse_charge_transaction() {
  let json = r#"
  {
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
    "consumption": {
      "startDate": "2024-07-21",
      "endDate": "2024-08-20",
      "quantity": "360.7100",
      "unit": "kWh",
      "usageCost": 0,
      "supplyCharge": 0
    },
    "isExport": false,
    "__typename": "Charge"
  }
  "#;

  let charge: Charge = serde_json::from_str(json).unwrap();
  assert_eq!(charge.transaction.amounts.net, Int::new(2711));
  assert_eq!(charge.transaction.amounts.tax, Int::new(136));
  assert_eq!(charge.transaction.amounts.gross, Int::new(2847));

  assert_eq!(charge.consumption.unit, ConsumptionUnit::KWH);

  let transaction: Transaction = serde_json::from_str(json).unwrap();

  if let Transaction::Charge(charge) = transaction {
      assert_eq!(charge.transaction.amounts.net, Int::new(2711));
      assert_eq!(charge.transaction.amounts.tax, Int::new(136));
      assert_eq!(charge.transaction.amounts.gross, Int::new(2847));
  }
  else {
      panic!("Expected Charge not {:?}", transaction);
  }

}

#[test]
fn test_parse_credit_transaction() {
  let json = r#"
{
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
"#;

  let credit: TransactionTypeInterface = serde_json::from_str(json).unwrap();
  assert_eq!(credit.amounts.net, Int::new(478));
  assert_eq!(credit.amounts.tax, Int::new(24));
  assert_eq!(credit.amounts.gross, Int::new(502));

  println!("Got credit {}", &serde_json::to_string_pretty(&credit).unwrap());

  let transaction: Transaction = serde_json::from_str(json).unwrap();

  if let Transaction::Credit(credit) = transaction {
      assert_eq!(credit.transaction.amounts.net, Int::new(478));
      assert_eq!(credit.transaction.amounts.tax, Int::new(24));
      assert_eq!(credit.transaction.amounts.gross, Int::new(502));
  }
  else {
      panic!("Expected Credit not {:?}", transaction);
  }

}

// #[test]
// fn test_get_field_names() {
//   let field_names = PageOfTransactions::get_query(());

//   println!("{}", field_names);

//   assert_eq!(PageOfTransactions::get_field_names(), String::from("foo"));
// }

#[test]
fn test_parse_page() {
  let json = r#"
  {
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
          "consumption": {
            "startDate": "2024-07-21",
            "endDate": "2024-08-20",
            "quantity": "360.7100",
            "unit": "kWh",
            "usageCost": 0,
            "supplyCharge": 0
          },
          "isExport": false,
          "__typename": "Charge"
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
          "consumption": {
            "startDate": "2024-08-13",
            "endDate": "2024-08-20",
            "quantity": "181.0500",
            "unit": "kWh",
            "usageCost": 0,
            "supplyCharge": 0
          },
          "isExport": true,
          "__typename": "Charge"
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
          "consumption": {
            "startDate": "2024-08-08",
            "endDate": "2024-08-20",
            "quantity": "334.7100",
            "unit": "kWh",
            "usageCost": 0,
            "supplyCharge": 0
          },
          "isExport": false,
          "__typename": "Charge"
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
          "consumption": {
            "startDate": "2024-07-21",
            "endDate": "2024-08-12",
            "quantity": "300.8200",
            "unit": "kWh",
            "usageCost": 0,
            "supplyCharge": 0
          },
          "isExport": true,
          "__typename": "Charge"
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
          "consumption": {
            "startDate": "2024-07-21",
            "endDate": "2024-08-07",
            "quantity": "322.5100",
            "unit": "kWh",
            "usageCost": 0,
            "supplyCharge": 0
          },
          "isExport": false,
          "__typename": "Charge"
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
  }
  "#;

  // let bill = Bill::from_json(json).unwrap();
  let page: ForwardPageOf<Transaction> = serde_json::from_str(json).unwrap();

  assert_eq!(page.page_info.has_next_page, false);

  if let Transaction::Charge(charge) = &page.edges[0].node {
    assert_eq!(charge.transaction.amounts.net, Int::new(2711));
    assert_eq!(charge.transaction.amounts.tax, Int::new(136));
    assert_eq!(charge.transaction.amounts.gross, Int::new(2847));
  }
  else {
    panic!("Expected first transaction to be Charge not {}", &page.edges[0].node);
  }

  if let Transaction::Credit(credit) = &page.edges[3].node {
    assert_eq!(credit.transaction.amounts.net, Int::new(478));
    assert_eq!(credit.transaction.amounts.tax, Int::new(24));
    assert_eq!(credit.transaction.amounts.gross, Int::new(502));
  }
  else {
    panic!("Expected 4th transaction to be Credit not {}", &page.edges[0].node);
  }
}


}