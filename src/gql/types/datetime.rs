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


use std::fmt::{self, Display};
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};
use serde::ser::Error as SerError;
use time::format_description;

use sparko_graphql::Error;

static FORMAT: format_description::well_known::Rfc3339 = format_description::well_known::Rfc3339;


/// A GraphQL OffsetDateTime value
#[derive(Debug)]
pub struct DateTime(time::OffsetDateTime);

impl DateTime {
  pub fn from_unix_timestamp(timestamp: i64) -> Result<DateTime, Error> {
      Ok(DateTime(time::OffsetDateTime::from_unix_timestamp(timestamp)?))
  }

  pub fn from_unix_timestamp_nanos(timestamp: i128) -> Result<DateTime, Error> {
    Ok(DateTime(time::OffsetDateTime::from_unix_timestamp_nanos(timestamp)?))
  }

  // pub fn from_calendar_date(year: i32, month: time::Month, day: u8) -> Result<OffsetDateTime, Error> {
  //   Ok(OffsetDateTime(time::OffsetDateTime: from_calendar_date(year, month, day)?))
// }
}

impl Deref for DateTime {
    type Target = time::OffsetDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for DateTime {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Eq for DateTime {
}

impl Ord for DateTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(other)
    }
}


impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self.0.format(&FORMAT) {
        Ok(s) => f.pad(&s),
        Err(error) => Err(std::fmt::Error::custom(format!("Can't format OffsetDateTime: {}", error))),
      }
    }
  }
  
  impl FromStr for DateTime {
      type Err = Error;
  
      fn from_str(str: &str) -> Result<DateTime, Self::Err> {
        Ok(DateTime(time::OffsetDateTime::parse(str, 
          //&format_description::parse("[month]-[day]").unwrap()
          &FORMAT
      )?))
      }
  }
  
  
  struct DateVisitor;
  
  impl<'de> Visitor<'de> for DateVisitor {
      type Value = DateTime;
  
      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
          formatter.write_str("a date value YYYY-MM-DD")
      }
  
      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
          E: de::Error,
      {
        match  DateTime::from_str(value) {
          Ok(value) => Ok(value),
          Err(error) => Err(E::custom(format!("Invalid OffsetDateTime value: {}", error)))
        }
      }
  }
  
  impl<'de> serde::Deserialize<'de> for DateTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
      deserializer.deserialize_string(DateVisitor)
      }
  }

  impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
          match self.0.format(&FORMAT) {
            Ok(s) => serializer.serialize_str(&s),
            Err(error) => Err(S::Error::custom(format!("Can't format OffsetDateTime: {}", error))),
          }
      }
    }
  
  
  #[cfg(test)]
  mod tests {
      use display_json::DisplayAsJsonPretty;
    use serde::Deserialize;

    use super::*;
  
  
      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      struct MyStruct {
          value: DateTime,
      }
  
      #[test]
      fn test_from_str() {
        let expected = time::OffsetDateTime::from_unix_timestamp(0).unwrap();
        let value = DateTime::from_str("1970-01-01T00:00:00.0Z").unwrap();
  
        assert_eq!(value.0, expected);
      }
  
      #[test]
      fn test_display() {
        let date = DateTime::from_str("1944-06-06T00:06:00Z").unwrap();
        let value = format!("{}", date);
  
        assert_eq!(value, "1944-06-06T00:06:00Z");
      }
  
      fn expect_parse(s: &str, timestamp: i64) {
        let expect = time::OffsetDateTime::from_unix_timestamp(timestamp).unwrap();
        let result: Result<MyStruct, serde_json::Error> = serde_json::from_str(s);
        if let Ok(my_struct) = result {
          assert_eq!(my_struct.value.0, expect);
        }
        else {
          panic!("Expecting {:?} for {}", expect, s);
        }
      }
  
      fn expect_parse_error(s: &str) {
        let result: Result<MyStruct, serde_json::Error> = serde_json::from_str(s);
        if let Ok(_) = result {
          panic!("Expecting error for {}", s);
        }
      }
  
      #[test]
      fn test_parse() {
        
        expect_parse(r#"{ "value": "1944-06-06T00:06:00Z" }"#, -806975640);
        
        expect_parse_error(r#"{ "value": "444." }"#);
        expect_parse_error(r#"{ "value": "1/2/2022" }"#);
      }
  
      #[test]
      fn test_serialize() {
        let value = DateTime::from_unix_timestamp(-806975640).unwrap();
        assert_eq!(serde_json::to_string(&MyStruct {
          value
        }).unwrap(), "{\"value\":\"1944-06-06T00:06:00Z\"}");
      }
  }