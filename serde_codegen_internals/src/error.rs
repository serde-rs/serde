use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    UnexpectedItemKind,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "expected a struct or enum")
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "expected a struct or enum"
    }
}
