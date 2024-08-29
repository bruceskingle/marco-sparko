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

use std::collections::BTreeMap;

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

use crate::{gql::types::{Date, Int, ID}, octopus::decimal::Decimal};
use super::{ page_info::ForwardPageInfo, transaction::PageOfTransactions, transaction::Transaction};

// // Represents AccountUserType in the GraphQL schema
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct AccountBillResults {
//     account: BillResults
// }

// impl AccountBillResults {
//     pub fn get_field_names() -> String {
//         format!(r#"
//         account {{
//             {}
//         }}
//         "#, BillResults::get_field_names())
//     }
// }

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct BillResults {
    pub status: String,
    pub number: String,
    pub balance: i32,
    pub bills: PageOfBills
}

impl BillResults {
    pub fn get_field_names() -> String {
        format!(r#"
        status
        number
        balance
        bills (first: 1) {{
                {}
        }}
        "#, PageOfBills::get_field_names())
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct PageOfBills {
    pub page_info: ForwardPageInfo,
    pub edges: Vec<BillEdge>
}

impl PageOfBills {
    pub fn get_field_names() -> String {
        format!(r#"
        pageInfo {{
                {}
        }}
        edges {{
                {}
        }}
        "#, ForwardPageInfo::get_field_names(), BillEdge::get_field_names())
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct BillEdge {
    pub node: Bill
}

impl BillEdge {
    pub fn get_field_names() -> String {
        format!(r#"
        node {{
            {}
        }}
        "#, Bill::get_field_names())
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "billType")]
pub enum Bill {
    Statement(StatementType)
}

impl Bill {
    pub fn get_field_names() -> String {
        format!(r#"
        {}
        ...on StatementType {{
            {}
        }}
        "#, BillInterfaceType::get_field_names(), StatementType::get_field_names())
    }

    pub fn print(&self) {
        match self {
            Bill::Statement(bill) => bill.print(),
        };
    }
}

// impl Bill {
//     pub fn from_json(json: &str) -> Result<Bill, Error> {
//         // let value: serde_json::Value = serde_json::from_str(json)?;
//         let bill_interface: BillInterfaceType = serde_json::from_str(json)?;

//         match bill_interface.bill_type {
//             BillTypeEnum::Statement => {
//                 let result: StatementType = serde_json::from_str(json)?;
//                 Ok(Bill::Statement(result))
//             },
//             BillTypeEnum::Invoice => todo!(),
//             BillTypeEnum::CreditNote => todo!(),
//             BillTypeEnum::PreKraken => todo!(),
//         }
//     }
// }

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct BillInterfaceType {
    pub id: ID,
    // bill_type: BillTypeEnum,
    pub from_date: Date,
    pub to_date: Date,

    // Requesting this field generates a temporary URL at which bill is available.
    //         This URL will expire after approximately an hour.  It is intended for redirection purposes,
    //         NOT persistence in any form (e.g. inclusion in emails or the body of a web page).
    //         This field can raise an error with errorClass NOT_FOUND if the bill document has not
    //         been created/issued yet.
    //
    //
    //
    // temporary_url: String,

    // The date the bill was sent to the customer.
    pub issued_date: Date
}

impl BillInterfaceType {
    pub fn get_field_names() -> &'static str {
        r#"
        id
        fromDate
        toDate
        issuedDate
        billType
        "#
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct StatementType {
    pub id: ID,
    // bill_type: BillTypeEnum,
    pub from_date: Date,
    pub to_date: Date,

    // Requesting this field generates a temporary URL at which bill is available.
    //         This URL will expire after approximately an hour.  It is intended for redirection purposes,
    //         NOT persistence in any form (e.g. inclusion in emails or the body of a web page).
    //         This field can raise an error with errorClass NOT_FOUND if the bill document has not
    //         been created/issued yet.
    //
    //
    //
    // temporary_url: String,

    // The date the bill was sent to the customer.
    pub issued_date: Date,

    // This field returns the closing balance of an issued statement.
    pub closing_balance: Int,

    // This field returns the opening balance of a statement.
    pub opening_balance: Int,

    // Whether the bill originated in Kraken or externally.
    pub is_external_bill: bool,

    // Transactions on the bill.
    //   transactions(
    //     before: String
    //     after: String
    //     first: Int
    //     last: Int
    //   ): TransactionConnectionTypeConnection
    pub transactions: PageOfTransactions,

    // Email recipient user ID.
    pub user_id: Int,

    // Email recipient address.
    pub to_address: String,

    // The date the bill is due to be paid.
    pub payment_due_date: Date,

    // The first day of consumption that this statement includes.
    pub consumption_start_date: Option<Date>,

    // The last day of consumption that this statement includes.
    pub consumption_end_date: Option<Date>,

    // How many charges have been reversed after the close date.
    pub reversals_after_close: StatementReversalsAfterClose,

    // Current status of the associated statement.
    pub status: AccountStatementStatus,

    // Retrieve the held status of a account statement.
    pub held_status: HeldStatus,

    // The total amounts for all charges on the statement.
    pub total_charges: StatementTotalType,

    // The total amounts for all credits on the statement.
    pub total_credits: StatementTotalType
}

impl StatementType {
    pub fn get_field_names() -> String {
        format!(r#"
        closingBalance
        openingBalance
        isExternalBill
        transactions(first: 100) {{
            {}
        }}
        userId
        toAddress
        paymentDueDate
        consumptionStartDate
        consumptionEndDate
        reversalsAfterClose
        status
        heldStatus {{
            {}
        }}
        totalCharges {{
            {}
        }}
        totalCredits {{
            {}
        }}
        "#, PageOfTransactions::get_field_names(),
            HeldStatus::get_field_names(),
            StatementTotalType::get_field_names(),
            StatementTotalType::get_field_names())
    }

    pub fn print(&self) {
        println!("Energy Account Statement");
        println!("========================");
        println!("Date                 {}", self.issued_date);
        println!("Ref                  {}", self.id);
        println!("From                 {}", self.from_date);
        println!("To                   {}", self.to_date);
        println!();

        // let mut map = BTreeMap::new();
        // for edge in &self.transactions.edges {
        //     let txn = edge.node.as_transaction();

        //     map.insert(&txn.posted_date, &edge.node);
        // }

        print!("{:20} {:10} ", 
            "Description",
            "Posted"
        );
        print!("{:>10} {:>10} {:>10} {:>10} ", 
            "Net",
            "Tax", 
            "Total",
            "Balance"
        );
        print!("{:10} {:10} {:>12} ", 
            "From",
            "To",
            "Units"
        );
        println!();


        // for transaction in &mut map.values() {
        for edge in (&self.transactions.edges).into_iter().rev() {
            let transaction = &edge.node;
            let txn = transaction.as_transaction();
            if let Transaction::Charge(charge) = &transaction {
                if *charge.is_export {
                    print!("{} {:width$} ", txn.title, "Export", width = 20 - txn.title.len() - 1);
                }
                else {
                    print!("{} {:width$} ", txn.title, "Import",width =  20 - txn.title.len() - 1);
                }
            }
            else {
                print!("{:20} ", txn.title);
            }
            print!("{:10} ", 
                        txn.posted_date
                    );
            print!("{:>10} {:>10} {:>10} {:>10} ", 
                txn.amounts.net.as_decimal(2),
                txn.amounts.tax.as_decimal(2), 
                txn.amounts.gross.as_decimal(2),
                txn.balance_carried_forward.as_decimal(2)
            );
            if let Transaction::Charge(charge) = &transaction {
                print!("{:10} {:10} {:>12}", 
                    charge.consumption.start_date,
                    charge.consumption.end_date,
                    charge.consumption.quantity
                );

                let rate = Decimal::from_int(&txn.amounts.gross) / charge.consumption.quantity;

                print!(" {:>12}", rate);
            }
            println!();
        }
    }
}

// impl Display for StatementType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if let Ok(json) = serde_json::to_string_pretty(self) {
//             f.write_str(&json)?;
//             Ok(())
//         }
//         else {
//             Err(std::fmt::Error)
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// enum BillTypeEnum {
//     Statement,
//     Invoice,
//     CreditNote,
//     PreKraken
//   }

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct StatementTotalType {
    pub net_total: Int,
    pub tax_total: Int,
    pub gross_total: Int,
}

impl StatementTotalType {
    pub fn get_field_names() -> &'static str {
        r#"
            netTotal
            taxTotal
            grossTotal
        "#
    }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatementReversalsAfterClose {
    All,
    Some,
    None,
    NotClosed
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatementStatus {
    Open,
    Closed
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct HeldStatus {
    pub is_held: bool,
    pub reason: Option<String>
}

impl HeldStatus {
    pub fn get_field_names() -> &'static str {
        r#"
            isHeld
            reason
        "#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_statement_total() {
        let json = r#"
        {
            "netTotal": 1667,
            "taxTotal": 85,
            "grossTotal": 1752
        }
        "#;


        let value: StatementTotalType = serde_json::from_str(json).unwrap();
        
        assert_eq!(value.net_total, Int::new(1667));
        assert_eq!(value.tax_total, Int::new(85));
        assert_eq!(value.gross_total, Int::new(1752));
    }

    #[test]
    fn test_parse_statement() {
        let json = r#"
        {
            "id": "236646425",
            "billType": "STATEMENT",
            "fromDate": "2024-07-22",
            "toDate": "2024-08-21",
            "issuedDate": "2024-08-22",
            "__typename": "StatementType",
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
                    "isExport": false,
                    "__typename": "Charge"
                }
                },
                {
                "node": {
                    "id": "-1871043601",
                    "postedDate": "2024-08-20",
                    "createdAt": "2024-08-21T21:32:19.902722+00:00",
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
                    "isExport": false,
                    "__typename": "Charge"
                }
                },
                {
                "node": {
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
                },
                {
                "node": {
                    "id": "-1871043620",
                    "postedDate": "2024-08-12",
                    "createdAt": "2024-08-21T21:32:19.073366+00:00",
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
                    "isExport": true,
                    "__typename": "Charge"
                }
                },
                {
                "node": {
                    "id": "-1871044052",
                    "postedDate": "2024-08-07",
                    "createdAt": "2024-08-21T21:32:01.008991+00:00",
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
                    "isExport": false,
                    "__typename": "Charge"
                }
                },
                {
                "node": {
                    "id": "-1949392858",
                    "postedDate": "2024-07-29",
                    "createdAt": "2024-08-01T03:09:50.202838+00:00",
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
                    "note": null,
                    "__typename": "Payment"
                }
                },
                {
                "node": {
                    "id": "-1973989678",
                    "postedDate": "2024-07-24",
                    "createdAt": "2024-07-25T10:53:30.897903+00:00",
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
                    "note": "",
                    "__typename": "Credit"
                }
                },
                {
                "node": {
                    "id": "-1974036696",
                    "postedDate": "2024-07-24",
                    "createdAt": "2024-07-25T10:43:02.339290+00:00",
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
                    "note": "",
                    "__typename": "Credit"
                }
                },
                {
                "node": {
                    "id": "-1974103763",
                    "postedDate": "2024-07-24",
                    "createdAt": "2024-07-25T10:17:07.255688+00:00",
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
                    "note": "",
                    "__typename": "Credit"
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
        "#;

        // let bill = Bill::from_json(json).unwrap();
        let bill: Bill = serde_json::from_str(json).unwrap();
        
        if let Bill::Statement(statement) = bill {
            assert_eq!(statement.total_credits.net_total, Int::new(1667));
            assert_eq!(statement.total_credits.tax_total, Int::new(85));
            assert_eq!(statement.total_credits.gross_total, Int::new(1752));
        }
        else {
            panic!("Expected Statement not {:?}", bill);
        }
        
    }

    #[test]
    fn test_parse_bill_result() {
        let json = r#"
        {
            "status": "ACTIVE",
            "number": "A-B3D8B29D",
            "balance": 39303,
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
                    "__typename": "StatementType",
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
                            "isExport": false,
                            "__typename": "Charge"
                            }
                        },
                        {
                            "node": {
                            "id": "-1871043601",
                            "postedDate": "2024-08-20",
                            "createdAt": "2024-08-21T21:32:19.902722+00:00",
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
                            "isExport": false,
                            "__typename": "Charge"
                            }
                        },
                        {
                            "node": {
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
                        },
                        {
                            "node": {
                            "id": "-1871043620",
                            "postedDate": "2024-08-12",
                            "createdAt": "2024-08-21T21:32:19.073366+00:00",
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
                            "isExport": true,
                            "__typename": "Charge"
                            }
                        },
                        {
                            "node": {
                            "id": "-1871044052",
                            "postedDate": "2024-08-07",
                            "createdAt": "2024-08-21T21:32:01.008991+00:00",
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
                            "isExport": false,
                            "__typename": "Charge"
                            }
                        },
                        {
                            "node": {
                            "id": "-1949392858",
                            "postedDate": "2024-07-29",
                            "createdAt": "2024-08-01T03:09:50.202838+00:00",
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
                            "note": null,
                            "__typename": "Payment"
                            }
                        },
                        {
                            "node": {
                            "id": "-1973989678",
                            "postedDate": "2024-07-24",
                            "createdAt": "2024-07-25T10:53:30.897903+00:00",
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
                            "note": "",
                            "__typename": "Credit"
                            }
                        },
                        {
                            "node": {
                            "id": "-1974036696",
                            "postedDate": "2024-07-24",
                            "createdAt": "2024-07-25T10:43:02.339290+00:00",
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
                            "note": "",
                            "__typename": "Credit"
                            }
                        },
                        {
                            "node": {
                            "id": "-1974103763",
                            "postedDate": "2024-07-24",
                            "createdAt": "2024-07-25T10:17:07.255688+00:00",
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
                            "note": "",
                            "__typename": "Credit"
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

        let bill_results: BillResults = serde_json::from_str(json).unwrap();

        assert_eq!(bill_results.number, "A-B3D8B29D");

    }
}