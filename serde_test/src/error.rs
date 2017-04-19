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

use token::Token;

/// Error expected in `assert_ser_tokens_error` and `assert_de_tokens_error`.
#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    /// A custom error.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # extern crate serde_test;
    /// #
    /// # fn main() {
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// use serde_test::{assert_ser_tokens_error, Token, Error};
    ///
    /// #[derive(Serialize)]
    /// struct Example {
    ///     lock: Arc<Mutex<u32>>,
    /// }
    ///
    /// let example = Example { lock: Arc::new(Mutex::new(0)) };
    /// let lock = example.lock.clone();
    ///
    /// let _ = thread::spawn(move || {
    ///     // This thread will acquire the mutex first, unwrapping the result
    ///     // of `lock` because the lock has not been poisoned.
    ///     let _guard = lock.lock().unwrap();
    ///
    ///     // This panic while holding the lock (`_guard` is in scope) will
    ///     // poison the mutex.
    ///     panic!()
    /// }).join();
    ///
    /// let expected = &[
    ///     Token::Struct("Example", 1),
    ///     Token::Str("lock"),
    /// ];
    /// let error = Error::Message("lock poison error while serializing".to_owned());
    /// assert_ser_tokens_error(&example, expected, error);
    /// # }
    /// ```
    Message(String),

    /// `Deserialize` was expecting a struct of one name, and another was found.
    InvalidName(&'static str),

    /// `Serialize` generated a token that didn't match the test.
    UnexpectedToken(Token),

    /// The expected token list was too short.
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
