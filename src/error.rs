use std::error::Error as StdError;
use std::num::ParseIntError;
use std::fmt::{self, Display};
use std::panic::Location;
use serde::{de, ser};

#[derive(Debug)]
pub struct Error {
    location: &'static Location<'static>,
    cause: Cause,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.cause, self.location)
    }
}

impl StdError for Error {
}

#[derive(Debug)]
pub enum Cause {
    GraphQLError(sparko_graphql::Error),
    IOError(std::io::Error),
    JsonError(serde_json::Error),
    InternalError(&'static str),
    CallerError(&'static str),
    StringError(String),
    WrappedError(Box<dyn StdError>)
}

impl Display for Cause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cause::GraphQLError(err) => f.write_fmt(format_args!("GraphQLError({})", err)),
            Cause::IOError(err) => f.write_fmt(format_args!("IOError({})", err)),
            Cause::JsonError(err) => f.write_fmt(format_args!("JsonError({})", err)),
            Cause::InternalError(err) => f.write_fmt(format_args!("InternalError({})", err)),
            Cause::CallerError(err) => f.write_fmt(format_args!("CallerError({})", err)),
            Cause::StringError(err) => f.write_fmt(format_args!("StringError({})", err)),
            Cause::WrappedError(err) => f.write_fmt(format_args!("WrappedError({})", err))
        }
    }
}




impl From<Box<dyn StdError>> for Error {
    #[track_caller]
    #[inline]
    fn from(err: Box<dyn StdError>) -> Error {
        Error {
            location: Location::caller(),
            cause: Cause::WrappedError(err)
        }
    }
}
impl From<reqwest::Error> for Error {
    #[track_caller]
    #[inline]
    fn from(err: reqwest::Error) -> Error {
        Error {
            location: Location::caller(),
            cause: Cause::WrappedError(Box::new(err))
        }
    }
}

impl From<rust_decimal::Error> for Error {
    #[track_caller]
    #[inline]
    fn from(err: rust_decimal::Error) -> Error {
        Error {
            location: Location::caller(),
            cause: Cause::StringError(err.to_string())
        }
    }
}

impl From<sparko_graphql::error::Error> for Error {
    #[track_caller]
    #[inline]
    fn from(err: sparko_graphql::error::Error) -> Error {
        Error {
            location: Location::caller(),
            cause: Cause::GraphQLError(err)
        }
    }
}

impl From<serde_json::Error> for Error {
    #[track_caller]
    #[inline]
    fn from(err: serde_json::Error) -> Error {
        Error {
            location: Location::caller(),
            cause: Cause::JsonError(err)
        }
    }
}


impl From<std::io::Error> for Error {
    #[track_caller]
    #[inline]
    fn from(err: std::io::Error) -> Error {
        Error {
            location: Location::caller(),
            cause: Cause::IOError(err)
        }
    }
}

// impl From<crate::Error> for Error {
//     #[track_caller]
//     #[inline]
//     fn from(err: crate::Error) -> Error {
//         Error {
//             location: Location::caller(),
//             cause: Cause::StringError(format!("{:?}", err))
//         }
//     }
// }



impl From<ParseIntError> for Error {
    #[track_caller]
    #[inline]
    fn from(err: ParseIntError) -> Error {
        Error {
            location: Location::caller(),
            cause: Cause::StringError(format!("{:?}", err))
        }
    }
}

impl ser::Error for Error {
    #[track_caller]
    #[inline]
    fn custom<T: Display>(msg: T) -> Self {
        Error {
            location: Location::caller(),
            cause: Cause::StringError(msg.to_string())
        }
    }
}

impl de::Error for Error {
    #[track_caller]
    #[inline]
    fn custom<T: Display>(msg: T) -> Self {
        Error {
            location: Location::caller(),
            cause: Cause::StringError(msg.to_string())
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    #[track_caller]
    #[inline]
    fn from(err: std::sync::PoisonError<T>) -> Error {
        Error {
            location: Location::caller(),
            cause: Cause::StringError(format!("Mutex poison error {:?}", err))
        }
    }
}

impl From<String> for Error {
    #[track_caller]
    #[inline]
    fn from(err: String) -> Error {
        Error {
            location: Location::caller(),
            cause: Cause::StringError(err)
        }
    }
}

impl From<&'static str> for Error {
    #[track_caller]
    #[inline]
    fn from(err: &'static str) -> Error {
        Error {
            location: Location::caller(),
            cause: Cause::CallerError(err)
        }
    }
}