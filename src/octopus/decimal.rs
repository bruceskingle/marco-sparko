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
use std::str::FromStr;
use serde::{Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};

use super::Error;


#[derive(Debug)]
pub struct Decimal {
  int: i32,
  dec: u32
}

impl Decimal {

  pub fn new(int: i32,
    dec: u32) -> Decimal {
      Decimal {
        int,
        dec
      }
    }
}

impl Display for Decimal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.pad(&format!("{}.{}", self.int, self.dec))
  }
}

impl FromStr for Decimal {
    type Err = Error;

    fn from_str(str: &str) -> Result<Decimal, Self::Err> {
        let mut int: i32 = 0;
        let mut dec: u32 = 0;
        let mut i = 0;

        for part in str.split(".") {
          if i==0 {
            int = i32::from_str(part)?
          }
          else if i==1 {
            dec = u32::from_str(part)?
          }
          else {
            return Err(Error::StringError(format!("Too many decimal points '{}'", str)))
          }
          i += 1;
        }

        if i==0 {
          return Err(Error::StringError(format!("Empty value ''{}'", str)))
        }

        Ok(Decimal {
          int,
          dec
        })

    }
}


struct DecimalVisitor;

impl<'de> Visitor<'de> for DecimalVisitor {
    type Value = Decimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a decimal value")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
      match  Decimal::from_str(value) {
        Ok(value) => Ok(value),
        Err(error) => Err(E::custom(format!("Invalid Decimal value: {}", error)))
      }
    }
}

impl<'de> serde::Deserialize<'de> for Decimal {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    deserializer.deserialize_string(DecimalVisitor)
    }
}



impl Serialize for Decimal {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
  {
    serializer.serialize_str(&format!("{}.{}", self.int, self.dec))
  }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct MyStruct {
        decimal: Decimal,
    }

    #[test]
    fn test_from_str() {
      let decimal = Decimal::from_str("123.456").unwrap();

      assert_eq!(decimal.int, 123);
      assert_eq!(decimal.dec, 456);
    }

    fn expect_parse(s: &str, int: i32, dec: u32) {
      let result: Result<MyStruct, serde_json::Error> = serde_json::from_str(s);
      if let Ok(value) = result {
        assert_eq!(value.decimal.int, int);
        assert_eq!(value.decimal.dec, dec);
      }
      else {
        panic!("Expecting {}.{} for {}", int, dec, s);
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
      
      expect_parse(r#"{ "decimal": "123.456" }"#, 123, 456);
      
      expect_parse(r#"{ "decimal": "444" }"#, 444, 0);
      
      expect_parse(r#"{ "decimal": "444.0" }"#, 444, 0);
      
      expect_parse_error(r#"{ "decimal": "444." }"#);
      expect_parse_error(r#"{ "decimal": ".444" }"#);
      
      expect_parse(r#"{ "decimal": "0.444" }"#, 0, 444);
      
      expect_parse(r#"{ "decimal": "0000.444" }"#, 0, 444);
      
      expect_parse(r#"{ "decimal": "876.444" }"#, 876, 444);
      
      expect_parse_error(r#"{ "decimal": "0.1.2" }"#);
    }
  
    #[test]
    fn test_serialize() {
      assert_eq!(serde_json::to_string(&MyStruct {
        decimal: Decimal::new(3, 14159)
      }).unwrap(), "{\"decimal\":\"3.14159\"}");
    }
}