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
use sparko_graphql::{types::{Boolean, Date, DateTime, ForwardPageOf, Int, ID}, GraphQL, GraphQLQueryParams, GraphQLType, NoParams, ParamBuffer};
use crate::octopus::consumption_type::ConsumptionType;

use super::{consumption_type::ConsumptionTypeQueryParams, meter::{ExportMetersQueryParams, MeterIdView}};

#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[graphql(super_type = ["MeterPropertyViewInterface"])]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(tag = "__typename")]
pub enum MeterPropertyView {
  ElectricityMeterPropertyView(ElectricityMeterPropertyView),
  GasMeterPropertyView(GasMeterPropertyView)
}

impl MeterPropertyView {
  pub fn as_meter_interface(&self) -> &MeterPropertyViewInterface {
    match self {
      MeterPropertyView::ElectricityMeterPropertyView(txn) => &txn.meter_interface,
      MeterPropertyView::GasMeterPropertyView(txn) => &txn.meter_interface,
    }
  }
}

// interface MeterPropertyView in the schema
#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct MeterPropertyViewInterface {
  pub id: ID,
  pub serial_number: String,
  pub consumption_units: Option<String>,
  pub fuel_type: String,
}


#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ElectricityMeterPropertyView {
  #[serde(flatten)]
  pub meter_interface: MeterPropertyViewInterface,

  #[graphql(no_params)]
  pub import_meter: Option<MeterIdView>,
  pub node_id: ID,
  pub has_and_allows_hh_readings: Boolean,
}



// A gas meter has a register which holds readings. We would expect this to be a one-to-one relationship between meter and register.
// implements Node & Meter
#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct GasMeterPropertyView  {
  #[serde(flatten)]
  pub meter_interface: MeterPropertyViewInterface,

  pub node_id: ID,
  pub has_and_allows_hh_readings: Boolean,
}