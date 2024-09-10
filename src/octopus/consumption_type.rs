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

use sparko_graphql::{types::ForwardPageOf, GraphQL, GraphQLQueryParams, GraphQLType, ParamBuffer, VariableBuffer};
use sparko_graphql::types::{DateTime, Int};
use sparko_graphql_derive::{GraphQLQueryParams, GraphQLType};

use super::decimal::Decimal;

/*
  These types relate to Meters. There is also a Consumption
  which is related to Transactions on Statements
*/

pub type ConsumptionConnection = ForwardPageOf<ConsumptionType>;
// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct ConsumptionConnection {
//   pageInfo: PageInfo,
//   edges: Vec<ConsumptionEdge>
//   totalCount: Int!

//   # Number of nodes in the edge.
//   edgeCount: Int!
// }


// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct ConsumptionEdge {
//   # The item at the end of the edge
//   node: ConsumptionType

//   # A cursor for use in pagination
//   cursor: String!
// }

// Energy consumption between two points in time.
#[derive(GraphQLType)]
#[graphql(params = "ConsumptionTypeQueryParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ConsumptionType {
  value: Decimal,
  start_at: DateTime,
  end_at: DateTime,
}

// impl GraphQLType<()> for ConsumptionType {
//   fn get_query(params: ()) -> String {
//     format!(r#"
//     value
//     startAt
//     endAt
//     "#)
//     }
// }v


#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ConsumptionTypeQueryParams {
  // Earliest consumption reading to return. Must specify a timezone.
  pub start_at: DateTime,

  // Aggregate consumption according to this grouping.
  pub grouping: ConsumptionGroupings,

  // Timezone to use for grouping.
  pub timezone: String,
  pub before: Option<String>,
  pub after: Option<String>,
  pub first: Option<Int>,
  pub last: Option<Int>,
}


// impl GraphQLQueryParams for ConsumptionTypeQueryParams
// {
//     fn get_formal_part(& self, params : & mut ParamBuffer, prefix : String)
//     {
//         params.push_formal(& prefix, "startAt", "DateTime");
//         self.grouping.get_formal_part(params, GraphQL ::
//         prefix(& prefix, "grouping"));
//         params.push_formal(& prefix, "timezone", "String");
//         params.push_formal(& prefix, "before", "String");
//         params.push_formal(& prefix, "after", "String");
//         params.push_formal(& prefix, "first", "Int");
//         params.push_formal(& prefix, "last", "Int");
//     } fn get_actual_part(& self, params : & mut ParamBuffer, prefix : String)
//     {
//         params.push_actual(& prefix, "startAt");
//         params.push_actual(& prefix, "timezone");
//         params.push_actual(& prefix, "before");
//         params.push_actual(& prefix, "after");
//         params.push_actual(& prefix, "first");
//         params.push_actual(& prefix, "last");
//     } fn
//     get_variables_part(& self, variables : & mut VariableBuffer, prefix :
//     String) -> Result < (), serde_json :: Error >
//     {
//         variables.push_variable(& prefix, "startAt", & self.start_at) ? ;
//         self.grouping.get_variables_part(variables, GraphQL ::
//         prefix(& prefix, "grouping")) ? ;
//         variables.push_variable(& prefix, "timezone", & self.timezone) ? ;
//         variables.push_variable(& prefix, "before", & self.before) ? ;
//         variables.push_variable(& prefix, "after", & self.after) ? ;
//         variables.push_variable(& prefix, "first", & self.first) ? ;
//         variables.push_variable(& prefix, "last", & self.last) ? ; Ok(())
//     }
// }

#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConsumptionGroupings {
  QuarterHour,
  HalfHour,
  HOUR,
  DAY,
  WEEK,
  MONTH,
  QUARTER,
}

// impl GraphQLQueryParams for ConsumptionGroupings
// {
//     fn get_formal_part(& self, params : & mut ParamBuffer, prefix : String) {}
//     fn get_actual_part(& self, params : & mut ParamBuffer, prefix : String) {}
//     fn
//     get_variables_part(& self, variables : & mut VariableBuffer, prefix :
//     String) -> Result < (), serde_json :: Error > { Ok(()) }
// }

// impl GraphQLQueryParams for ConsumptionGroupings {
//     fn get_formal_part(&self, params: &mut ParamBuffer, prefix: String) {
//         todo!()
//     }

//     fn get_actual_part(&self, params: &mut ParamBuffer, prefix: String) {
//         todo!()
//     }

//     fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: String) -> Result<(), serde_json::Error> {
//         todo!()
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let json = r#"
{
  "startAt": "2024-08-08T00:00:00.0Z",
  "endAt": "2024-08-20T00:00:00.0Z",
  "value": "334.7100",
  "unit": "kWh"
}
        "#;

        let value = serde_json::from_str(json).unwrap();
        let consumption = ConsumptionType::from(value);

        assert_eq!(consumption.start_at, DateTime::from_unix_timestamp(1723075200).unwrap());
        assert_eq!(consumption.end_at, DateTime::from_unix_timestamp(1724112000).unwrap());
    }
}