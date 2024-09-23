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

use sparko_graphql_derive::{GraphQLQueryParams, GraphQLType};
use sparko_graphql::{types::{Boolean, Date, DateTime, Float, ForwardPageOf, Int, ID}, GraphQL, GraphQLQueryParams, GraphQLType, NoParams, ParamBuffer, VariableBuffer};
use crate::octopus::consumption_type::ConsumptionType;

use super::{consumption_type::ConsumptionTypeQueryParams, decimal::Decimal};

#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct MeterQueryParams {
  pub id: Option<Int>,
  pub include_inactive: Option<bool>,
  pub consumption: ConsumptionTypeQueryParams
}

#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct MeterIdView {
  pub id: ID,
  pub serial_number: String,
}

#[derive(GraphQLType)]
#[graphql(params = "MeterQueryParams")]
#[graphql(super_type = ["MeterInterface"])]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(tag = "__typename")]
pub enum Meter {
  ElectricityMeterType(ElectricityMeterType),
  GasMeterType(GasMeterType)
}

impl Meter {
  pub fn as_meter_interface(&self) -> &MeterInterface {
    match self {
      Meter::ElectricityMeterType(txn) => &txn.meter_interface,
      Meter::GasMeterType(txn) => &txn.meter_interface,
    }
  }
}

// interface Meter in the schema
#[derive(GraphQLType)]
#[graphql(params = "MeterQueryParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct MeterInterface {
  pub id: ID,
  pub serial_number: String,
  pub consumption_units: Option<String>,
  // Whether this meter requires a final change of tenancy (COT) reading.
  pub requires_cot_final_reading: Boolean,
  pub fuel_type: String,
  pub consumption: ForwardPageOf<ConsumptionType>
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


// #[derive(GraphQLType)]
// #[graphql(params = "MeterQueryParams")]
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct AbstractMeter{
//   #[serde(flatten)]
//   pub meter_interface: MeterInterface,
// }

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
  pub export_meters: ExportMetersQueryParams,
  pub readings: ReadingsQueryParams,
}


#[derive(GraphQLType)]
#[graphql(params = "MeterQueryParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ElectricityMeterType {
  #[serde(flatten)]
  pub meter_interface: MeterInterface,

  
  pub requires_access: Boolean,
  pub is_digital: Boolean,
  pub installation_date: Date,
  pub last_inspection_date: Date,
  pub created_at: DateTime,
  pub updated_at: Option<DateTime>,
  pub active_from: Date,
  pub active_to: Option<Date>,
  // meter_point: ElectricityMeterPointType,
  pub location: Option<String>,
  pub current_rating: Option<Int>,
  pub make_and_type: Option<String>,
  #[graphql(no_params)]
  #[graphql(scalar)]
  pub meter_type: Option<ElectricityMeterMeterType>,
  pub certification_date: Option<Date>,
  pub certified_until: Option<Date>,
  pub retrieval_method: Option<String>,
  #[graphql(no_params)]
  pub import_meter: Option<MeterIdView>,
  #[graphql(no_params)]
  #[graphql(scalar)]
  pub export_meters: ForwardPageOf<ElectricityMeterType>,
  // pub prepay_ledgers: Option<PrepayLedgersType>,
  // pub smart_import_electricity_meter: Option<SmartMeterDeviceType>,
  // pub smart_export_electricity_meter: Option<SmartMeterDeviceType>,
  pub node_id: ID,
  // pub readings: ForwardPageOf<ElectricityMeterReadingType>,
  #[graphql(no_params)]
  // pub registers: Vec<ElectricityMeterRegisterType>,
  pub has_and_allows_hh_readings: Boolean,
  // pub smart_devices: Vec<SmartMeterDeviceType>,
  pub is_trad_prepay: Boolean,
  pub is_ready_for_topup: Boolean,
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
pub enum ElectricityMeterMeterType {
  CHECK,
  // Half Hourly
  H,
  // Key
  K,
  #[serde(rename = "LAG_")]
  LAG,
  #[serde(rename = "LEAD_")]
  LEAD,
  #[serde(rename = "MAIN_")]
  MAIN,
  // Non-Half Hourly
  N,
  // Non-remotely Configurable Automated Meter Reading
  NCAMR,
  // A meter that meets the definition of an ADM but is not compliant with any version of SMETS
  NSS,
  // Remotely Configurable Automated Meter Reading without remote enable/disable capability
  RCAMR,
  // Remotely Configurable Automated Meter Reading with remote enable/disable capability
  RCAMY,
  // Smartcard Prepayment
  S,
  // A meter that is compliant with the Smart Metering Equipment Technical Specifications 1 (SMETS1)
  S1,
  // A single element meter that is compliant with SMETS2
  S2A,
  // A twin element meter that is compliant with SMETS2
  S2B,
  // A polyphase meter that is compliant with SMETS2
  S2C,
  // A single element meter with one or more ALCS that is compliant with SMETS2
  S2AD,
  // A twin element meter with one or more ALCS that is compliant with SMETS2
  S2BD,
  // A polyphase meter with one or more ALCS that is compliant with SMETS2
  S2CD,
  // Single element meter with one or more ALCS and Boost Function that is compliant with SMETS2
  S2ADE,
  // A twin element meter with one or more ALCS and Boost Function that is compliant with SMETS2
  S2BDE,
  // A polyphase meter with one or more ALCS and Boost Function that is compliant with SMETS2
  S2CDE,
  // Special
  SPECL,
  // Token
  T,
  // Single Element with APC that is compliant with SMETS2
  #[serde(rename = "A_2AF")]
  A2af,
  // Single Element with ALCS and APC that is compliant with SMETS2
  #[serde(rename = "A_2ADF")]
  A2adf,
  // Single Element with Boost Function and APC that is compliant with SMETS2
  #[serde(rename = "A_2AEF")]
  A2aef,
  // Single Element with ALCS, Boost Function and APC that is compliant with SMETS2
  #[serde(rename = "A_2ADEF")]
  A2adef,
  // Twin Element  with APC that is compliant with SMETS2
  #[serde(rename = "A_2BF")]
  A2bf,
  // Twin Element with ALCS and APC that is compliant with SMETS2
  #[serde(rename = "A_2BDF")]
  A2bdf,
  // Twin Element with Boost Function and APC that is compliant with SMETS2
  #[serde(rename = "A_2BEF")]
  A2bef,
  // Twin Element with ALCS, Boost Function and APC that is compliant with SMETS2
  #[serde(rename = "A_2BDEF")]
  A2bdef,
  // Polyphase with APC that is compliant with SMETS2
  #[serde(rename = "A_2CF")]
  A2cf,
  // Polyphase with ALCS and APC that is compliant with SMETS2
  #[serde(rename = "A_2CDF")]
  A2cdf,
  // Polyphase with Boost Function and APC that is compliant with SMETS2
  #[serde(rename = "A_2CEF")]
  A2cef,
  // Polyphase with ALCS, Boost Function and APC that is compliant with SMETS2
  #[serde(rename = "A_2CDEF")]
  A2cdef,
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
pub enum GasMeterMechanism {
  // Credit
  CR,
  // Electronic Token Meter
  ET,
  // Prepayment
  PP,
  // Mechanical Token Meter
  MT,
  // Coin Meter
  CM,
  // Thrift
  TH,
  // Non Compliant SMETS Smart Meter
  NS,
  // SMETS 1 compliant Smart Meter
  S1,
  // SMETS 2 compliant Smart Meter
  S2,
  // Unknown
  U
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
pub enum GasMeterStatus {
  // Live
  LI,
  // Faulty
  FA,
  // Inactive
  IN,
  // Cut off
  CU,
  // Clamped
  CL,
  // Capped
  CA,
  // Spin Cap
  SP,
  // Removed
  RE,
  // Other
  OT,
  // Unknown
  UN,
  // Not Installed
  NI
}

// A gas meter has a register which holds readings. We would expect this to be a one-to-one relationship between meter and register.
// implements Node & Meter
#[derive(GraphQLType)]
#[graphql(params = "MeterQueryParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct GasMeterType  {
  #[serde(flatten)]
  pub meter_interface: MeterInterface,

  pub requires_access: Boolean,
  pub is_digital: Boolean,
  pub installation_date: Date,
  pub last_inspection_date: Option<Date>,
  pub created_at: DateTime,
  pub updated_at: Option<DateTime>,
  pub active_from: Date,
  pub active_to: Option<Date>,
  // meterPoint: GasMeterPointType,
  pub manufacturer_code: String,
  pub model_name: String,
  pub manufactured_year: Option<Int>,
  pub meter_type: String,
  pub imperial: Boolean,
  pub units: Int,
  pub location: String,
  #[graphql(no_params)]
  #[graphql(scalar)]
  pub mechanism: GasMeterMechanism,
  pub correction: Float,
  pub location_description: String,
  pub reading_factor: Decimal,
  pub instructions: String,
  pub pulse_value: Decimal,
  pub link_code: String,
  pub collar_fitted: String,
  pub bypass_fitted: String,
  pub measuring_capacity: Option<Decimal>,
  #[graphql(no_params)]
  #[graphql(scalar)]
  pub status: GasMeterStatus,
  pub operational_status_date: Date,
  pub owner: String,
  pub current_meter_asset_manager: String,
  // pub prepay_ledgers: PrepayLedgersType,
  // pub smart_gas_meter: SmartMeterDeviceType,

  

  // This lets us get around the fact that we already use the field id as a primary key. We will migrate the id field over to be this id eventually.
  pub node_id: ID,
  // readings(
  //   before: String
  //   after: String
  //   first: Int
  //   last: Int
  // )
  // pub readings: ForwardPageOf<GasMeterReading>,

  // #[graphql(no_params)]
  // pub registers: Vec<GasMeterRegisterType>,

  // Returns if the meter has and allows half hourly readings
  pub has_and_allows_hh_readings: Boolean,

  // #[graphql(no_params)]
  // pub smart_devices: Vec<SmartMeterDeviceType>,
  pub is_trad_prepay: Boolean,
  pub is_ready_for_topup: Boolean
}