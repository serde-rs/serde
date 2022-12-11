use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};

use de::Deserializer;
use ser::Serializer;
use token::Token;

use std::fmt::Debug;

/// Runs both `assert_ser_tokens` and `assert_de_tokens`.
///
/// ```edition2018
/// # use serde::{Serialize, Deserialize};
/// # use serde_test::{assert_tokens, Token};
/// #
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// let s = S { a: 0, b: 0 };
/// assert_tokens(&s, &[
///     Token::Struct { name: "S", len: 2 },
///     Token::Str("a"),
///     Token::U8(0),
///     Token::Str("b"),
///     Token::U8(0),
///     Token::StructEnd,
/// ]);
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
/// ```edition2018
/// # use serde::{Serialize, Deserialize};
/// # use serde_test::{assert_ser_tokens, Token};
/// #
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// let s = S { a: 0, b: 0 };
/// assert_ser_tokens(&s, &[
///     Token::Struct { name: "S", len: 2 },
///     Token::Str("a"),
///     Token::U8(0),
///     Token::Str("b"),
///     Token::U8(0),
///     Token::StructEnd,
/// ]);
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
/// ```edition2018
/// use std::sync::{Arc, Mutex};
/// use std::thread;
///
/// use serde::Serialize;
/// use serde_test::{assert_ser_tokens_error, Token};
///
/// #[derive(Serialize)]
/// struct Example {
///     lock: Arc<Mutex<u32>>,
/// }
///
/// fn main() {
///     let example = Example { lock: Arc::new(Mutex::new(0)) };
///     let lock = example.lock.clone();
///
///     let _ = thread::spawn(move || {
///         // This thread will acquire the mutex first, unwrapping the result
///         // of `lock` because the lock has not been poisoned.
///         let _guard = lock.lock().unwrap();
///
///         // This panic while holding the lock (`_guard` is in scope) will
///         // poison the mutex.
///         panic!()
///     }).join();
///
///     let expected = &[
///         Token::Struct { name: "Example", len: 1 },
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
/// ```edition2018
/// # use serde::{Serialize, Deserialize};
/// # use serde_test::{assert_de_tokens, Token};
/// #
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// let s = S { a: 0, b: 0 };
/// assert_de_tokens(&s, &[
///     Token::Struct { name: "S", len: 2 },
///     Token::Str("a"),
///     Token::U8(0),
///     Token::Str("b"),
///     Token::U8(0),
///     Token::StructEnd,
/// ]);
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
/// ```edition2018
/// # use serde::{Serialize, Deserialize};
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

/// Same as [`assert_de_tokens`], but for [`DeserializeSeed`].
///
/// ```edition2018
/// # use serde::{de::{DeserializeSeed, Visitor}, Deserializer};
/// # use serde_test::{assert_de_seed_tokens, Token};
/// 
/// #[derive(Debug, PartialEq)]
/// struct Example {
///     a: u8,
///     b: u8,
/// }
/// 
/// struct ExampleDeserializer(u8);
/// 
/// impl<'de> DeserializeSeed<'de> for ExampleDeserializer {
///     type Value = Example;
/// 
///     fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
///         deserializer.deserialize_u8(self)
///     }
/// }
/// 
/// impl<'de> Visitor<'de> for ExampleDeserializer {
///     type Value = Example;
/// 
///     fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
///         formatter.write_str("Example")
///     }
/// 
///     fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E> {
///         Ok(Self::Value { a: v, b: self.0 })
///     }
/// }
///
/// let example = Example { b: 0, a: 0 };
/// let seed = ExampleDeserializer(0);
/// assert_de_seed_tokens(&example, &[Token::U8(0)], seed);
/// ```
#[cfg_attr(not(no_track_caller), track_caller)]
pub fn assert_de_seed_tokens<'de, T, D>(value: &T, tokens: &'de [Token], seed: D)
where
    T: PartialEq + Debug,
    D: DeserializeSeed<'de, Value = T>,
{
    let mut de = Deserializer::new(tokens);
    match seed.deserialize(&mut de) {
        Ok(v) => assert_eq!(v, *value),
        Err(e) => panic!("tokens failed to deserialize: {}", e),
    };

    if de.remaining() > 0 {
        panic!("{} remaining tokens", de.remaining());
    }
}

/// Same as [`assert_de_tokens_error`], but for [`DeserializeSeed`].
///
/// ```edition2018
/// # use serde::{de::{DeserializeSeed, Visitor}, Deserializer};
/// # use serde_test::{assert_de_seed_tokens_error, Token};
/// 
/// #[derive(Debug, PartialEq)]
/// struct Example {
///     a: u8,
///     b: u8,
/// }
/// 
/// struct ExampleDeserializer(u8);
/// 
/// impl<'de> DeserializeSeed<'de> for ExampleDeserializer {
///     type Value = Example;
/// 
///     fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
///         deserializer.deserialize_u8(self)
///     }
/// }
/// 
/// impl<'de> Visitor<'de> for ExampleDeserializer {
///     type Value = Example;
/// 
///     fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
///         formatter.write_str("Example")
///     }
/// 
///     fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E> {
///         Ok(Self::Value { a: v, b: self.0 })
///     }
/// }
///
/// let seed = ExampleDeserializer(0);
/// assert_de_seed_tokens_error(
///     &[Token::I8(0)],
///     seed,
///     "invalid type: integer `0`, expected Example"
/// );
/// ```
#[cfg_attr(not(no_track_caller), track_caller)]
pub fn assert_de_seed_tokens_error<'de, T, D>(tokens: &'de [Token], seed: D, error: &str)
where
    T: PartialEq + Debug,
    D: DeserializeSeed<'de, Value = T>,
{
    let mut de = Deserializer::new(tokens);
    match seed.deserialize(&mut de) {
        Ok(_) => panic!("tokens deserialized successfully"),
        Err(e) => assert_eq!(e, *error),
    };

    // There may be one token left if a peek caused the error
    de.next_token_opt();

    if de.remaining() > 0 {
        panic!("{} remaining tokens", de.remaining());
    }
}
