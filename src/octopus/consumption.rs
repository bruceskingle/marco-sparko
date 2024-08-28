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

use crate::gql::types::Date;

use super::decimal::Decimal;


// Represents AccountUserType in the GraphQL schema
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct Consumption {
  start_date: Date,
  end_date: Date,
  quantity: Decimal,
  unit: ConsumptionUnit,
  usage_cost: i32,
  supply_charge: i32,
}

impl Consumption {
  pub fn get_field_names() -> &'static str {
    r#"
    startDate
    endDate
    quantity
    unit
    usageCost
    supplyCharge
    "#
  }
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
enum ConsumptionUnit {
  #[serde(rename = "kWh")]
  KWH,
  MJ
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let json = r#"
{
  "startDate": "2024-08-08",
  "endDate": "2024-08-20",
  "quantity": "334.7100",
  "unit": "kWh",
  "usageCost": 0,
  "supplyCharge": 0
}
        "#;

        let value = serde_json::from_str(json).unwrap();
        let consumption = Consumption::from(value);

        assert_eq!(consumption.start_date, Date::from_calendar_date(2024, time::Month::August, 8).unwrap());
        assert_eq!(consumption.end_date, Date::from_calendar_date(2024, time::Month::August, 20).unwrap());
    }
}