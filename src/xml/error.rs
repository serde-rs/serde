use std::error;
use std::fmt;
use std::io;

use de;

use self::ErrorCode::*;

/// The errors that can arise while parsing a JSON stream.
#[derive(Copy, Clone, PartialEq)]
pub enum ErrorCode {
    EOF,
    RawValueCannotHaveAttributes,
    InvalidOptionalElement,
    NotUtf8,
    SerdeExpectedSomeValue,
    ExpectedEOF,
    LexingError(super::de::LexerError),
    Expected(&'static str),
    XmlDoesntSupportSeqofSeq,
}

impl fmt::Debug for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Debug;

        match *self {
            EOF => "EOF".fmt(f),
            RawValueCannotHaveAttributes => "raw value cannot have attributes".fmt(f),
            InvalidOptionalElement => "invalid optional element".fmt(f),
            NotUtf8 => "bad utf8".fmt(f),
            SerdeExpectedSomeValue => "serde expected some value".fmt(f),
            LexingError(e) => write!(f, "error during lexing: \"{:?}\"", e),
            Expected(what) => write!(f, "expected {}", what),
            XmlDoesntSupportSeqofSeq => "xml doesn't support sequences of sequences.\
            Please use a newtype".fmt(f),
            ExpectedEOF => "trailing characters".fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    /// msg, line, col
    SyntaxError(ErrorCode, usize, usize),
    IoError(io::Error),
    MissingFieldError(&'static str),
    UnknownField(String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::SyntaxError(..) => "syntax error",
            Error::IoError(ref error) => error::Error::description(error),
            Error::MissingFieldError(_) => "missing field",
            Error::UnknownField(_) => "unknown field",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref error) => Some(error),
            _ => None,
        }
    }

}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::SyntaxError(ref code, line, col) => {
                write!(fmt, "{:?} at line {} column {}", code, line, col)
            }
            Error::IoError(ref error) => fmt::Display::fmt(error, fmt),
            Error::MissingFieldError(ref field) => {
                write!(fmt, "missing field {}", field)
            },
            Error::UnknownField(ref field) => {
                write!(fmt, "unknown field {}", field)
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IoError(error)
    }
}

impl de::Error for Error {
    fn syntax_error() -> Error {
        Error::SyntaxError(SerdeExpectedSomeValue, 0, 0)
    }

    fn unknown_field_error(field: &str) -> Error {
        Error::UnknownField(field.to_string())
    }

    fn end_of_stream_error() -> Error {
        Error::SyntaxError(EOF, 0, 0)
    }

    fn missing_field_error(field: &'static str) -> Error {
        Error::MissingFieldError(field)
    }
}
