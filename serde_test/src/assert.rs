use serde::{Deserialize, Serialize};

use de::Deserializer;
use ser::Serializer;
use token::Token;

use std::fmt::Debug;

/// Runs both `assert_ser_tokens` and `assert_de_tokens`.
///
/// ```edition2021
/// # use serde_derive::{Deserialize, Serialize};
/// # use serde_test::{assert_tokens, Token};
/// #
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// let s = S { a: 0, b: 0 };
/// assert_tokens(
///     &s,
///     &[
///         Token::Struct { name: "S", len: 2 },
///         Token::Str("a"),
///         Token::U8(0),
///         Token::Str("b"),
///         Token::U8(0),
///         Token::StructEnd,
///     ],
/// );
/// ```
#[cfg_attr(not(no_track_caller), track_caller)]
pub fn assert_tokens<'de, T>(value: &T, tokens: &'de [Token])
where
    T: Serialize + Deserialize<'de> + PartialEq + Debug,
{
    assert_ser_tokens(value, tokens);
    assert_de_tokens(value, tokens);
}

/// Asserts that `value` serializes to the given `tokens`.
///
/// ```edition2021
/// # use serde_derive::{Deserialize, Serialize};
/// # use serde_test::{assert_ser_tokens, Token};
/// #
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// let s = S { a: 0, b: 0 };
/// assert_ser_tokens(
///     &s,
///     &[
///         Token::Struct { name: "S", len: 2 },
///         Token::Str("a"),
///         Token::U8(0),
///         Token::Str("b"),
///         Token::U8(0),
///         Token::StructEnd,
///     ],
/// );
/// ```
#[cfg_attr(not(no_track_caller), track_caller)]
pub fn assert_ser_tokens<T: ?Sized>(value: &T, tokens: &[Token])
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

/// Asserts that `value` serializes to the given `tokens`, and then yields
/// `error`.
///
/// ```edition2021
/// use serde_derive::Serialize;
/// use serde_test::{assert_ser_tokens_error, Token};
/// use std::sync::{Arc, Mutex};
/// use std::thread;
///
/// #[derive(Serialize)]
/// struct Example {
///     lock: Arc<Mutex<u32>>,
/// }
///
/// fn main() {
///     let example = Example {
///         lock: Arc::new(Mutex::new(0)),
///     };
///     let lock = example.lock.clone();
///
///     let thread = thread::spawn(move || {
///         // This thread will acquire the mutex first, unwrapping the result
///         // of `lock` because the lock has not been poisoned.
///         let _guard = lock.lock().unwrap();
///
///         // This panic while holding the lock (`_guard` is in scope) will
///         // poison the mutex.
///         panic!()
///     });
///     thread.join();
///
///     let expected = &[
///         Token::Struct {
///             name: "Example",
///             len: 1,
///         },
///         Token::Str("lock"),
///     ];
///     let error = "lock poison error while serializing";
///     assert_ser_tokens_error(&example, expected, error);
/// }
/// ```
#[cfg_attr(not(no_track_caller), track_caller)]
pub fn assert_ser_tokens_error<T: ?Sized>(value: &T, tokens: &[Token], error: &str)
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
///
/// ```edition2021
/// # use serde_derive::{Deserialize, Serialize};
/// # use serde_test::{assert_de_tokens, Token};
/// #
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// let s = S { a: 0, b: 0 };
/// assert_de_tokens(
///     &s,
///     &[
///         Token::Struct { name: "S", len: 2 },
///         Token::Str("a"),
///         Token::U8(0),
///         Token::Str("b"),
///         Token::U8(0),
///         Token::StructEnd,
///     ],
/// );
/// ```
#[cfg_attr(not(no_track_caller), track_caller)]
pub fn assert_de_tokens<'de, T>(value: &T, tokens: &'de [Token])
where
    T: Deserialize<'de> + PartialEq + Debug,
{
    let mut de = Deserializer::new(tokens);
    let mut deserialized_val = match T::deserialize(&mut de) {
        Ok(v) => {
            assert_eq!(v, *value);
            v
        }
        Err(e) => panic!("tokens failed to deserialize: {}", e),
    };
    if de.remaining() > 0 {
        panic!("{} remaining tokens", de.remaining());
    }

    // Do the same thing for deserialize_in_place. This isn't *great* because a
    // no-op impl of deserialize_in_place can technically succeed here. Still,
    // this should catch a lot of junk.
    let mut de = Deserializer::new(tokens);
    match T::deserialize_in_place(&mut de, &mut deserialized_val) {
        Ok(()) => {
            assert_eq!(deserialized_val, *value);
        }
        Err(e) => panic!("tokens failed to deserialize_in_place: {}", e),
    }
    if de.remaining() > 0 {
        panic!("{} remaining tokens", de.remaining());
    }
}

/// Asserts that the given `tokens` yield `error` when deserializing.
///
/// ```edition2021
/// # use serde_derive::{Deserialize, Serialize};
/// # use serde_test::{assert_de_tokens_error, Token};
/// #
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// assert_de_tokens_error::<S>(
///     &[
///         Token::Struct { name: "S", len: 2 },
///         Token::Str("x"),
///     ],
///     "unknown field `x`, expected `a` or `b`",
/// );
/// ```
#[cfg_attr(not(no_track_caller), track_caller)]
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
