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

use std::error::Error as StdError;
use std::num::ParseIntError;
use std::fmt::{self, Display};
use serde::{de, ser};

#[derive(Debug)]
pub enum Error {
    GraphQLError(crate::gql::Error),
    IOError(std::io::Error),
    JsonError(serde_json::Error),
    InternalError(&'static str),
    CallerError(&'static str),
    StringError(String)
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::GraphQLError(err) => f.write_fmt(format_args!("GraphQLError({})", err)),
            Error::IOError(err) => f.write_fmt(format_args!("IOError({})", err)),
            Error::JsonError(err) => f.write_fmt(format_args!("JsonError({})", err)),
            Error::InternalError(err) => f.write_fmt(format_args!("InternalError({})", err)),
            Error::CallerError(err) => f.write_fmt(format_args!("CallerError({})", err)),
            Error::StringError(err) => f.write_fmt(format_args!("StringError({})", err))
        }
    }
}

impl StdError for Error {

}



impl From<rust_decimal::Error> for Error {
    fn from(err: rust_decimal::Error) -> Error {
        Error::StringError(err.to_string())
    }
}

impl From<crate::gql::error::Error> for Error {
    fn from(err: crate::gql::error::Error) -> Error {
        Error::GraphQLError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonError(err)
    }
}


impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<crate::Error> for Error {
    fn from(err: crate::Error) -> Error {
        Error::StringError(format!("{:?}", err))
    }
}



impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Error {
        Error::StringError(format!("{:?}", err))
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::StringError(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::StringError(msg.to_string())
    }
}