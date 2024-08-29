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


/// A GraphQL Boolean value
#[derive(Serialize, Debug)]
pub struct Boolean(bool);

impl Boolean {
  pub fn new(s: bool) -> Boolean {
    Boolean(s)
  }
}

impl Deref for Boolean {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for Boolean {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Boolean {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}


impl Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      self.0.fmt(f)
    }
  }
  
  impl FromStr for Boolean {
      type Err = Error;
  
      fn from_str(str: &str) -> Result<Boolean, Self::Err> {
        Ok(Boolean(bool::from_str(str)?))
      }
  }
  
  
  struct BooleanVisitor;
  
  impl<'de> Visitor<'de> for BooleanVisitor {
      type Value = Boolean;
  
      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
          formatter.write_str("a bool value")
      }
  
      fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
      where
          E: de::Error,
      {
        Ok(Boolean(value))
      }
  }
  
  
  impl<'de> serde::Deserialize<'de> for Boolean {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
      deserializer.deserialize_bool(BooleanVisitor)
      }
  }
  
  #[cfg(test)]
  mod tests {
      use display_json::DisplayAsJsonPretty;
    use serde::Deserialize;

    use super::*;
  
  
      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      struct MyStruct {
          value: Boolean,
      }
  
      #[test]
      fn test_from_str() {  
        assert_eq!(Boolean::from_str("true").unwrap().0, true);
        assert_eq!(Boolean::from_str("false").unwrap().0, false);

        expect_str_error("maybe");
        expect_str_error("\"maybe\"");
        expect_str_error("2");
        expect_str_error("[true]");
      }

      fn expect_str_error(s: &str) {
        let result = Boolean::from_str(s);
        if let Ok(_) = result {
          panic!("Expecting error for {}", s);
        }
      }
  
      fn expect_parse(s: &str, expect: bool) {
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
        
        expect_parse(r#"{ "value": true }"#, true);
        expect_parse(r#"{ "value": false }"#, false);
        
        expect_parse_error(r#"{ "value": 123 }"#);
        expect_parse_error(r#"{ "value": [1,2,3]] }"#);
        expect_parse_error(r#"{ "value": {} }"#);
      }
  
      #[test]
      fn test_serialize() {
        assert_eq!(serde_json::to_string(&MyStruct {
          value: Boolean(true)
        }).unwrap(), "{\"value\":true}");
      }
  }