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

use serde::{Deserializer, Serialize};
use serde::de::{self, Visitor};

use crate::gql::Error;


/// A GraphQL ID value
#[derive(Serialize, Debug)]
pub struct ID(String);

impl ID {
  pub fn new(s: String) -> ID {
    ID(s)
  }
}

impl Deref for ID {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for ID {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for ID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}


impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.write_fmt(format_args!("{}", self.0))
    }
  }
  
  impl FromStr for ID {
      type Err = Error;
  
      fn from_str(str: &str) -> Result<ID, Self::Err> {
        Ok(ID(String::from(str)))
      }
  }
  
  
  struct IDVisitor;
  
  impl<'de> Visitor<'de> for IDVisitor {
      type Value = ID;
  
      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
          formatter.write_str("a string value")
      }
  
      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
          E: de::Error,
      {
        match  ID::from_str(value) {
          Ok(value) => Ok(value),
          Err(error) => Err(E::custom(format!("Invalid ID value: {}", error)))
        }
      }
  }
  
  
  impl<'de> serde::Deserialize<'de> for ID {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
      deserializer.deserialize_string(IDVisitor)
      }
  }
  
  #[cfg(test)]
  mod tests {
    use display_json::DisplayAsJsonPretty;
    use serde::Deserialize;

    use super::*;
  
  
      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      struct MyStruct {
          value: ID,
      }
  
      #[test]
      fn test_from_str() {
        let expected = String::from("IdValue");
        let value = ID::from_str(&expected).unwrap();
  
        assert_eq!(value.0, expected);
      }
  
      fn expect_parse(s: &str, value: &str) {
        let expect = String::from(value);
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
        
        expect_parse(r#"{ "value": "2024-04-05" }"#, "2024-04-05");
        
        expect_parse_error(r#"{ "value": 123 }"#);
        expect_parse_error(r#"{ "value": [1,2,3]] }"#);
        expect_parse_error(r#"{ "value": {} }"#);
      }
  
      #[test]
      fn test_serialize() {
        assert_eq!(serde_json::to_string(&MyStruct {
          value: ID(String::from("King Richard the Third"))
        }).unwrap(), "{\"value\":\"King Richard the Third\"}");
      }
  }