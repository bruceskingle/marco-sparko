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

use super::error::Error;

/*
union ElectricityTariffType =
    StandardTariff
  | DayNightTariff
  | ThreeRateTariff
  | HalfHourlyTariff
  | PrepayTariff



interface TariffType {
  id: ID
  displayName: String
  fullName: String
  description: String
  productCode: String
  standingCharge: Float
  preVatStandingCharge: Float

  # Describes a particular tariff by combining the product code, number of rates, available from date and GSP code.
  tariffCode: String
}





type HalfHourlyTariff implements TariffType {
  id: ID
  displayName: String
  fullName: String
  description: String
  productCode: String
  standingCharge: Float
  preVatStandingCharge: Float

  # Describes a particular tariff by combining the product code, number of rates, available from date and GSP code.
  tariffCode: String
  unitRates: [UnitRate]

  # Information on how agile unit rates have been calculated.
  agileCalculationInfo: AgileCalculationInfo
}

type UnitRate {
  validFrom: DateTime
  validTo: DateTime

  # Price in pence (inc VAT).
  value: Float

  # Price in pence (not including VAT).
  preVatValue: Float
}

type AgileCalculationInfo {
  # The maximum value/cap for the unit rate.
  priceCap: Float

  # The peak offset for the unit rate.
  peakOffset: Float

  # The price multiplier/coefficient used to calculate the unit rate.
  gspCoefficient: Float
}

type PrepayTariff implements TariffType {
  id: ID
  displayName: String
  fullName: String
  description: String
  productCode: String
  standingCharge: Float
  preVatStandingCharge: Float

  # Describes a particular tariff by combining the product code, number of rates, available from date and GSP code.
  tariffCode: String
  unitRate: Float
  preVatUnitRate: Float
}
*/


// 
/*
Represents StandardTariff in the GraphQL schema
type StandardTariff implements TariffType {
  id: ID
  displayName: String
  fullName: String
  description: String
  productCode: String
  standingCharge: Float
  preVatStandingCharge: Float

  # Describes a particular tariff by combining the product code, number of rates, available from date and GSP code.
  tariffCode: String
  unitRate: Float

  # Is EPG applied to the unit rate.
  unitRateEpgApplied: Boolean
  preVatUnitRate: Float
}
*/

// pub trait Tariff {
//     fn from(value: serde_json::Value) -> Result<Box<dyn Tariff>, Error> {
//        if let Some(type_name) = value.get("__typename") {
//         match(type_name) {
//             "StandardTariff" => {
//                 let tariff: StandardTariff = serde_json::from_value(value);
//                 Box::new(tariff)
//             },
//             _ => Err(Error::StringError(format!("Invalid Tariff type {}", type_name)))
//         }
//        }
//        else {
//         Err(Error::StringError(format!("No __typename attribute for Tariff")))
//        }
//     }
// }


#[derive(Serialize, Debug)]
pub enum  Tariff {
    Standard(StandardTariff),
    DayNight(DayNightTariff),
    ThreeRate(ThreeRateTariff)
}

impl Tariff {
  pub fn from(value: serde_json::Value) -> Result<Tariff, Error> {
    println!("Got {:?}", value);
    println!("Got {:?}", value.is_object());
    println!("Got {:?}", value.as_str());
    
    if let Some(type_name_value) = value.get("__typename") {
    if type_name_value.is_string() {
        let type_name = type_name_value.as_str().unwrap();
        match type_name {
            "StandardTariff" => {
                let tariff: StandardTariff = serde_json::from_value(value)?;
                Ok(Tariff::Standard(tariff))
            },
            "DayNightTariff" => {
              let tariff: DayNightTariff = serde_json::from_value(value)?;
              Ok(Tariff::DayNight(tariff))
            },
            "ThreeRateTariff" => {
              let tariff: ThreeRateTariff = serde_json::from_value(value)?;
              Ok(Tariff::ThreeRate(tariff))
            },
            _ => Err(Error::StringError(format!("Invalid Tariff type {}", type_name)))
        }
        }
        else {
            Err(Error::StringError(format!("No __typename attribute for Tariff")))
           }
    }
    else {
     Err(Error::StringError(format!("No __typename attribute for Tariff")))
    }
 }
 

 /*
 All field names

 {
                            ... on TariffType {
                                id
                                displayName
                                fullName
                                description
                                productCode
                                standingCharge
                                preVatStandingCharge
                                tariffCode
                            }
                            ... on StandardTariff {
                                unitRate
                                unitRateEpgApplied
                                preVatUnitRate
                            }
                            ... on DayNightTariff {
                                dayRate

                                # Is EPG applied to the unit rate.
                                dayRateEpgApplied
                                nightRate

                                # Is EPG applied to the unit rate.
                                nightRateEpgApplied
                                preVatDayRate
                                preVatNightRate
                            }
                            ... on ThreeRateTariff {
                                dayRate

                                # Is EPG applied to the unit rate.
                                dayRateEpgApplied
                                nightRate

                                # Is EPG applied to the unit rate.
                                nightRateEpgApplied
                                offPeakRate

                                # Is EPG applied to the unit rate.
                                offPeakRateEpgApplied
                                preVatDayRate
                                preVatNightRate
                                preVatOffPeakRate
                            }
                            ... on HalfHourlyTariff {
                            unitRates {
                                validFrom
                                validTo

                                # Price in pence (inc VAT).
                                value

                                # Price in pence (not including VAT).
                                preVatValue
                                    # Information on how agile unit rates have been calculated.
                                agileCalculationInfo  {
                                    # The maximum value/cap for the unit rate.
                                    priceCap

                                    # The peak offset for the unit rate.
                                    peakOffset

                                    # The price multiplier/coefficient used to calculate the unit rate.
                                    gspCoefficient
                                }
                            }

                            
                            ... on PrepayTariff {
                                unitRate
                                preVatUnitRate
                            }
                            __typename
                            }
                        }
 
  */

 pub fn get_energy_tariff_type_field_names() -> String {
  format!(r#"
  {{
                            ... on TariffType {{
                                id
                                displayName
                                fullName
                                description
                                productCode
                                standingCharge
                                preVatStandingCharge
                                tariffCode
 }}
                            ... on StandardTariff {{
                                unitRate
                                unitRateEpgApplied
                                preVatUnitRate
 }}
                            ... on DayNightTariff {{
                                dayRate

                                # Is EPG applied to the unit rate.
                                dayRateEpgApplied
                                nightRate

                                # Is EPG applied to the unit rate.
                                nightRateEpgApplied
                                preVatDayRate
                                preVatNightRate
 }}
                            ... on ThreeRateTariff {{
                                dayRate

                                # Is EPG applied to the unit rate.
                                dayRateEpgApplied
                                nightRate

                                # Is EPG applied to the unit rate.
                                nightRateEpgApplied
                                offPeakRate

                                # Is EPG applied to the unit rate.
                                offPeakRateEpgApplied
                                preVatDayRate
                                preVatNightRate
                                preVatOffPeakRate
 }}
                            __typename
 }}
 }}
"#)
}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StandardTariff {
    pub id: String,
    pub display_name: String,
    pub full_name: String,
    pub description: String,
    pub product_code: String,
    pub standing_charge: f64,
    pub pre_vat_standing_charge: Option<f64>,
  
    // Describes a particular tariff by combining the product code, number of rates, available from date and GSP code.
    pub tariff_code: String,
    pub unit_rate: f64,
  
    // Is EPG applied to the unit rate.
    pub unit_rate_epg_applied: Option<bool>,
    pub pre_vat_unit_rate: Option<f64>,
  }

  impl StandardTariff {


    // fn from(value: serde_json::Value) -> Result<Box<StandardTariffType>, Error> {
    //     println!("Got {:?}", value);
    //     println!("Got {:?}", value.is_object());
    //     println!("Got {:?}", value.as_str());
        
    //     if let Some(type_name_value) = value.get("__typename") {
    //     if type_name_value.is_string() {
    //         let type_name = type_name_value.as_str().unwrap();
    //         match(type_name) {
    //             "StandardTariff" => {
    //                 let tariff: StandardTariffType = serde_json::from_value(value)?;
    //                 Ok(Box::new(tariff))
    //             },
    //             _ => Err(Error::StringError(format!("Invalid Tariff type {}", type_name)))
    //         }
    //         }
    //         else {
    //             Err(Error::StringError(format!("No __typename attribute for Tariff")))
    //            }
    //     }
    //     else {
    //      Err(Error::StringError(format!("No __typename attribute for Tariff")))
    //     }
    //  }

//     pub fn get_field_names() -> String {
//         format!(r#"id
// id
// displayName
// fullName
// description
// productCode
// standingCharge
// preVatStandingCharge
// tariffCode
// unitRate
// unitRateEpgApplied
// preVatUnitRate
// "#)
//     }

    // pub async fn get_account_user(
    //     gql_client: &Arc<crate::gql::Client>,
    //     token_manager: &mut TokenManager,
    // ) -> Result<AccountUser, Error> {
    //     let operation_name = "getAccountUser";
    //     let query = format!(
    //         r#"query {}
    //                         {{
    //                             viewer
    //                             {{
    //                                 {}
    //                             }}
    //                         }}"#,
    //         operation_name, Self::get_field_names(AccountInterface::get_field_names())
    //     );

    //     println!("QUERY {}", query);

    //     let mut headers = HashMap::new();
    //     // let token = String::from(self.get_authenticator().await?);
    //     let token = &*token_manager.get_authenticator().await?;
    //     headers.insert("Authorization", token);

    //     let href = Some(&headers);

    //     let variables =  {};

    //     let mut response = gql_client
    //         .call(operation_name, &query, &variables, href)
    //         .await?;

    //     if let Some(result_json) = response.remove("viewer") {
    //         let account_user: AccountUser = serde_json::from_value(result_json)?;

    //         Ok(account_user)
    //     } else {
    //         return Err(Error::InternalError("No result found"));
    //     }
    // }
}

impl Display for Tariff {
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
pub struct DayNightTariff {
  id: String,
  display_name: String,
  full_name: String,
  description: String,
  product_code: String,
  standing_charge: f64,
  pre_vat_standing_charge: Option<f64>,
  tariff_code: String,
  day_rate: f64,
  day_rate_epg_applied: Option<bool>,
  night_rate: f64,
  night_rate_epg_applied: Option<bool>,
  pre_vat_day_rate: Option<f64>,
  pre_vat_night_rate: Option<f64>
}

impl DayNightTariff {
  pub fn get_field_names() -> &'static str {
    r#"
    id
    displayName
    fullName
    description
    productCode
    standingCharge
    preVatStandingCharge
    tariffCode
    dayRate
    dayRateEpgApplied
    nightRate
    nightRateEpgApplied
    preVatDayRate
    preVatNightRate
    "#
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThreeRateTariff {
  id: String,
  display_name: String,
  full_name: String,
  description: String,
  product_code: String,
  standing_charge: f64,
  pre_vat_standing_charge: Option<f64>,
  tariff_code: String,
  day_rate: f64,
  day_rate_epg_applied: Option<bool>,
  night_rate: f64,
  night_rate_epg_applied: Option<bool>,
  off_peak_rate: f64,
  off_peak_rate_epg_applied: Option<bool>,
  pre_vat_day_rate: Option<f64>,
  pre_vat_night_rate: Option<f64>,
  pre_vat_off_peak_rate: Option<f64>,
}

impl ThreeRateTariff {
  pub fn get_field_names() -> &'static str {
    r#"
    id
    displayName
    fullName
    description
    productCode
    standingCharge
    preVatStandingCharge
    tariffCode
    dayRate
    dayRateEpgApplied
    nightRate
    nightRateEpgApplied
    offPeakRate
    offPeakRateEpgApplied
    preVatDayRate
    preVatNightRate
    preVatOffPeakRate
    "#
  }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_tariff() {
        let json = r#"
        {
          "id": "242336",
          "displayName": "Octopus 12M Fixed",
          "fullName": "Octopus 12M Fixed August 2024 v3",
          "description": "This tariff features 100% renewable electricity and fixes your unit rates and standing charge for 12 months.",
          "productCode": "OE-FIX-12M-24-08-20",
          "standingCharge": 47.85,
          "preVatStandingCharge": null,
          "tariffCode": "E-1R-OE-FIX-12M-24-08-20-A",
          "unitRate": 24.15,
          "unitRateEpgApplied": false,
          "preVatUnitRate": null,
          "__typename": "StandardTariff"
        }
        "#;

        let tariff = Tariff::from(serde_json::from_str(json).unwrap()).unwrap();

        match tariff {
          Tariff::Standard(_) => {}
          _ => { 
            panic!("Expected StandardTariff but got {}", tariff);
          }
        }
    }

    #[test]
    fn test_day_night_tariff() {
        let json = r#"
        {
          "id": "6421",
          "displayName": "Octopus Key and Card",
          "fullName": "Octopus Key and Card",
          "description": "Non-smart prepayment tariff",
          "productCode": "PREPAY-VAR-18-09-21",
          "standingCharge": 48.4,
          "preVatStandingCharge": null,
          "tariffCode": "E-2R-PREPAY-VAR-18-09-21-A",
          "dayRate": 28.66,
          "dayRateEpgApplied": false,
          "nightRate": 10.34,
          "nightRateEpgApplied": false,
          "preVatDayRate": null,
          "preVatNightRate": null,
          "__typename": "DayNightTariff"
        }
        "#;
        
        let tariff = Tariff::from(serde_json::from_str(json).unwrap()).unwrap();

        match tariff {
          Tariff::DayNight(_) => {}
          _ => { 
            panic!("Expected DayNightTariff but got {}", tariff);
          }
        }
    }

    #[test]
    fn test_three_rate_tariff() {
        let json = r#"
        {
          "id": "177076",
          "displayName": "Cosy Octopus",
          "fullName": "Cosy Octopus",
          "description": "Cosy Octopus is a heat pump tariff with eight hours of super cheap electricity every day to warm your home.",
          "productCode": "COSY-22-12-08",
          "standingCharge": 47.85,
          "preVatStandingCharge": 45.57,
          "tariffCode": "E-1R-COSY-22-12-08-A",
          "dayRate": 33.9,
          "dayRateEpgApplied": null,
          "nightRate": 11.46,
          "nightRateEpgApplied": null,
          "offPeakRate": 23.38,
          "offPeakRateEpgApplied": null,
          "preVatDayRate": 32.29,
          "preVatNightRate": 10.91,
          "preVatOffPeakRate": 22.27,
          "__typename": "ThreeRateTariff"
        }
        "#;
        
        let tariff = Tariff::from(serde_json::from_str(json).unwrap()).unwrap();

        match tariff {
          Tariff::ThreeRate(_) => {}
          _ => { 
            panic!("Expected ThreeRateTariff but got {}", tariff);
          }
        }
    }

}

