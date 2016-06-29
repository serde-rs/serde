use std::{error, fmt};

use serde::{ser, de};

use token::Token;

#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    CustomError(String),
    EndOfStreamError,
    UnknownFieldError(String),
    UnknownVariantError(String),
    MissingFieldError(&'static str),
    DuplicateFieldError(&'static str),
    InvalidName(&'static str),
    InvalidValue(String),
    UnexpectedToken(Token<'static>),
    ValueError(de::value::Error),
}

impl ser::Error for Error {
    fn custom<T: Into<String>>(msg: T) -> Error {
        Error::CustomError(msg.into())
    }

    fn invalid_value(msg: &str) -> Error {
        Error::InvalidValue(msg.to_owned())
    }
}

impl de::Error for Error {
    fn custom<T: Into<String>>(msg: T) -> Error {
        Error::CustomError(msg.into())
    }

    fn end_of_stream() -> Error {
        Error::EndOfStreamError
    }

    fn invalid_value(msg: &str) -> Error {
        Error::InvalidValue(msg.to_owned())
    }

    fn unknown_field(field: &str) -> Error {
        Error::UnknownFieldError(field.to_owned())
    }

    fn unknown_variant(variant: &str) -> Error {
        Error::UnknownVariantError(variant.to_owned())
    }

    fn missing_field(field: &'static str) -> Error {
        Error::MissingFieldError(field)
    }

    fn duplicate_field(field: &'static str) -> Error {
        Error::DuplicateFieldError(field)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str(format!("{:?}", self).as_ref())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Serde Error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
