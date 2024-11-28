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
use serde::{Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};

use sparko_graphql::types::Int;

use super::Error;


#[derive(Debug, Clone, Copy)]
pub struct Decimal(rust_decimal::Decimal);

impl Decimal {

  pub fn new(num: i64, scale: u32) -> Decimal {
    Decimal (rust_decimal::Decimal::new(num, scale))
  }

  pub fn from_int(i: &Int) -> Decimal {
    Decimal::new(**i as i64, 0)
  }

  pub fn is_positive(&self)-> bool {
    self.0.gt(&rust_decimal::Decimal::from(0)) //.gt(&0)
  }
}

impl Div for Decimal {
    type Output = Decimal;

    fn div(self, rhs: Self) -> Self::Output {
        Decimal(self.0.div(rhs.0))
    }
}

impl Mul for Decimal {
    type Output = Decimal;

    fn mul(self, rhs: Self) -> Self::Output {
        Decimal(self.0.mul(rhs.0))
    }
}

impl AddAssign for Decimal {
    fn add_assign(&mut self, rhs: Self) {
        self.0.add_assign(rhs.0);
    }
}

impl Add for Decimal {
    type Output = Decimal;

    fn add(self, rhs: Self) -> Self::Output {
        Decimal(self.0.add(rhs.0))
    }
}

impl Sub for Decimal {
    type Output = Decimal;

    fn sub(self, rhs: Self) -> Self::Output {
        Decimal(self.0.sub(rhs.0))
    }
}

impl Display for Decimal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // f.pad(&format!("{}.{}", self.int, self.dec))
    self.0.fmt(f)
  }
}

impl Deref for Decimal {
    type Target = rust_decimal::Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Decimal {
    type Err = Error;

    fn from_str(str: &str) -> Result<Decimal, Self::Err> {
      Ok(Decimal(rust_decimal::Decimal::from_str(str)?))
        // let mut int: i32 = 0;
        // let mut dec: u32 = 0;
        // let mut i = 0;

        // for part in str.split(".") {
        //   if i==0 {
        //     int = i32::from_str(part)?
        //   }
        //   else if i==1 {
        //     dec = u32::from_str(part)?
        //   }
        //   else {
        //     return Err(Error::StringError(format!("Too many decimal points '{}'", str)))
        //   }
        //   i += 1;
        // }

        // if i==0 {
        //   return Err(Error::StringError(format!("Empty value ''{}'", str)))
        // }

        // Ok(Decimal {
        //   int,
        //   dec
        // })

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
    // serializer.serialize_str(&format!("{}.{}", self.int, self.dec))
    serializer.serialize_str(&self.0.to_string())
  }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;


    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct MyStruct {
        decimal: Decimal,
    }

    #[test]
    fn test_from_str() {

      let dd = rust_decimal::Decimal::new(123456, 3);
      assert_eq!(dd.to_string(), "123.456");

      let decimal = Decimal::from_str("123.456").unwrap();

      assert_eq!(decimal.0, rust_decimal::Decimal::new(123456, 3));
    }

    fn expect_parse(s: &str, v:rust_decimal::Decimal) {
      let result: Result<MyStruct, serde_json::Error> = serde_json::from_str(s);
      if let Ok(value) = result {
        assert_eq!(value.decimal.0, v);
      }
      else {
        panic!("Expecting {} for {}", v, s);
      }
    }

    fn expect_parse_error(s: &str) {
      let result: Result<MyStruct, serde_json::Error> = serde_json::from_str(s);
      if let Ok(v) = result {
        panic!("Expecting error for {}, got {}", s, v.decimal);
      }
    }

    #[test]
    fn test_parse() {
      
      expect_parse(r#"{ "decimal": "123.456" }"#, rust_decimal::Decimal::new(123456,3));
      
      expect_parse(r#"{ "decimal": "444" }"#, rust_decimal::Decimal::new(444, 0));
      
      expect_parse(r#"{ "decimal": "444.0" }"#, rust_decimal::Decimal::new(444, 0));
      expect_parse(r#"{ "decimal": "444." }"#, rust_decimal::Decimal::new(444, 0));
      expect_parse(r#"{ "decimal": ".444" }"#, dec!(0.444));
      
      // expect_parse_error(r#"{ "decimal": "444." }"#);
      // expect_parse_error(r#"{ "decimal": ".444" }"#);
      
      expect_parse(r#"{ "decimal": "0.444" }"#, dec!(0.444));
      
      expect_parse(r#"{ "decimal": "0000.444" }"#, dec!(0.444));
      
      expect_parse(r#"{ "decimal": "876.444" }"#, dec!(876.444));
      
      expect_parse_error(r#"{ "decimal": "0.1.2" }"#);
    }
  
    #[test]
    fn test_serialize() {
      assert_eq!(serde_json::to_string(&MyStruct {
        decimal: Decimal::new(314159, 5)
      }).unwrap(), "{\"decimal\":\"3.14159\"}");
    }
}