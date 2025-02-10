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

use marco_sparko::octopus::bill::AccountBillsView;
use marco_sparko::octopus::Client;
use marco_sparko::Error;

fn main() -> Result<(), Error> {
    let json = r#"{
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
}"#;


        // let result: AccountBillsView = serde_json::from_str(json).unwrap();
        
        // let _ = Client::handle_bill(&result);
    
    Ok(())
}