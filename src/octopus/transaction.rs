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

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::gql::types::{Boolean, Date, DateTime, Int, ID};
use super::{ consumption::Consumption, decimal::Decimal, page_info::ForwardPageInfo, Error};



#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageOfTransactions {
    page_info: ForwardPageInfo,
    edges: Vec<TransactionEdge>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionEdge {
    node: Transaction
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "__typename")]
pub enum Transaction {
  Charge(Charge),
  Credit(TransactionTypeInterface),
  Payment(TransactionTypeInterface),
  Refund(TransactionTypeInterface)
}

impl Display for Transaction {
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTypeInterface {
  id: ID,
  posted_date: Date,
  created_at: DateTime,
  account_number: String,
  amounts: TransactionAmountType,
  balance_carried_forward: Int,
  is_held: Boolean,
  is_issued: Boolean,
  title: String,
  billing_document_identifier: ID,
  is_reversed: Boolean,
  has_statement: Boolean,
  note: Option<String>
}

impl TransactionTypeInterface {
  pub fn get_field_names() -> &'static str {
    r#"
      id
      postedDate
      createdAt
      accountNumber
      amounts
      balanceCarriedForward
      isHeld
      isIssued
      title
      billingDocumentIdentifier
      isReversed,
      hasStatement
      note
    "#
  }
}





#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Charge {
  id: ID,
  posted_date: Date,
  created_at: DateTime,
  account_number: String,
  amounts: TransactionAmountType,
  balance_carried_forward: Int,
  is_held: Boolean,
  is_issued: Boolean,
  title: String,
  billing_document_identifier: ID,
  is_reversed: Boolean,
  has_statement: Boolean,
  note: Option<String>,
  consumption: Consumption,
  is_export: Boolean
}

impl Charge {
    pub fn get_field_names() -> &'static str {
      r#"
        id
        postedDate
        createdAt
        accountNumber
        amounts
        balanceCarriedForward
        isHeld
        isIssued
        title
        billingDocumentIdentifier
        isReversed,
        hasStatement
        note
        consumption
        isExport
      "#
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionAmountType {
    net: Int,
    tax: Int,
    gross: Int,
}

impl TransactionAmountType {
    pub fn get_field_names() -> &'static str {
        r#"
            net
            tax
            gross
        "#
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// enum StatementReversalsAfterClose {
//     All,
//     Some,
//     None,
//     NotClosed
// }


// #[derive(Serialize, Deserialize, Debug)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// enum AccountStatementStatus {
//     Open,
//     Closed
// }

#[cfg(test)]
mod tests {
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
  assert_eq!(charge.amounts.net, Int::new(2711));
  assert_eq!(charge.amounts.tax, Int::new(136));
  assert_eq!(charge.amounts.gross, Int::new(2847));

  let transaction: Transaction = serde_json::from_str(json).unwrap();

  if let Transaction::Charge(charge) = transaction {
      assert_eq!(charge.amounts.net, Int::new(2711));
      assert_eq!(charge.amounts.tax, Int::new(136));
      assert_eq!(charge.amounts.gross, Int::new(2847));
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
    "note": "",
    "__typename": "Credit"
  }
  "#;

  let credit: TransactionTypeInterface = serde_json::from_str(json).unwrap();
  assert_eq!(credit.amounts.net, Int::new(478));
  assert_eq!(credit.amounts.tax, Int::new(24));
  assert_eq!(credit.amounts.gross, Int::new(502));

  let transaction: Transaction = serde_json::from_str(json).unwrap();

  if let Transaction::Credit(credit) = transaction {
      assert_eq!(credit.amounts.net, Int::new(478));
      assert_eq!(credit.amounts.tax, Int::new(24));
      assert_eq!(credit.amounts.gross, Int::new(502));
  }
  else {
      panic!("Expected Credit not {:?}", transaction);
  }

}

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
let page: PageOfTransactions = serde_json::from_str(json).unwrap();

assert_eq!(page.page_info.has_next_page, false);

if let Transaction::Charge(charge) = &page.edges[0].node {
  assert_eq!(charge.amounts.net, Int::new(2711));
  assert_eq!(charge.amounts.tax, Int::new(136));
  assert_eq!(charge.amounts.gross, Int::new(2847));
}
else {
  panic!("Expected first transaction to be Charge not {}", &page.edges[0].node);
}

// if let Bill::Statement(statement) = bill {
//     assert_eq!(statement.total_credits.net_total, Int::new(1667));
//     assert_eq!(statement.total_credits.tax_total, Int::new(85));
//     assert_eq!(statement.total_credits.gross_total, Int::new(1752));
// }
// else {
//     panic!("Expected Statement not {:?}", bill);
// }

}


}