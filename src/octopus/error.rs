use std::error::Error as StdError;
use std::num::ParseIntError;
use std::fmt::{self, Display};
use serde::{de, ser};

#[derive(Debug)]
pub enum Error {
    GraphQLError(sparko_graphql::Error),
    IOError(std::io::Error),
    JsonError(serde_json::Error),
    InternalError(&'static str),
    CallerError(&'static str),
    StringError(String),
    WrappedError(Box<dyn StdError>)
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::GraphQLError(err) => f.write_fmt(format_args!("GraphQLError({})", err)),
            Error::IOError(err) => f.write_fmt(format_args!("IOError({})", err)),
            Error::JsonError(err) => f.write_fmt(format_args!("JsonError({})", err)),
            Error::InternalError(err) => f.write_fmt(format_args!("InternalError({})", err)),
            Error::CallerError(err) => f.write_fmt(format_args!("CallerError({})", err)),
            Error::StringError(err) => f.write_fmt(format_args!("StringError({})", err)),
            Error::WrappedError(err) => f.write_fmt(format_args!("WrappedError({})", err))
        }
    }
}

impl StdError for Error {

}



impl From<Box<dyn StdError>> for Error {
    fn from(err: Box<dyn StdError>) -> Error {
        Error::WrappedError(err)
    }
}
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::WrappedError(Box::new(err))
    }
}

impl From<rust_decimal::Error> for Error {
    fn from(err: rust_decimal::Error) -> Error {
        Error::StringError(err.to_string())
    }
}

impl From<sparko_graphql::error::Error> for Error {
    fn from(err: sparko_graphql::error::Error) -> Error {
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