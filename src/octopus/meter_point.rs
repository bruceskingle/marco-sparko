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

// use sparko_graphql_derive::{GraphQLQueryParams, GraphQLType};
use sparko_graphql::{types::{Boolean, Date, DateTime, ForwardPageOf, Int, ID}, GraphQL, GraphQLQueryParams, GraphQLType, NoParams, ParamBuffer, VariableBuffer};
use crate::octopus::consumption_type::ConsumptionType;

use super::{consumption_type::ConsumptionTypeQueryParams, decimal::Decimal, meter::{Meter, MeterQueryParams}};




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

#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct MeterPointQueryParams {
  pub meters: MeterQueryParams
}

#[derive(GraphQLType)]
#[graphql(params = "MeterPointQueryParams")]
#[graphql(super_type = ["MeterPointInterface"])]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(tag = "__typename")]
pub enum MeterPoint {
  ElectricityMeterPointType(ElectricityMeterPointType),
  GasMeterPointType(GasMeterPointType)
}

impl MeterPoint {
  pub fn as_meter_point_interface(&self) -> &MeterPointInterface {
    match self {
      MeterPoint::ElectricityMeterPointType(txn) => &txn.meter_point_interface,
      MeterPoint::GasMeterPointType(txn) => &txn.meter_point_interface,
    }
  }
}

#[derive(GraphQLType)]
#[graphql(params = "MeterPointQueryParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct MeterPointInterface {
  pub status: String,
  pub meters: Vec<Meter>,
  // Details of an ongoing enrolment process.
  #[graphql(no_params)]
  pub enrolment: Option<EnrolmentType>
}

// Details of an ongoing enrolment process.
#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct EnrolmentType {
  // Date the switch started.
  switch_start_date: Date,

  // Target date for supply to start.
  supply_start_date: Date,

  // The last company to supply this meter point.
  previous_supplier: String,

  // The enrolment status on a meter point.
  #[graphql(no_params)]
  #[graphql(scalar)]
  status: EnrolmentStatusOptions
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnrolmentStatusOptions {
  // The previous supplier objects to the switch. The have not has cancelled the switch yet, but in 99% cases, they will cancel the switch.
  ObjectionReceived,
  // The previous supplier cancelled the switch. This is a terminal state, and we will have to reapply before this can start again.
  RegistrationObjected,
  // The overseeing industry body has objected to the switch
  Rejected,
  // The request to bring the meter point on supply has been withdrawn.
  Withdrawn,
  // The meterpoint has been created but the enrolment process has not started yet.
  PreRegistration,
  // Enrolment has been requested. This is the default catch-all status, which is returned when no other defined process is happening.
  Requested,
  // Enrolment has been completed.
  Completed,
  // Enrolment has been disputed. This could be that the meter point details that have been provided have been disputed.
  Disputed,
  // Enrolment has been accepted by the industry, which means that it has all the information needed to switch supplier and if that information is correct (to it's knowledge)
  Accepted
}

#[derive(GraphQLType)]
#[graphql(params = "MeterPointQueryParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct GasMeterPointType{
  #[serde(flatten)]
  pub meter_point_interface: MeterPointInterface,

  pub id: ID,
  pub supply_end_date: Date,
  pub mprn: String,
  pub status_updated_at: Option<DateTime>,
  pub old_supplier_id: Option<String>,
  pub new_supplier_id: Option<String>,
  pub smart_start_date: Option<Date>,
  pub requires_enrolment: Boolean,
  pub target_ssd: Option<Date>,
  pub requires_withdrawal: Boolean,
  pub has_open_opening_read_dispute: Boolean,
  pub has_open_closing_read_dispute: Boolean,

  #[graphql(no_params)]
  #[graphql(scalar)]
  pub market_sector_code: Option<GasMeterPointMarketSectorCode>,
  #[graphql(no_params)]
  #[graphql(scalar)]
  pub market_category: Option<GasMeterPointMarketCategory>,
  #[graphql(no_params)]
  #[graphql(scalar)]
  pub meter_ownership_type: Option<GasMeterPointMeterOwnershipType>,
  pub confirmation_reference: Option<Int>,
  pub nomination_type: String,
  pub supply_class: Int,
  pub nomination_shipper_reference: String,

  // Industry status code
  pub xserver_status: Option<String>,
  pub exit_capacity_charge_rate: Option<Decimal>,
  pub ldz_capacity_charge_rate: Option<Decimal>,
  pub ldz_commodity_charge_rate: Option<Decimal>,
  pub ldz_customer_charge_rate: Option<Decimal>,
  pub nts_exit_commodity_charge_rate: Option<Decimal>,
  pub mrf_type: String,
  pub meter_read_batch_frequency: String,

  // SOQ fixed for year
  pub formula_year_smp_soq: Option<Int>,

  // AQ fixed for year
  pub formula_year_smp_aq: Option<Int>,

  // Rolling SOQ
  pub current_dm_soq: Option<Int>,

  // Rolling SOQ
  pub current_ndm_soq: Option<Int>,
  pub exit_zone: String,

  // Local distribution zone - Distribution charges are based upon this
  pub ldz: String,
  pub supply_point_category: String,
  pub end_user_category: Option<Int>,
  pub euc_identifier: Option<String>,
  pub igt_identifier: String,
  pub igt_checked_at: Option<DateTime>,
  // pub meters(id: Int, includeInactive: Boolean): [GasMeterType]
  pub status: Option<String>,

  // Details of an ongoing enrolment process.
  #[graphql(no_params)]
  pub enrolment: Option<EnrolmentType>,

  // A list of agents responsible for management of the meterpoint.
  // pub agentContracts(
  //   // Filter the contracts by status.
  //   statuses: [AgentContractStatusType]
  // ): [GasAgentContractType]

  // A list of gas agreements belonging to an account that is linked to the viewer. Filters out expired agreements by default.
  // pub agreements(
  //   validAfter: Option<DateTime>,
  //   includeInactive: Option<Boolean>,

  //   // Exclude agreements starting in the future.
  //   excludeFuture: Option<Boolean>,
  // ): [GasAgreementType]

  // A list of unbilled gas readings for the meterpoint.
  // pub unbilledReadings: [GasMeterReadingType]

  // The current MPID for this meter point.
  pub current_supplier_mpid: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
pub enum GasMeterPointMarketSectorCode {
  // Domestic
  D,
  // Industrial
  I
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
pub enum GasMeterPointMarketCategory {
  // SSP
  SSP,
  // LSP
  LSP
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
pub enum GasMeterPointMeterOwnershipType {
  // Transporter
  T,
  // Supplier
  S,
  // Customer
  C
}

#[derive(GraphQLType)]
#[graphql(params = "MeterPointQueryParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
// An electricity meterpoint is a collection of meters. Meters can be changed over time, so it is convenient to keep an invariant reference. Sometimes there are multiple active meters on a meterpoint at a time (eg ECO10), but expect that to be an edge case.
pub struct ElectricityMeterPointType {

  #[serde(flatten)]
  pub meter_point_interface: MeterPointInterface,

  pub id: ID,
  pub supply_end_date: Date,
  pub mpan: String,

  // Standard settlement configuration
  pub ssc: String,
  pub enervation_status: String,
  pub dcc_service_flag: String,
  pub status_updated_at: Option<DateTime>,
  pub old_supplier_id: Option<String>,
  pub new_supplier_id: Option<String>,
  pub smart_start_date: Option<Date>,
  pub requires_enrolment: Boolean,
  pub target_ssd: Option<Date>,
  pub requires_withdrawal: Boolean,
  pub has_open_opening_read_dispute: Boolean,
  pub has_open_closing_read_dispute: Boolean,

  // The profile class of the electricity meter point.
  pub profile_class: Option<Int>,

  // Line loss factor class
  pub llf: Option<String>,

  // Meter timeswitch code
  pub mtc: Option<Int>,
  pub measurement_class: String,
  pub last_validated_reading_date: Option<Date>,

  // Smart Metering System Operator
  pub sms_operator: String,
  pub sms_operator_effective_from: Option<Date>,
  pub ihd_status: String,
  pub ihd_effective_from: Option<Date>,
  pub dcc_effective_from: Option<Date>,

  // Details of an ongoing enrolment process.
  #[graphql(no_params)]
  pub enrolment: Option<EnrolmentType>,

  // The distribution network the grid supply point falls under
  pub gsp_group_id: Option<String>,

  // A list of agents responsible for management of the meterpoint.
  // agentContracts(
  //   // Filter the contracts by status.
  //   statuses: [AgentContractStatusType]
  // ): [ElectricityAgentContractType]

  // // A list of electricity agreements belonging to an account that is linked to the viewer. Filters out expired agreements by default.
  // agreements(
  //   validAfter: Option<DateTime>,
  //   includeInactive: Option<Boolean>,

  //   // Exclude agreements starting in the future.
  //   excludeFuture: Option<Boolean>,
  // ): [ElectricityAgreementType]
  #[graphql(no_params)]
  pub smart_tariff_onboarding: Option<SmartTariffOnboardingType>,

  // A list of unbilled electricity readings for the meterpoint.
  // pub unbilled_readings: Vec<ElectricityMeterReadingType>,

  // The current MPID for this meter point.
  pub current_supplier_mpid: Option<String>,
}


// The smart tariff onboarding process. Only relevant for Kraken instances that support half hourly tariffs. Returns null if not applicable.
#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct SmartTariffOnboardingType {
  pub id: ID,
  #[graphql(no_params)]
  #[graphql(scalar)]
  pub latest_status: Option<SmartOnboardingEventType>,
  #[graphql(no_params)]
  #[graphql(scalar)]
  pub latest_terms_status: Option<SmartOnboardingTermsStatuses>,
  #[graphql(no_params)]
  #[graphql(scalar)]
  pub smart_tariff_code: Option<SmartOnboardingTariffCodes>,
  pub last_updated: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
pub enum SmartOnboardingEventType {
  STARTED,
  COMPLETED,
  CANCELLED,
  NOTE_ADDED,
  METER_EXCHANGE_EMAIL_SENT,
  METER_EXCHANGE_BOOKED,
  METER_INSTALLED,
  METER_READINGS_AVAILABLE,
  TERMS_EMAIL_SENT,
  TERMS_ACCEPTED,
  AGREEMENTS_UPDATED,
  TARIFF_SWITCH_CONFIRMATION_EMAIL_SENT,
  TARIFF_CHANGED_ON_METER,
  UNABLE_TO_PROCEED,
  PREVIOUS_AGREEMENT_BILLING_GAP_FILLED,
  INTELLIGENT_OCTOPUS_INSTALL_APP_EMAIL_SENT,
  INTELLIGENT_OCTOPUS_TEST_DISPATCH_COMPLETE,
  DOCUMENTS_CHECKED,
  FIT_RESOLUTION,
  EXPORT_MPAN_APPLIED_FOR,
  EXPORT_MPAN_CREATED,
  EXPORT_MPAN_NOT_FOUND,
  MTD_UPDATED,
  EXPORT_ENABLED_IN_KRAKEN,
  EXPORT_METER_READING_AVAILABLE,
  EXPORT_MPAN_ON_SUPPLY,
  FIRST_CREDIT_APPLIED
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
pub enum SmartOnboardingTermsStatuses {
  TERMS_ACCEPTANCE_REQUIRED,
  TERMS_EMAIL_SENT,
  TERMS_ACCEPTED
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
pub enum SmartOnboardingTariffCodes {
  AGILE_OCTOPUS,
  OCTOPUS_GO,
  OCTOPUS_GO_FASTER,
  OCTOPUS_GO_GREEN,
  TESLA_IMPORT,
  INTELLIGENT_OCTOPUS,
  INTELLIGENT_FLUX,
  OUTGOING_FIXED,
  OUTGOING_AGILE,
  COSY_OCTOPUS,
  OCTOPUS_FLUX,
  POWERLOOP
}