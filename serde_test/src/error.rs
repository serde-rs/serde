// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error;
use std::fmt::{self, Display};

use serde::{ser, de};

#[derive(Clone, Debug)]
pub struct Error {
    msg: String,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error { msg: msg.to_string() }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error { msg: msg.to_string() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.msg)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl PartialEq<str> for Error {
    fn eq(&self, other: &str) -> bool {
        self.msg == other
    }
}
