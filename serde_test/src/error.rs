use std::error;
use std::fmt::{self, Display};

use serde::{ser, de};

use token::Token;

#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    Message(String),
    InvalidName(&'static str),
    UnexpectedToken(Token<'static>),
    EndOfTokens,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error::Message(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Message(ref msg) => formatter.write_str(msg),
            Error::InvalidName(name) => write!(formatter, "invalid name `{}`", name),
            Error::UnexpectedToken(_) => formatter.write_str("unexpected token"),
            Error::EndOfTokens => formatter.write_str("end of tokens"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::InvalidName(_) => "invalid name",
            Error::UnexpectedToken(_) => "unexpected token",
            Error::EndOfTokens => "end of tokens",
        }
    }
}
