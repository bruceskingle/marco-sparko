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

use crate::{gql::{types::{Boolean, Date, Int, ID}, GraphQLType}, octopus::consumption_type::ConsumptionType};

use super::consumption_type::{ConsumptionConnection, ConsumptionQueryParams};




#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(tag = "__typename")]
pub enum Meter {
  ElectricityMeterType(ElectricityMeterType),
  GasMeterType(AbstractMeter)
}

impl GraphQLType<()> for Meter {
  fn get_query(params: ()) -> String {
        
    format!(r#"
    {}
    ... on ElectricityMeterType {{
      {}
    }}
    "#, MeterInterface::get_field_names(),
        ElectricityMeterType::get_field_names())
    }
}

impl Meter {
  pub fn as_meter_point_interface(&self) -> &MeterInterface {
    match self {
      Meter::ElectricityMeterType(txn) => &txn.meter_point_interface,
      Meter::GasMeterType(txn) => &txn.meter_point_interface,
    }
  }
}



#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct MeterInterfaceQueryParams {
  consumption: ConsumptionQueryParams
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct MeterInterface {
  pub id: ID,
  pub serial_number: String,
  pub consumption_units: Option<String>,
  // Whether this meter requires a final change of tenancy (COT) reading.
  pub requires_cot_final_reading: Boolean,
  pub fuel_type: String,
  pub consumption: ConsumptionConnection
}

/*
consumption(
    # Earliest consumption reading to return. Must specify a timezone.
    startAt: DateTime!

    # Aggregate consumption according to this grouping.
    grouping: ConsumptionGroupings!

    # Timezone to use for grouping.
    timezone: String!
    before: String
    after: String
    first: Int
    last: Int
  )
*/

// This is an interface in the GraphQL schema
impl GraphQLType<MeterInterfaceQueryParams> for  MeterInterface {
  fn get_query(params: MeterInterfaceQueryParams) -> String {
    format!(r#"
    id
    serialNumber
    consumptionUnits
    requiresCotFinalReading
    fuelType
    consumption({}) {{
      {}
    }}
    "#, params.consumption, ConsumptionType::get_query(()))
    }
}

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(tag = "__typename")]
// pub enum  ElectricityMeterPointType {
//   StandardMeterPoint(StandardMeterPoint),
//   DayNightMeterPoint(DayNightMeterPoint),
//   ThreeRateMeterPoint(ThreeRateMeterPoint),
//   HalfHourlyMeterPoint(HalfHourlyMeterPoint),
//   PrepayMeterPoint(PrepayMeterPoint)
// }

// impl ElectricityMeterPointType {
//   pub fn get_field_names() -> String {
//     format!(r#"
//     {{
//       ... on MeterPointType {{
//           id
//           displayName
//           fullName
//           description
//           productCode
//           standingCharge
//           preVatStandingCharge
//           tariffCode
//       }}
//       ... on StandardMeterPoint {{
//           unitRate
//           unitRateEpgApplied
//           preVatUnitRate
//       }}
//       ... on DayNightMeterPoint {{
//           dayRate
  
//           # Is EPG applied to the unit rate.
//           dayRateEpgApplied
//           nightRate
  
//           # Is EPG applied to the unit rate.
//           nightRateEpgApplied
//           preVatDayRate
//           preVatNightRate
//       }}
//       ... on ThreeRateMeterPoint {{
//           dayRate
  
//           # Is EPG applied to the unit rate.
//           dayRateEpgApplied
//           nightRate
  
//           # Is EPG applied to the unit rate.
//           nightRateEpgApplied
//           offPeakRate
  
//           # Is EPG applied to the unit rate.
//           offPeakRateEpgApplied
//           preVatDayRate
//           preVatNightRate
//           preVatOffPeakRate
//       }}
//       __typename
//     }}
//   "#)
//   }
// }

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(tag = "__typename")]
// pub enum EnergyMeterPointType {
//   StandardMeterPoint(StandardMeterPoint),
//   DayNightMeterPoint(DayNightMeterPoint),
//   ThreeRateMeterPoint(ThreeRateMeterPoint),
//   GasMeterPointType(GasMeterPointType)
// }

// impl EnergyMeterPointType {

//  /*
//  All field names

//  {
//       ... on MeterPointType {
//           id
//           displayName
//           fullName
//           description
//           productCode
//           standingCharge
//           preVatStandingCharge
//           tariffCode
//       }
//       ... on StandardMeterPoint {
//           unitRate
//           unitRateEpgApplied
//           preVatUnitRate
//       }
//       ... on DayNightMeterPoint {
//           dayRate

//           # Is EPG applied to the unit rate.
//           dayRateEpgApplied
//           nightRate

//           # Is EPG applied to the unit rate.
//           nightRateEpgApplied
//           preVatDayRate
//           preVatNightRate
//       }
//       ... on ThreeRateMeterPoint {
//           dayRate

//           # Is EPG applied to the unit rate.
//           dayRateEpgApplied
//           nightRate

//           # Is EPG applied to the unit rate.
//           nightRateEpgApplied
//           offPeakRate

//           # Is EPG applied to the unit rate.
//           offPeakRateEpgApplied
//           preVatDayRate
//           preVatNightRate
//           preVatOffPeakRate
//       }
//       ... on HalfHourlyMeterPoint {
//       unitRates {
//           validFrom
//           validTo

//           # Price in pence (inc VAT).
//           value

//           # Price in pence (not including VAT).
//           preVatValue
//               # Information on how agile unit rates have been calculated.
//           agileCalculationInfo  {
//               # The maximum value/cap for the unit rate.
//               priceCap

//               # The peak offset for the unit rate.
//               peakOffset

//               # The price multiplier/coefficient used to calculate the unit rate.
//               gspCoefficient
//           }
//       }

      
//       ... on PrepayMeterPoint {
//           unitRate
//           preVatUnitRate
//       }
//       __typename
//       }
//   }
 
//   */

//  pub fn get_field_names() -> String {
//   format!(r#"
//   {{
//     ... on MeterPointType {{
//         id
//         displayName
//         fullName
//         description
//         productCode
//         standingCharge
//         preVatStandingCharge
//         tariffCode
//     }}
//     ... on StandardMeterPoint {{
//         unitRate
//         unitRateEpgApplied
//         preVatUnitRate
//     }}
//     ... on DayNightMeterPoint {{
//         dayRate

//         # Is EPG applied to the unit rate.
//         dayRateEpgApplied
//         nightRate

//         # Is EPG applied to the unit rate.
//         nightRateEpgApplied
//         preVatDayRate
//         preVatNightRate
//     }}
//     ... on ThreeRateMeterPoint {{
//         dayRate

//         # Is EPG applied to the unit rate.
//         dayRateEpgApplied
//         nightRate

//         # Is EPG applied to the unit rate.
//         nightRateEpgApplied
//         offPeakRate

//         # Is EPG applied to the unit rate.
//         offPeakRateEpgApplied
//         preVatDayRate
//         preVatNightRate
//         preVatOffPeakRate
//     }}
//     __typename
//   }}
// "#)
// }
// }



// // Details of an ongoing enrolment process.
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct EnrolmentType {
//   // Date the switch started.
//   switch_start_date: Date,

//   // Target date for supply to start.
//   supply_start_date: Date,

//   // The last company to supply this meter point.
//   previous_supplier: String,

//   // The enrolment status on a meter point.
//   status: EnrolmentStatusOptions
// }

// impl GraphQLType for EnrolmentType {
//   fn get_field_names() -> String {
//     format!(r#"
//     switchStartDate
//     supplyStartDate
//     previousSupplier
//     "#)
//   }
// }


// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// pub enum EnrolmentStatusOptions {
//   // The previous supplier objects to the switch. The have not has cancelled the switch yet, but in 99% cases, they will cancel the switch.
//   ObjectionReceived,
//   // The previous supplier cancelled the switch. This is a terminal state, and we will have to reapply before this can start again.
//   RegistrationObjected,
//   // The overseeing industry body has objected to the switch
//   Rejected,
//   // The request to bring the meter point on supply has been withdrawn.
//   Withdrawn,
//   // The meterpoint has been created but the enrolment process has not started yet.
//   PreRegistration,
//   // Enrolment has been requested. This is the default catch-all status, which is returned when no other defined process is happening.
//   Requested,
//   // Enrolment has been completed.
//   Completed,
//   // Enrolment has been disputed. This could be that the meter point details that have been provided have been disputed.
//   Disputed,
//   // Enrolment has been accepted by the industry, which means that it has all the information needed to switch supplier and if that information is correct (to it's knowledge)
//   Accepted
// }

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct AbstractMeter{
  #[serde(flatten)]
  pub meter_point_interface: MeterInterface,
}

/*
  An electricity meter is a collection of registers which store readings.
  Eco7 meters are an example of a meter with multiple registers (for day and night).

  implements Node & Meter

  consumption(
    # Earliest consumption reading to return. Must specify a timezone.
    startAt: DateTime!

    # Aggregate consumption according to this grouping.
    grouping: ConsumptionGroupings!

    # Timezone to use for grouping.
    timezone: String!
    before: String
    after: String
    first: Int
    last: Int
  ): ConsumptionConnection
*/



#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportMetersQueryParams {
  pub offset: Int,
  pub before: String,
  pub after: String,
  pub first: Int,
  pub last: Int,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReadingsQueryParams {
  pub before: String,
  pub after: String,
  pub first: Int,
  pub last: Int,
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ElectricityMeterTypeQueryParams {
  pub exportMeters: ExportMetersQueryParams,
  pub readings: ReadingsQueryParams,
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ElectricityMeterType {
  #[serde(flatten)]
  pub meter_point_interface: MeterInterface,

  
  requiresAccess: Boolean
  isDigital: Boolean
  installationDate: Date
  lastInspectionDate: Date
  createdAt: DateTime!
  updatedAt: DateTime
  activeFrom: Date!
  activeTo: Date
  meterPoint: ElectricityMeterPointType!
  location: String
  currentRating: Int
  makeAndType: String
  meterType: ElectricityMeterMeterType
  certificationDate: Date
  certifiedUntil: Date
  retrievalMethod: String
  importMeter: ElectricityMeterType
  exportMeters: ElectricityMeterTypeConnection!
  prepayLedgers: PrepayLedgersType
  smartImportElectricityMeter: SmartMeterDeviceType
  smartExportElectricityMeter: SmartMeterDeviceType
  nodeId: ID!
  readings: ElectricityMeterReadingConnectionTypeConnection
  registers: [ElectricityMeterRegisterType]
  hasAndAllowsHhReadings: Boolean
  smartDevices: [SmartMeterDeviceType]
  isTradPrepay: Boolean
  isReadyForTopup: Boolean
}

impl ElectricityMeterType {
  pub fn get_query() -> String {
    format!(r#"
    {}
    requiresAccess
    isDigital
    installationDate
    lastInspectionDate
    createdAt
    updatedAt
    activeFrom
    activeTo
    meterPoint
    location
    currentRating
    makeAndType
    meterType
    certificationDate
    certifiedUntil
    retrievalMethod
    importMeter
    exportMeters
    prepayLedgers
    smartImportElectricityMeter
    smartExportElectricityMeter
    nodeId
    readings
    registers: [ElectricityMeterRegisterType]
    hasAndAllowsHhReadings
    smartDevices: [SmartMeterDeviceType]
    isTradPrepay
    isReadyForTopup
    "#,
    
  )
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
          "__typename": "StandardMeter"
        }
        "#;
        let tariff: ElectricityMeterType = serde_json::from_str(json).unwrap();
        // let tariff = Meter::from(serde_json::from_str(json).unwrap()).unwrap();

        match tariff {
          ElectricityMeterType::StandardMeter(_) => {}
          _ => { 
            panic!("Expected StandardMeter but got {}", tariff);
          }
        }
    }

}