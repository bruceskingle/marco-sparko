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
use std::ops::{Add, AddAssign, Deref, Div, Mul, Sub};
use std::str::FromStr;

use serde::{Deserializer, Serialize};
use serde::de::{self, Visitor};

use sparko_graphql::Error;

/// A GraphQL Int value
#[derive(Serialize, Debug, Clone, Copy)]
pub struct Int(i32);

impl Int {
  pub fn new(s: i32) -> Int {
    Int(s)
  }

  pub fn as_decimal(&self, decimals: usize) -> String{
    // let f = 10 ^ decimals as u32;
    let mut s = format!("{}", self.0);

    let mut l = s.len();

    if l <= decimals {
      while l < decimals{
        s.insert(0, '0');
        l += 1;
      }
      s.insert(0, '.');
      s.insert(0, '0');
    }
    else {
      s.insert(s.len() - decimals, '.');
    }
    s
  }
}

impl Deref for Int {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for Int {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Int {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}


impl Div for Int {
  type Output = Int;

  fn div(self, rhs: Self) -> Self::Output {
      Int(self.0.div(rhs.0))
  }
}

impl Mul for Int {
  type Output = Int;

  fn mul(self, rhs: Self) -> Self::Output {
      Int(self.0.mul(rhs.0))
  }
}

impl AddAssign for Int {
  fn add_assign(&mut self, rhs: Self) {
      self.0.add_assign(rhs.0);
  }
}

impl Add for Int {
  type Output = Int;

  fn add(self, rhs: Self) -> Self::Output {
      Int(self.0.add(rhs.0))
  }
}

impl Sub for Int {
  type Output = Int;

  fn sub(self, rhs: Self) -> Self::Output {
      Int(self.0.sub(rhs.0))
  }
}


impl Display for Int {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      self.0.fmt(f)
    }
  }
  
  impl FromStr for Int {
      type Err = Error;
  
      fn from_str(str: &str) -> Result<Int, Self::Err> {
        Ok(Int(str.parse::<i32>()?))
      }
  }
  
  
  struct IntVisitor;
  
  impl<'de> Visitor<'de> for IntVisitor {
      type Value = Int;
  
      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
          formatter.write_str("an i32 value")
      }

      fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
      where
          E: de::Error,
      {
        match  i32::try_from(value) {
          Ok(value) => Ok(Int::new(value)),
          Err(error) => Err(E::custom(format!("Invalid i32 value: {}", error)))
        }
      }
  
      fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
      where
          E: de::Error,
      {
        match  i32::try_from(value) {
          Ok(value) => Ok(Int::new(value)),
          Err(error) => Err(E::custom(format!("Invalid i32 value: {}", error)))
        }
      }
  }
  
  
  impl<'de> serde::Deserialize<'de> for Int {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
      deserializer.deserialize_any(IntVisitor)
      }
  }
  
  #[cfg(test)]
  mod tests {
      use display_json::DisplayAsJsonPretty;
    use serde::Deserialize;

    use super::*;
  
  
      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      struct MyStruct {
          value: Int,
      }
  
      #[test]
      fn test_from_str() {
        let expected = 42;
        let value = Int::from_str("42").unwrap();
  
        assert_eq!(value.0, expected);
      }
  
      fn expect_parse(s: &str, expect: i32) {
        let result: Result<MyStruct, serde_json::Error> = serde_json::from_str(s);

        if let Err(error) = &result {
          println!("Error {}", error);
        }
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
        
        expect_parse(r#"{ "value": 42 }"#, 42);
        expect_parse(r#"{ "value": -42 }"#, -42);

        expect_parse(r#"{ "value": 32000 }"#, 32000);
        expect_parse(r#"{ "value": -32000 }"#, -32000);
        
        expect_parse(r#"{ "value": 66000 }"#, 66000);
        expect_parse(r#"{ "value": -66000 }"#, -66000);

        
        expect_parse_error(r#"{ "value": [1,2,3]] }"#);
        expect_parse_error(r#"{ "value": {} }"#);
      }
  
      #[test]
      fn test_serialize() {
        assert_eq!(serde_json::to_string(&MyStruct {
          value: Int(42)
        }).unwrap(), "{\"value\":42}");
      }
  
      #[test]
      fn test_as_decimal() {
        assert_eq!(Int(1).as_decimal(2), "0.01");
        assert_eq!(Int(12).as_decimal(2), "0.12");
        assert_eq!(Int(4212).as_decimal(2), "42.12");
      }
  }