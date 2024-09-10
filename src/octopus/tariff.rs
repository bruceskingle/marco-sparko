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

use sparko_graphql::types::{Boolean, DateTime, Float, ID};

/*
union ElectricityTariffType =
    StandardTariff
  | DayNightTariff
  | ThreeRateTariff
  | HalfHourlyTariff
  | PrepayTariff

dd












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


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(tag = "__typename")]
pub enum  ElectricityTariffType {
  StandardTariff(StandardTariff),
  DayNightTariff(DayNightTariff),
  ThreeRateTariff(ThreeRateTariff),
  HalfHourlyTariff(HalfHourlyTariff),
  PrepayTariff(PrepayTariff)
}

impl ElectricityTariffType {
  pub fn get_field_names() -> String {
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
  "#)
  }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(tag = "__typename")]
pub enum EnergyTariffType {
  StandardTariff(StandardTariff),
  DayNightTariff(DayNightTariff),
  ThreeRateTariff(ThreeRateTariff),
  GasTariffType(GasTariffType)
}

impl EnergyTariffType {

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

 pub fn get_field_names() -> String {
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
"#)
}
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct TariffType {
  pub id: String,
  pub display_name: String,
  pub full_name: String,
  pub description: String,
  pub product_code: String,
  pub standing_charge: f64,
  pub pre_vat_standing_charge: Option<f64>,

  // Describes a particular tariff by combining the product code, number of rates, available from date and GSP code.
  pub tariff_code: String,
}

// This is an interface in the GraphQL schema
impl TariffType {
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
    "#
  }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AbstractTariff{
  #[serde(flatten)]
  pub tariff: TariffType,
}

impl AbstractTariff {
  pub fn get_field_names() -> String {
    format!(r#"
        {}
        ... on StandardTariff {{
          {}
        }}
        ... on DayNightTariff {{
          {}
        }}
        ... on ThreeRateTariff {{
          {}
        }}
        ... on HalfHourlyTariff {{
          {}
        }}
        ... on GasTariff {{
          {}
        }}
    "#, TariffType::get_field_names(),
        StandardTariff::get_field_names(),
        DayNightTariff::get_field_names(),
        ThreeRateTariff::get_field_names(),
        HalfHourlyTariff::get_field_names(),
        GasTariffType::get_field_names()
      )
  }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct StandardTariff {
  #[serde(flatten)]
  pub tariff: TariffType,
  pub unit_rate: f64,

  // Is EPG applied to the unit rate.
  pub unit_rate_epg_applied: Option<bool>,
  pub pre_vat_unit_rate: Option<f64>,
}

impl StandardTariff {
  pub fn get_field_names() -> &'static str {
    r#"
    unitRate
    unitRateEpgApplied
    preVatUnitRate
    "#
  }
  // pub async fn get_account_user(
  //     gql_client: &Arc<sparko_graphql::Client>,
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

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct DayNightTariff {
  #[serde(flatten)]
  pub tariff: TariffType,
  pub day_rate: f64,
  pub day_rate_epg_applied: Option<bool>,
  pub night_rate: f64,
  pub night_rate_epg_applied: Option<bool>,
  pub pre_vat_day_rate: Option<f64>,
  pub pre_vat_night_rate: Option<f64>
}

impl DayNightTariff {
  pub fn get_field_names() -> &'static str {
    r#"
    dayRate
    dayRateEpgApplied
    nightRate
    nightRateEpgApplied
    preVatDayRate
    preVatNightRate
    "#
  }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ThreeRateTariff {
  #[serde(flatten)]
  pub tariff: TariffType,
  pub day_rate: f64,
  pub day_rate_epg_applied: Option<bool>,
  pub night_rate: f64,
  pub night_rate_epg_applied: Option<bool>,
  pub off_peak_rate: f64,
  pub off_peak_rate_epg_applied: Option<bool>,
  pub pre_vat_day_rate: Option<f64>,
  pub pre_vat_night_rate: Option<f64>,
  pub pre_vat_off_peak_rate: Option<f64>,
}

impl ThreeRateTariff {
  pub fn get_field_names() -> &'static str {
    r#"
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

/*
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
*/



#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct HalfHourlyTariff {
  #[serde(flatten)]
  pub tariff: TariffType,
  pub unit_rates: Vec<UnitRate>,

  // Information on how agile unit rates have been calculated.
  pub agile_calculation_info: Option<AgileCalculationInfo>
}

impl HalfHourlyTariff {
  pub fn get_field_names() -> &'static str {
    r#"
    unitRates: [UnitRate]
    agileCalculationInfo: AgileCalculationInfo
    "#
  }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct PrepayTariff {
  #[serde(flatten)]
  pub tariff: TariffType,
  pub unit_rate: Float,
  pub pre_vat_unit_rate: Float
}

impl PrepayTariff {
  pub fn get_field_names() -> &'static str {
    r#"
    unitRate
    preVatUnitRate
    "#
  }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct GasTariffType {
  #[serde(flatten)]
    pub tariff: TariffType,
    pub unit_rate: Float,
  
    // Is EPG applied to the unit rate.
    pub unit_rate_epg_applied: Option<Boolean>,
    pub pre_vat_unit_rate: Float
}

impl GasTariffType {
  pub fn get_field_names() -> &'static str {
    r#"
    unitRate
    unitRateEpgApplied
    preVatUnitRate
    "#
  }
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct UnitRate {
  pub valid_from: DateTime,
  pub valid_to: DateTime,
  // Price in pence (inc VAT).
  pub value: Float,
  // Price in pence (not including VAT).
  pub pre_vat_value: Float
}

impl UnitRate {
  pub fn get_field_names() -> &'static str {
    r#"
    validFrom
    validTo
    value
    preVatValue
    "#
  }
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AgileCalculationInfo {
  // The maximum value/cap for the unit rate.
  pub price_cap: Float,

  // The peak offset for the unit rate.
  pub peak_offset: Float,

  // The price multiplier/coefficient used to calculate the unit rate.
  pub gsp_coefficient: Float
}

impl AgileCalculationInfo {
  pub fn get_field_names() -> &'static str {
    r#"
    priceCap
    peakOffset
    gspCoefficient
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
        let tariff: ElectricityTariffType = serde_json::from_str(json).unwrap();
        // let tariff = Tariff::from(serde_json::from_str(json).unwrap()).unwrap();

        match tariff {
          ElectricityTariffType::StandardTariff(_) => {}
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
        
        let tariff: ElectricityTariffType = serde_json::from_str(json).unwrap();
        // let tariff = Tariff::from(serde_json::from_str(json).unwrap()).unwrap();

        match tariff {
          ElectricityTariffType::DayNightTariff(_) => {}
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
        
        let tariff: ElectricityTariffType = serde_json::from_str(json).unwrap();
        // let tariff = Tariff::from(serde_json::from_str(json).unwrap()).unwrap();

        match tariff {
          ElectricityTariffType::ThreeRateTariff(_) => {}
          _ => { 
            panic!("Expected ThreeRateTariff but got {}", tariff);
          }
        }
    }

    #[test]
    fn half_hourly_tariff() {
        let json = r#"
        {
          "id": "175911",
          "displayName": "Intelligent Octopus Go",
          "fullName": "Intelligent Octopus Go",
          "description": "With Intelligent Octopus Go EV tariff, you have access to a super low electricity rate between 23:30 - 05:30 every night, plus it smart-charges your car at the cheapest and greenest times overnight.",
          "productCode": "INTELLI-VAR-22-10-14",
          "standingCharge": 47.8485,
          "preVatStandingCharge": 45.57,
          "tariffCode": "E-1R-INTELLI-VAR-22-10-14-A",
          "unitRates": [
            {
              "validFrom": "2024-08-29T22:30:00+00:00",
              "validTo": "2024-08-30T04:30:00+00:00",
              "value": 7.00035,
              "preVatValue": 6.667
            },
            {
              "validFrom": "2024-08-30T04:30:00+00:00",
              "validTo": "2024-08-30T22:30:00+00:00",
              "value": 24.39255,
              "preVatValue": 23.231
            },
            {
              "validFrom": "2024-08-30T22:30:00+00:00",
              "validTo": "2024-08-31T04:30:00+00:00",
              "value": 7.00035,
              "preVatValue": 6.667
            },
            {
              "validFrom": "2024-08-31T04:30:00+00:00",
              "validTo": "2024-08-31T22:30:00+00:00",
              "value": 24.39255,
              "preVatValue": 23.231
            },
            {
              "validFrom": "2024-08-31T22:30:00+00:00",
              "validTo": "2024-09-01T04:30:00+00:00",
              "value": 7.00035,
              "preVatValue": 6.667
            }
          ],
          "agileCalculationInfo": null,
          "__typename": "HalfHourlyTariff"
        }
        "#;
        
        let tariff: ElectricityTariffType = serde_json::from_str(json).unwrap();
        // let tariff = Tariff::from(serde_json::from_str(json).unwrap()).unwrap();

        match tariff {
          ElectricityTariffType::HalfHourlyTariff(_) => {}
          _ => { 
            panic!("Expected HalfHourlyTariff but got {}", tariff);
          }
        }
    }

    #[test]
    fn test_gas_tariff() {
        let json = r#"
        {
          "id": "49383",
          "displayName": "Flexible Octopus",
          "fullName": "Flexible Octopus",
          "description": "Flexible Octopus prices follow wholesale costs and update every 3 months.\r\n\r\nGood to know: Ofgem has announced the energy price cap will rise from October 1. Flexible Octopus rates will also rise in October – we'll share details soon.",
          "productCode": "VAR-22-11-01",
          "standingCharge": 28.9485,
          "preVatStandingCharge": 27.57,
          "tariffCode": "G-1R-VAR-22-11-01-A",
          "unitRate": 5.401725,
          "unitRateEpgApplied": null,
          "preVatUnitRate": 5.1445,
          "__typename": "GasTariffType"
        }
        "#;
        
        let tariff: EnergyTariffType = serde_json::from_str(json).unwrap();
        // let tariff = Tariff::from(serde_json::from_str(json).unwrap()).unwrap();

        match tariff {
          EnergyTariffType::GasTariffType(_) => {}
          _ => { 
            panic!("Expected GasTariffType but got {}", tariff);
          }
        }
    }

}

