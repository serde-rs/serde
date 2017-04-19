// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::{Serialize, Deserialize};

use de::Deserializer;
use ser::Serializer;
use token::Token;

use std::fmt::Debug;

/// Runs both `assert_ser_tokens` and `assert_de_tokens`.
pub fn assert_tokens<'de, T>(value: &T, tokens: &'de [Token])
where
    T: Serialize + Deserialize<'de> + PartialEq + Debug,
{
    assert_ser_tokens(value, tokens);
    assert_de_tokens(value, tokens);
}

/// Asserts that `value` serializes to the given `tokens`.
pub fn assert_ser_tokens<T>(value: &T, tokens: &[Token])
where
    T: Serialize,
{
    let mut ser = Serializer::new(tokens);
    match value.serialize(&mut ser) {
        Ok(_) => {}
        Err(err) => panic!("value failed to serialize: {}", err),
    }

    if ser.remaining() > 0 {
        panic!("{} remaining tokens", ser.remaining());
    }
}

/// Asserts that `value` serializes to the given `tokens`, and then yields `error`.
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
pub fn assert_ser_tokens_error<T>(value: &T, tokens: &[Token], error: &str)
where
    T: Serialize,
{
    let mut ser = Serializer::new(tokens);
    match value.serialize(&mut ser) {
        Ok(_) => panic!("value serialized successfully"),
        Err(e) => assert_eq!(e, *error),
    }

    if ser.remaining() > 0 {
        panic!("{} remaining tokens", ser.remaining());
    }
}

/// Asserts that the given `tokens` deserialize into `value`.
pub fn assert_de_tokens<'de, T>(value: &T, tokens: &'de [Token])
where
    T: Deserialize<'de> + PartialEq + Debug,
{
    let mut de = Deserializer::new(tokens);
    match T::deserialize(&mut de) {
        Ok(v) => assert_eq!(v, *value),
        Err(e) => panic!("tokens failed to deserialize: {}", e),
    }

    if de.remaining() > 0 {
        panic!("{} remaining tokens", de.remaining());
    }
}

/// Asserts that the given `tokens` yield `error` when deserializing.
pub fn assert_de_tokens_error<'de, T>(tokens: &'de [Token], error: &str)
where
    T: Deserialize<'de>,
{
    let mut de = Deserializer::new(tokens);
    match T::deserialize(&mut de) {
        Ok(_) => panic!("tokens deserialized successfully"),
        Err(e) => assert_eq!(e, *error),
    }

    // There may be one token left if a peek caused the error
    de.next_token_opt();

    if de.remaining() > 0 {
        panic!("{} remaining tokens", de.remaining());
    }
}
