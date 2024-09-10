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
use once_cell::sync::Lazy;

use sparko_graphql::Error;


// use std::{sync::Mutex, collections::HashMap};

// static GLOBAL_DATA: Lazy<Mutex<HashMap<i32, String>>> = Lazy::new(|| {
//     let mut m = HashMap::new();
//     m.insert(13, "Spica".to_string());
//     m.insert(74, "Hoyten".to_string());
//     Mutex::new(m)
// });

// let x: Vec<format_description::FormatItem> = format_description::parse("[year]-[month]-[day]").unwrap();


static FORMAT: Lazy<Vec<format_description::FormatItem>> = 
  Lazy::new(|| {format_description::parse("[year]-[month]-[day]").unwrap()});


/// A GraphQL Date value
#[derive(Debug)]
pub struct Date(time::Date);

impl Date {
    pub fn from_calendar_date(year: i32, month: time::Month, day: u8) -> Result<Date, Error> {
        Ok(Date(time::Date::from_calendar_date(year, month, day)?))
    }
}

impl Deref for Date {
    type Target = time::Date;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for Date {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Eq for Date {
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(other)
    }
}


impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self.0.format(&FORMAT) {
        Ok(s) => f.pad(&s),
        Err(error) => Err(std::fmt::Error::custom(format!("Can't format Date: {}", error))),
      }
    }
  }
  
  impl FromStr for Date {
      type Err = Error;
  
      fn from_str(str: &str) -> Result<Date, Self::Err> {
        Ok(Date(time::Date::parse(str, 
          //&format_description::parse("[month]-[day]").unwrap()
          &FORMAT
      )?))
      }
  }
  
  
  struct DateVisitor;
  
  impl<'de> Visitor<'de> for DateVisitor {
      type Value = Date;
  
      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
          formatter.write_str("a date value YYYY-MM-DD")
      }
  
      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
          E: de::Error,
      {
        match  Date::from_str(value) {
          Ok(value) => Ok(value),
          Err(error) => Err(E::custom(format!("Invalid Date value: {}", error)))
        }
      }
  }
  
  impl<'de> serde::Deserialize<'de> for Date {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
      deserializer.deserialize_string(DateVisitor)
      }
  }

  impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
          match self.0.format(&FORMAT) {
            Ok(s) => serializer.serialize_str(&s),
            Err(error) => Err(S::Error::custom(format!("Can't format Date: {}", error))),
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
          value: Date,
      }
  
      #[test]
      fn test_from_str() {
        let expected = time::Date::from_calendar_date(2024, time::Month::April, 5).unwrap();
        let value = Date::from_str("2024-04-05").unwrap();
  
        assert_eq!(value.0, expected);
      }
  
      #[test]
      fn test_display() {
        let date = Date::from_str("2024-04-05").unwrap();
        let value = format!("{}", date);
  
        assert_eq!(value, "2024-04-05");
      }
  
      fn expect_parse(s: &str, year: i32, month: time::Month, day: u8) {
        let expect = time::Date::from_calendar_date(year, month, day).unwrap();
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
        
        expect_parse(r#"{ "value": "2024-04-05" }"#, 2024, time::Month::April, 5);
        
        expect_parse_error(r#"{ "value": "444." }"#);
        expect_parse_error(r#"{ "value": "1/2/2022" }"#);
      }
  
      #[test]
      fn test_serialize() {
        let value = Date::from_calendar_date(1944, time::Month::June, 6).unwrap();
        assert_eq!(serde_json::to_string(&MyStruct {
          value
        }).unwrap(), "{\"value\":\"1944-06-06\"}");
      }
  }