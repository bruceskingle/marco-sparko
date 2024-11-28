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

use super::{consumption_type::ConsumptionTypeQueryParams, meter::{Meter, MeterQueryParams}, meter_property_view::MeterPropertyView};


#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[graphql(super_type = ["MeterPointPropertyViewInterface"])]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(tag = "__typename")]
pub enum MeterPointPropertyView {
  ElectricityMeterPointPropertyView(ElectricityMeterPointPropertyView),
  GasMeterPointPropertyView(GasMeterPointPropertyView)
}

impl MeterPointPropertyView {
  pub fn as_meter_point_interface(&self) -> &MeterPointPropertyViewInterface {
    match self {
      MeterPointPropertyView::ElectricityMeterPointPropertyView(txn) => &txn.meter_point_interface,
      MeterPointPropertyView::GasMeterPointPropertyView(txn) => &txn.meter_point_interface,
    }
  }
}

#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct MeterPointPropertyViewInterface {
  pub status: String,
  #[graphql(no_params)]
  pub meters: Vec<MeterPropertyView>,
  // // Details of an ongoing enrolment process.
  // #[graphql(no_params)]
  // pub enrolment: Option<EnrolmentType>
}


#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct GasMeterPointPropertyView{
  #[serde(flatten)]
  pub meter_point_interface: MeterPointPropertyViewInterface,

  pub id: ID,
  pub mprn: String,

}

#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ElectricityMeterPointPropertyView {
  #[serde(flatten)]
  pub meter_point_interface: MeterPointPropertyViewInterface,

  pub id: ID,
  pub mpan: String,
}