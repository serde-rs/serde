#![allow(
    clippy::cast_lossless,
    clippy::decimal_literal_representation,
    clippy::derive_partial_eq_without_eq,
    clippy::empty_enum,
    clippy::manual_assert,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    clippy::unreadable_literal
)]
#![cfg_attr(feature = "unstable", feature(never_type))]

use fnv::FnvBuildHasher;
use serde::de::value::{F32Deserializer, F64Deserializer};
use serde::de::{Deserialize, DeserializeOwned, Deserializer, IntoDeserializer};
use serde_derive::Deserialize;
use serde_test::{assert_de_tokens, Configure, Token};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
use std::ffi::{CStr, CString, OsString};
use std::fmt::Debug;
use std::iter;
use std::net;
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Saturating, Wrapping,
};
use std::ops::Bound;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak as RcWeak};
use std::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI8, AtomicIsize, AtomicU16, AtomicU32, AtomicU8,
    AtomicUsize, Ordering,
};
#[cfg(target_arch = "x86_64")]
use std::sync::atomic::{AtomicI64, AtomicU64};
use std::sync::{Arc, Weak as ArcWeak};
use std::time::{Duration, UNIX_EPOCH};

#[macro_use]
mod macros;

//////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
struct UnitStruct;

#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
struct GenericUnitStruct<const N: u8>;

#[derive(PartialEq, Debug, Deserialize)]
struct NewtypeStruct(i32);

#[derive(PartialEq, Debug, Deserialize)]
struct TupleStruct(i32, i32, i32);

#[derive(PartialEq, Debug, Deserialize)]
struct Struct {
    a: i32,
    b: i32,
    #[serde(skip_deserializing)]
    c: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(default)]
struct StructDefault<T> {
    a: i32,
    b: T,
}

impl Default for StructDefault<String> {
    fn default() -> Self {
        StructDefault {
            a: 100,
            b: "default".to_string(),
        }
    }
}

#[derive(PartialEq, Debug, Deserialize)]
struct StructSkipAll {
    #[serde(skip_deserializing)]
    a: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(default)]
struct StructSkipDefault {
    #[serde(skip_deserializing)]
    a: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(default)]
pub struct StructSkipDefaultGeneric<T> {
    #[serde(skip_deserializing)]
    t: T,
}

impl Default for StructSkipDefault {
    fn default() -> Self {
        StructSkipDefault { a: 16 }
    }
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructSkipAllDenyUnknown {
    #[serde(skip_deserializing)]
    a: i32,
}

#[derive(Default, PartialEq, Debug)]
struct NotDeserializable;

#[derive(PartialEq, Debug, Deserialize)]
enum Enum {
    #[allow(dead_code)]
    #[serde(skip_deserializing)]
    Skipped,
    Unit,
    Simple(i32),
    Seq(i32, i32, i32),
    Map {
        a: i32,
        b: i32,
        c: i32,
    },
    SimpleWithSkipped(#[serde(skip_deserializing)] NotDeserializable),
}

#[derive(PartialEq, Debug, Deserialize)]
enum EnumOther {
    Unit,
    #[serde(other)]
    Other,
}

#[derive(PartialEq, Debug)]
struct IgnoredAny;

impl<'de> Deserialize<'de> for IgnoredAny {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde::de::IgnoredAny::deserialize(deserializer)?;
        Ok(IgnoredAny)
    }
}

//////////////////////////////////////////////////////////////////////////

#[track_caller]
fn test<'de, T>(value: T, tokens: &'de [Token])
where
    T: Deserialize<'de> + PartialEq + Debug,
{
    // Test ser/de roundtripping
    assert_de_tokens(&value, tokens);

    // Test that the tokens are ignorable
    assert_de_tokens_ignore(tokens);
}

#[derive(Debug)]
struct SkipPartialEq<T>(T);

impl<'de, T> Deserialize<'de> for SkipPartialEq<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(SkipPartialEq)
    }
}

impl<T> PartialEq for SkipPartialEq<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[track_caller]
fn assert_de_tokens_ignore(ignorable_tokens: &[Token]) {
    #[derive(PartialEq, Debug, Deserialize)]
    struct IgnoreBase {
        a: i32,
    }

    // Embed the tokens to be ignored in the normal token
    // stream for an IgnoreBase type
    let concated_tokens: Vec<Token> = vec![
        Token::Map { len: Some(2) },
        Token::Str("a"),
        Token::I32(1),
        Token::Str("ignored"),
    ]
    .into_iter()
    .chain(ignorable_tokens.iter().copied())
    .chain(iter::once(Token::MapEnd))
    .collect();

    let expected = IgnoreBase { a: 1 };
    assert_de_tokens(&expected, &concated_tokens);
}

//////////////////////////////////////////////////////////////////////////

#[test]
fn test_bool() {
    test(true, &[Token::Bool(true)]);
    test(false, &[Token::Bool(false)]);
}

#[test]
fn test_i8() {
    let test = test::<i8>;

    // from signed
    test(-128, &[Token::I8(-128)]);
    test(-128, &[Token::I16(-128)]);
    test(-128, &[Token::I32(-128)]);
    test(-128, &[Token::I64(-128)]);
    test(127, &[Token::I8(127)]);
    test(127, &[Token::I16(127)]);
    test(127, &[Token::I32(127)]);
    test(127, &[Token::I64(127)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(127, &[Token::U8(127)]);
    test(127, &[Token::U16(127)]);
    test(127, &[Token::U32(127)]);
    test(127, &[Token::U64(127)]);
}

#[test]
fn test_i16() {
    let test = test::<i16>;

    // from signed
    test(-128, &[Token::I8(-128)]);
    test(-32768, &[Token::I16(-32768)]);
    test(-32768, &[Token::I32(-32768)]);
    test(-32768, &[Token::I64(-32768)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(32767, &[Token::I32(32767)]);
    test(32767, &[Token::I64(32767)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(255, &[Token::U8(255)]);
    test(32767, &[Token::U16(32767)]);
    test(32767, &[Token::U32(32767)]);
    test(32767, &[Token::U64(32767)]);
}

#[test]
fn test_i32() {
    let test = test::<i32>;

    // from signed
    test(-128, &[Token::I8(-128)]);
    test(-32768, &[Token::I16(-32768)]);
    test(-2147483648, &[Token::I32(-2147483648)]);
    test(-2147483648, &[Token::I64(-2147483648)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(2147483647, &[Token::I64(2147483647)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(2147483647, &[Token::U32(2147483647)]);
    test(2147483647, &[Token::U64(2147483647)]);
}

#[test]
fn test_i64() {
    let test = test::<i64>;

    // from signed
    test(-128, &[Token::I8(-128)]);
    test(-32768, &[Token::I16(-32768)]);
    test(-2147483648, &[Token::I32(-2147483648)]);
    test(-9223372036854775808, &[Token::I64(-9223372036854775808)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(9223372036854775807, &[Token::I64(9223372036854775807)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(4294967295, &[Token::U32(4294967295)]);
    test(9223372036854775807, &[Token::U64(9223372036854775807)]);
}

#[test]
fn test_i128() {
    let test = test::<i128>;

    // from signed
    test(-128, &[Token::I8(-128)]);
    test(-32768, &[Token::I16(-32768)]);
    test(-2147483648, &[Token::I32(-2147483648)]);
    test(-9223372036854775808, &[Token::I64(-9223372036854775808)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(9223372036854775807, &[Token::I64(9223372036854775807)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(4294967295, &[Token::U32(4294967295)]);
    test(18446744073709551615, &[Token::U64(18446744073709551615)]);
}

#[test]
fn test_isize() {
    let test = test::<isize>;

    // from signed
    test(-10, &[Token::I8(-10)]);
    test(-10, &[Token::I16(-10)]);
    test(-10, &[Token::I32(-10)]);
    test(-10, &[Token::I64(-10)]);
    test(10, &[Token::I8(10)]);
    test(10, &[Token::I16(10)]);
    test(10, &[Token::I32(10)]);
    test(10, &[Token::I64(10)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(10, &[Token::U8(10)]);
    test(10, &[Token::U16(10)]);
    test(10, &[Token::U32(10)]);
    test(10, &[Token::U64(10)]);
}

#[test]
fn test_u8() {
    let test = test::<u8>;

    // from signed
    test(0, &[Token::I8(0)]);
    test(0, &[Token::I16(0)]);
    test(0, &[Token::I32(0)]);
    test(0, &[Token::I64(0)]);
    test(127, &[Token::I8(127)]);
    test(255, &[Token::I16(255)]);
    test(255, &[Token::I32(255)]);
    test(255, &[Token::I64(255)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(255, &[Token::U8(255)]);
    test(255, &[Token::U16(255)]);
    test(255, &[Token::U32(255)]);
    test(255, &[Token::U64(255)]);
}

#[test]
fn test_u16() {
    let test = test::<u16>;

    // from signed
    test(0, &[Token::I8(0)]);
    test(0, &[Token::I16(0)]);
    test(0, &[Token::I32(0)]);
    test(0, &[Token::I64(0)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(65535, &[Token::I32(65535)]);
    test(65535, &[Token::I64(65535)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(65535, &[Token::U32(65535)]);
    test(65535, &[Token::U64(65535)]);
}

#[test]
fn test_u32() {
    let test = test::<u32>;

    // from signed
    test(0, &[Token::I8(0)]);
    test(0, &[Token::I16(0)]);
    test(0, &[Token::I32(0)]);
    test(0, &[Token::I64(0)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(4294967295, &[Token::I64(4294967295)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(4294967295, &[Token::U32(4294967295)]);
    test(4294967295, &[Token::U64(4294967295)]);
}

#[test]
fn test_u64() {
    let test = test::<u64>;

    // from signed
    test(0, &[Token::I8(0)]);
    test(0, &[Token::I16(0)]);
    test(0, &[Token::I32(0)]);
    test(0, &[Token::I64(0)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(9223372036854775807, &[Token::I64(9223372036854775807)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(4294967295, &[Token::U32(4294967295)]);
    test(18446744073709551615, &[Token::U64(18446744073709551615)]);
}

#[test]
fn test_u128() {
    let test = test::<u128>;

    // from signed
    test(0, &[Token::I8(0)]);
    test(0, &[Token::I16(0)]);
    test(0, &[Token::I32(0)]);
    test(0, &[Token::I64(0)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(9223372036854775807, &[Token::I64(9223372036854775807)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(4294967295, &[Token::U32(4294967295)]);
    test(18446744073709551615, &[Token::U64(18446744073709551615)]);
}

#[test]
fn test_usize() {
    let test = test::<usize>;

    // from signed
    test(0, &[Token::I8(0)]);
    test(0, &[Token::I16(0)]);
    test(0, &[Token::I32(0)]);
    test(0, &[Token::I64(0)]);
    test(10, &[Token::I8(10)]);
    test(10, &[Token::I16(10)]);
    test(10, &[Token::I32(10)]);
    test(10, &[Token::I64(10)]);

    // from unsigned
    test(0, &[Token::U8(0)]);
    test(0, &[Token::U16(0)]);
    test(0, &[Token::U32(0)]);
    test(0, &[Token::U64(0)]);
    test(10, &[Token::U8(10)]);
    test(10, &[Token::U16(10)]);
    test(10, &[Token::U32(10)]);
    test(10, &[Token::U64(10)]);
}

#[test]
fn test_nonzero_i8() {
    let test = |value, tokens| test(NonZeroI8::new(value).unwrap(), tokens);

    // from signed
    test(-128, &[Token::I8(-128)]);
    test(-128, &[Token::I16(-128)]);
    test(-128, &[Token::I32(-128)]);
    test(-128, &[Token::I64(-128)]);
    test(127, &[Token::I8(127)]);
    test(127, &[Token::I16(127)]);
    test(127, &[Token::I32(127)]);
    test(127, &[Token::I64(127)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(127, &[Token::U8(127)]);
    test(127, &[Token::U16(127)]);
    test(127, &[Token::U32(127)]);
    test(127, &[Token::U64(127)]);
}

#[test]
fn test_nonzero_i16() {
    let test = |value, tokens| test(NonZeroI16::new(value).unwrap(), tokens);

    // from signed
    test(-128, &[Token::I8(-128)]);
    test(-32768, &[Token::I16(-32768)]);
    test(-32768, &[Token::I32(-32768)]);
    test(-32768, &[Token::I64(-32768)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(32767, &[Token::I32(32767)]);
    test(32767, &[Token::I64(32767)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(255, &[Token::U8(255)]);
    test(32767, &[Token::U16(32767)]);
    test(32767, &[Token::U32(32767)]);
    test(32767, &[Token::U64(32767)]);
}

#[test]
fn test_nonzero_i32() {
    let test = |value, tokens| test(NonZeroI32::new(value).unwrap(), tokens);

    // from signed
    test(-128, &[Token::I8(-128)]);
    test(-32768, &[Token::I16(-32768)]);
    test(-2147483648, &[Token::I32(-2147483648)]);
    test(-2147483648, &[Token::I64(-2147483648)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(2147483647, &[Token::I64(2147483647)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(2147483647, &[Token::U32(2147483647)]);
    test(2147483647, &[Token::U64(2147483647)]);
}

#[test]
fn test_nonzero_i64() {
    let test = |value, tokens| test(NonZeroI64::new(value).unwrap(), tokens);

    // from signed
    test(-128, &[Token::I8(-128)]);
    test(-32768, &[Token::I16(-32768)]);
    test(-2147483648, &[Token::I32(-2147483648)]);
    test(-9223372036854775808, &[Token::I64(-9223372036854775808)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(9223372036854775807, &[Token::I64(9223372036854775807)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(4294967295, &[Token::U32(4294967295)]);
    test(9223372036854775807, &[Token::U64(9223372036854775807)]);
}

#[test]
fn test_nonzero_i128() {
    let test = |value, tokens| test(NonZeroI128::new(value).unwrap(), tokens);

    // from signed
    test(-128, &[Token::I8(-128)]);
    test(-32768, &[Token::I16(-32768)]);
    test(-2147483648, &[Token::I32(-2147483648)]);
    test(-9223372036854775808, &[Token::I64(-9223372036854775808)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(9223372036854775807, &[Token::I64(9223372036854775807)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(4294967295, &[Token::U32(4294967295)]);
    test(18446744073709551615, &[Token::U64(18446744073709551615)]);
}

#[test]
fn test_nonzero_isize() {
    let test = |value, tokens| test(NonZeroIsize::new(value).unwrap(), tokens);

    // from signed
    test(-10, &[Token::I8(-10)]);
    test(-10, &[Token::I16(-10)]);
    test(-10, &[Token::I32(-10)]);
    test(-10, &[Token::I64(-10)]);
    test(10, &[Token::I8(10)]);
    test(10, &[Token::I16(10)]);
    test(10, &[Token::I32(10)]);
    test(10, &[Token::I64(10)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(10, &[Token::U8(10)]);
    test(10, &[Token::U16(10)]);
    test(10, &[Token::U32(10)]);
    test(10, &[Token::U64(10)]);
}

#[test]
fn test_nonzero_u8() {
    let test = |value, tokens| test(NonZeroU8::new(value).unwrap(), tokens);

    // from signed
    test(1, &[Token::I8(1)]);
    test(1, &[Token::I16(1)]);
    test(1, &[Token::I32(1)]);
    test(1, &[Token::I64(1)]);
    test(127, &[Token::I8(127)]);
    test(255, &[Token::I16(255)]);
    test(255, &[Token::I32(255)]);
    test(255, &[Token::I64(255)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(255, &[Token::U8(255)]);
    test(255, &[Token::U16(255)]);
    test(255, &[Token::U32(255)]);
    test(255, &[Token::U64(255)]);
}

#[test]
fn test_nonzero_u16() {
    let test = |value, tokens| test(NonZeroU16::new(value).unwrap(), tokens);

    // from signed
    test(1, &[Token::I8(1)]);
    test(1, &[Token::I16(1)]);
    test(1, &[Token::I32(1)]);
    test(1, &[Token::I64(1)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(65535, &[Token::I32(65535)]);
    test(65535, &[Token::I64(65535)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(65535, &[Token::U32(65535)]);
    test(65535, &[Token::U64(65535)]);
}

#[test]
fn test_nonzero_u32() {
    let test = |value, tokens| test(NonZeroU32::new(value).unwrap(), tokens);

    // from signed
    test(1, &[Token::I8(1)]);
    test(1, &[Token::I16(1)]);
    test(1, &[Token::I32(1)]);
    test(1, &[Token::I64(1)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(4294967295, &[Token::I64(4294967295)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(4294967295, &[Token::U32(4294967295)]);
    test(4294967295, &[Token::U64(4294967295)]);
}

#[test]
fn test_nonzero_u64() {
    let test = |value, tokens| test(NonZeroU64::new(value).unwrap(), tokens);

    // from signed
    test(1, &[Token::I8(1)]);
    test(1, &[Token::I16(1)]);
    test(1, &[Token::I32(1)]);
    test(1, &[Token::I64(1)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(9223372036854775807, &[Token::I64(9223372036854775807)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(4294967295, &[Token::U32(4294967295)]);
    test(18446744073709551615, &[Token::U64(18446744073709551615)]);
}

#[test]
fn test_nonzero_u128() {
    let test = |value, tokens| test(NonZeroU128::new(value).unwrap(), tokens);

    // from signed
    test(1, &[Token::I8(1)]);
    test(1, &[Token::I16(1)]);
    test(1, &[Token::I32(1)]);
    test(1, &[Token::I64(1)]);
    test(127, &[Token::I8(127)]);
    test(32767, &[Token::I16(32767)]);
    test(2147483647, &[Token::I32(2147483647)]);
    test(9223372036854775807, &[Token::I64(9223372036854775807)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(255, &[Token::U8(255)]);
    test(65535, &[Token::U16(65535)]);
    test(4294967295, &[Token::U32(4294967295)]);
    test(18446744073709551615, &[Token::U64(18446744073709551615)]);
}

#[test]
fn test_nonzero_usize() {
    let test = |value, tokens| test(NonZeroUsize::new(value).unwrap(), tokens);

    // from signed
    test(1, &[Token::I8(1)]);
    test(1, &[Token::I16(1)]);
    test(1, &[Token::I32(1)]);
    test(1, &[Token::I64(1)]);
    test(10, &[Token::I8(10)]);
    test(10, &[Token::I16(10)]);
    test(10, &[Token::I32(10)]);
    test(10, &[Token::I64(10)]);

    // from unsigned
    test(1, &[Token::U8(1)]);
    test(1, &[Token::U16(1)]);
    test(1, &[Token::U32(1)]);
    test(1, &[Token::U64(1)]);
    test(10, &[Token::U8(10)]);
    test(10, &[Token::U16(10)]);
    test(10, &[Token::U32(10)]);
    test(10, &[Token::U64(10)]);
}

#[test]
fn test_f32() {
    let test = test::<f32>;

    test(1.11, &[Token::F32(1.11)]);
    test(1.11, &[Token::F64(1.11)]);
}

#[test]
fn test_f64() {
    let test = test::<f64>;

    test(1.11f32 as f64, &[Token::F32(1.11)]);
    test(1.11, &[Token::F64(1.11)]);
}

#[test]
fn test_nan() {
    let f32_deserializer = F32Deserializer::<serde::de::value::Error>::new;
    let f64_deserializer = F64Deserializer::<serde::de::value::Error>::new;

    let pos_f32_nan = f32_deserializer(f32::NAN.copysign(1.0));
    let pos_f64_nan = f64_deserializer(f64::NAN.copysign(1.0));
    assert!(f32::deserialize(pos_f32_nan).unwrap().is_sign_positive());
    assert!(f32::deserialize(pos_f64_nan).unwrap().is_sign_positive());
    assert!(f64::deserialize(pos_f32_nan).unwrap().is_sign_positive());
    assert!(f64::deserialize(pos_f64_nan).unwrap().is_sign_positive());

    let neg_f32_nan = f32_deserializer(f32::NAN.copysign(-1.0));
    let neg_f64_nan = f64_deserializer(f64::NAN.copysign(-1.0));
    assert!(f32::deserialize(neg_f32_nan).unwrap().is_sign_negative());
    assert!(f32::deserialize(neg_f64_nan).unwrap().is_sign_negative());
    assert!(f64::deserialize(neg_f32_nan).unwrap().is_sign_negative());
    assert!(f64::deserialize(neg_f64_nan).unwrap().is_sign_negative());
}

#[test]
fn test_char() {
    test('a', &[Token::Char('a')]);
    test('a', &[Token::Str("a")]);
    test('a', &[Token::String("a")]);
}

#[test]
fn test_string() {
    test("abc".to_owned(), &[Token::Str("abc")]);
    test("abc".to_owned(), &[Token::String("abc")]);
    test("a".to_owned(), &[Token::Char('a')]);
}

#[test]
fn test_option() {
    test(None::<i32>, &[Token::Unit]);
    test(None::<i32>, &[Token::None]);
    test(Some(1), &[Token::Some, Token::I32(1)]);
}

#[test]
fn test_result() {
    test(
        Ok::<i32, i32>(0),
        &[
            Token::Enum { name: "Result" },
            Token::Str("Ok"),
            Token::I32(0),
        ],
    );
    test(
        Err::<i32, i32>(1),
        &[
            Token::Enum { name: "Result" },
            Token::Str("Err"),
            Token::I32(1),
        ],
    );
}

#[test]
fn test_unit() {
    test((), &[Token::Unit]);
}

#[test]
fn test_unit_struct() {
    test(UnitStruct, &[Token::Unit]);
    test(UnitStruct, &[Token::UnitStruct { name: "UnitStruct" }]);
}

#[test]
fn test_generic_unit_struct() {
    test(GenericUnitStruct::<8>, &[Token::Unit]);
    test(
        GenericUnitStruct::<8>,
        &[Token::UnitStruct {
            name: "GenericUnitStruct",
        }],
    );
}

#[test]
fn test_newtype_struct() {
    test(
        NewtypeStruct(1),
        &[
            Token::NewtypeStruct {
                name: "NewtypeStruct",
            },
            Token::I32(1),
        ],
    );
}

#[test]
fn test_tuple_struct() {
    test(
        TupleStruct(1, 2, 3),
        &[
            Token::Seq { len: Some(3) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
        ],
    );
    test(
        TupleStruct(1, 2, 3),
        &[
            Token::Seq { len: None },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
        ],
    );
    test(
        TupleStruct(1, 2, 3),
        &[
            Token::TupleStruct {
                name: "TupleStruct",
                len: 3,
            },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::TupleStructEnd,
        ],
    );
    test(
        TupleStruct(1, 2, 3),
        &[
            Token::TupleStruct {
                name: "TupleStruct",
                len: 3,
            },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::TupleStructEnd,
        ],
    );
}

#[test]
fn test_btreeset() {
    test(
        BTreeSet::<isize>::new(),
        &[Token::Seq { len: Some(0) }, Token::SeqEnd],
    );
    test(
        btreeset![btreeset![], btreeset![1], btreeset![2, 3]],
        &[
            Token::Seq { len: Some(3) },
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::Seq { len: Some(1) },
            Token::I32(1),
            Token::SeqEnd,
            Token::Seq { len: Some(2) },
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
            Token::SeqEnd,
        ],
    );
    test(
        BTreeSet::<isize>::new(),
        &[
            Token::TupleStruct {
                name: "Anything",
                len: 0,
            },
            Token::TupleStructEnd,
        ],
    );
}

#[test]
fn test_hashset() {
    test(
        HashSet::<isize>::new(),
        &[Token::Seq { len: Some(0) }, Token::SeqEnd],
    );
    test(
        hashset![1, 2, 3],
        &[
            Token::Seq { len: Some(3) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
        ],
    );
    test(
        HashSet::<isize>::new(),
        &[
            Token::TupleStruct {
                name: "Anything",
                len: 0,
            },
            Token::TupleStructEnd,
        ],
    );
    test(
        hashset![FnvBuildHasher; 1, 2, 3],
        &[
            Token::Seq { len: Some(3) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_vec() {
    test(
        Vec::<isize>::new(),
        &[Token::Seq { len: Some(0) }, Token::SeqEnd],
    );

    test(
        vec![vec![], vec![1], vec![2, 3]],
        &[
            Token::Seq { len: Some(3) },
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::Seq { len: Some(1) },
            Token::I32(1),
            Token::SeqEnd,
            Token::Seq { len: Some(2) },
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
            Token::SeqEnd,
        ],
    );
    test(
        Vec::<isize>::new(),
        &[
            Token::TupleStruct {
                name: "Anything",
                len: 0,
            },
            Token::TupleStructEnd,
        ],
    );
}

#[test]
fn test_array() {
    test([0; 0], &[Token::Seq { len: Some(0) }, Token::SeqEnd]);
    test([0; 0], &[Token::Tuple { len: 0 }, Token::TupleEnd]);
    test(
        ([0; 0], [1], [2, 3]),
        &[
            Token::Seq { len: Some(3) },
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::Seq { len: Some(1) },
            Token::I32(1),
            Token::SeqEnd,
            Token::Seq { len: Some(2) },
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
            Token::SeqEnd,
        ],
    );
    test(
        ([0; 0], [1], [2, 3]),
        &[
            Token::Tuple { len: 3 },
            Token::Tuple { len: 0 },
            Token::TupleEnd,
            Token::Tuple { len: 1 },
            Token::I32(1),
            Token::TupleEnd,
            Token::Tuple { len: 2 },
            Token::I32(2),
            Token::I32(3),
            Token::TupleEnd,
            Token::TupleEnd,
        ],
    );
    test(
        [0; 0],
        &[
            Token::TupleStruct {
                name: "Anything",
                len: 0,
            },
            Token::TupleStructEnd,
        ],
    );
}

#[test]
fn test_tuple() {
    test(
        (1,),
        &[Token::Seq { len: Some(1) }, Token::I32(1), Token::SeqEnd],
    );
    test(
        (1, 2, 3),
        &[
            Token::Seq { len: Some(3) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
        ],
    );
    test(
        (1,),
        &[Token::Tuple { len: 1 }, Token::I32(1), Token::TupleEnd],
    );
    test(
        (1, 2, 3),
        &[
            Token::Tuple { len: 3 },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn test_btreemap() {
    test(
        BTreeMap::<isize, isize>::new(),
        &[Token::Map { len: Some(0) }, Token::MapEnd],
    );
    test(
        btreemap![1 => 2],
        &[
            Token::Map { len: Some(1) },
            Token::I32(1),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    test(
        btreemap![1 => 2, 3 => 4],
        &[
            Token::Map { len: Some(2) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::I32(4),
            Token::MapEnd,
        ],
    );
    test(
        btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]],
        &[
            Token::Map { len: Some(2) },
            Token::I32(1),
            Token::Map { len: Some(0) },
            Token::MapEnd,
            Token::I32(2),
            Token::Map { len: Some(2) },
            Token::I32(3),
            Token::I32(4),
            Token::I32(5),
            Token::I32(6),
            Token::MapEnd,
            Token::MapEnd,
        ],
    );
    test(
        BTreeMap::<isize, isize>::new(),
        &[
            Token::Struct {
                name: "Anything",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_hashmap() {
    test(
        HashMap::<isize, isize>::new(),
        &[Token::Map { len: Some(0) }, Token::MapEnd],
    );
    test(
        hashmap![1 => 2],
        &[
            Token::Map { len: Some(1) },
            Token::I32(1),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    test(
        hashmap![1 => 2, 3 => 4],
        &[
            Token::Map { len: Some(2) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::I32(4),
            Token::MapEnd,
        ],
    );
    test(
        hashmap![1 => hashmap![], 2 => hashmap![3 => 4, 5 => 6]],
        &[
            Token::Map { len: Some(2) },
            Token::I32(1),
            Token::Map { len: Some(0) },
            Token::MapEnd,
            Token::I32(2),
            Token::Map { len: Some(2) },
            Token::I32(3),
            Token::I32(4),
            Token::I32(5),
            Token::I32(6),
            Token::MapEnd,
            Token::MapEnd,
        ],
    );
    test(
        HashMap::<isize, isize>::new(),
        &[
            Token::Struct {
                name: "Anything",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
    test(
        hashmap![FnvBuildHasher; 1 => 2, 3 => 4],
        &[
            Token::Map { len: Some(2) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::I32(4),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_struct() {
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("b"),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::U8(0),
            Token::I32(1),
            Token::U8(1),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::U16(0),
            Token::I32(1),
            Token::U16(1),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::U32(0),
            Token::I32(1),
            Token::U32(1),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::U64(0),
            Token::I32(1),
            Token::U64(1),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    // Mixed key types
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::U8(0),
            Token::I32(1),
            Token::U64(1),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::U8(0),
            Token::I32(1),
            Token::Str("b"),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Struct {
                name: "Struct",
                len: 2,
            },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("b"),
            Token::I32(2),
            Token::StructEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Seq { len: Some(3) },
            Token::I32(1),
            Token::I32(2),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_struct_borrowed_keys() {
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::BorrowedStr("a"),
            Token::I32(1),
            Token::BorrowedStr("b"),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Struct {
                name: "Struct",
                len: 2,
            },
            Token::BorrowedStr("a"),
            Token::I32(1),
            Token::BorrowedStr("b"),
            Token::I32(2),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_struct_owned_keys() {
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::String("a"),
            Token::I32(1),
            Token::String("b"),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Struct {
                name: "Struct",
                len: 2,
            },
            Token::String("a"),
            Token::I32(1),
            Token::String("b"),
            Token::I32(2),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_struct_with_skip() {
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("b"),
            Token::I32(2),
            Token::Str("c"),
            Token::I32(3),
            Token::Str("d"),
            Token::I32(4),
            Token::MapEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Map { len: Some(3) },
            Token::U8(0),
            Token::I32(1),
            Token::U16(1),
            Token::I32(2),
            Token::U32(2),
            Token::I32(3),
            Token::U64(3),
            Token::I32(4),
            Token::MapEnd,
        ],
    );
    test(
        Struct { a: 1, b: 2, c: 0 },
        &[
            Token::Struct {
                name: "Struct",
                len: 2,
            },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("b"),
            Token::I32(2),
            Token::Str("c"),
            Token::I32(3),
            Token::Str("d"),
            Token::I32(4),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_struct_skip_all() {
    test(
        StructSkipAll { a: 0 },
        &[
            Token::Struct {
                name: "StructSkipAll",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
    test(
        StructSkipAll { a: 0 },
        &[
            Token::Struct {
                name: "StructSkipAll",
                len: 0,
            },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("b"),
            Token::I32(2),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_struct_skip_default() {
    test(
        StructSkipDefault { a: 16 },
        &[
            Token::Struct {
                name: "StructSkipDefault",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_struct_skip_all_deny_unknown() {
    test(
        StructSkipAllDenyUnknown { a: 0 },
        &[
            Token::Struct {
                name: "StructSkipAllDenyUnknown",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_struct_default() {
    test(
        StructDefault {
            a: 50,
            b: "overwritten".to_string(),
        },
        &[
            Token::Struct {
                name: "StructDefault",
                len: 2,
            },
            Token::Str("a"),
            Token::I32(50),
            Token::Str("b"),
            Token::String("overwritten"),
            Token::StructEnd,
        ],
    );
    test(
        StructDefault {
            a: 100,
            b: "default".to_string(),
        },
        &[
            Token::Struct {
                name: "StructDefault",
                len: 2,
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_enum_unit() {
    test(
        Enum::Unit,
        &[Token::UnitVariant {
            name: "Enum",
            variant: "Unit",
        }],
    );
}

#[test]
fn test_enum_simple() {
    test(
        Enum::Simple(1),
        &[
            Token::NewtypeVariant {
                name: "Enum",
                variant: "Simple",
            },
            Token::I32(1),
        ],
    );
}

#[test]
fn test_enum_simple_with_skipped() {
    test(
        Enum::SimpleWithSkipped(NotDeserializable),
        &[Token::UnitVariant {
            name: "Enum",
            variant: "SimpleWithSkipped",
        }],
    );
}

#[test]
fn test_enum_seq() {
    test(
        Enum::Seq(1, 2, 3),
        &[
            Token::TupleVariant {
                name: "Enum",
                variant: "Seq",
                len: 3,
            },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::TupleVariantEnd,
        ],
    );
}

#[test]
fn test_enum_map() {
    test(
        Enum::Map { a: 1, b: 2, c: 3 },
        &[
            Token::StructVariant {
                name: "Enum",
                variant: "Map",
                len: 3,
            },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("b"),
            Token::I32(2),
            Token::Str("c"),
            Token::I32(3),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_enum_unit_usize() {
    test(
        Enum::Unit,
        &[Token::Enum { name: "Enum" }, Token::U32(0), Token::Unit],
    );
}

#[test]
fn test_enum_unit_bytes() {
    test(
        Enum::Unit,
        &[
            Token::Enum { name: "Enum" },
            Token::Bytes(b"Unit"),
            Token::Unit,
        ],
    );
}

#[test]
fn test_enum_other_unit() {
    test(
        EnumOther::Unit,
        &[
            Token::Enum { name: "EnumOther" },
            Token::Str("Unit"),
            Token::Unit,
        ],
    );
    test(
        EnumOther::Unit,
        &[Token::Enum { name: "EnumOther" }, Token::U8(0), Token::Unit],
    );
    test(
        EnumOther::Unit,
        &[
            Token::Enum { name: "EnumOther" },
            Token::U16(0),
            Token::Unit,
        ],
    );
    test(
        EnumOther::Unit,
        &[
            Token::Enum { name: "EnumOther" },
            Token::U32(0),
            Token::Unit,
        ],
    );
    test(
        EnumOther::Unit,
        &[
            Token::Enum { name: "EnumOther" },
            Token::U64(0),
            Token::Unit,
        ],
    );
}

#[test]
fn test_enum_other() {
    test(
        EnumOther::Other,
        &[
            Token::Enum { name: "EnumOther" },
            Token::Str("Foo"),
            Token::Unit,
        ],
    );
    test(
        EnumOther::Other,
        &[
            Token::Enum { name: "EnumOther" },
            Token::U8(42),
            Token::Unit,
        ],
    );
    test(
        EnumOther::Other,
        &[
            Token::Enum { name: "EnumOther" },
            Token::U16(42),
            Token::Unit,
        ],
    );
    test(
        EnumOther::Other,
        &[
            Token::Enum { name: "EnumOther" },
            Token::U32(42),
            Token::Unit,
        ],
    );
    test(
        EnumOther::Other,
        &[
            Token::Enum { name: "EnumOther" },
            Token::U64(42),
            Token::Unit,
        ],
    );
}

#[test]
fn test_box() {
    test(Box::new(0i32), &[Token::I32(0)]);
}

#[test]
fn test_boxed_slice() {
    test(
        Box::new([0, 1, 2]),
        &[
            Token::Seq { len: Some(3) },
            Token::I32(0),
            Token::I32(1),
            Token::I32(2),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_duration() {
    test(
        Duration::new(1, 2),
        &[
            Token::Struct {
                name: "Duration",
                len: 2,
            },
            Token::Str("secs"),
            Token::U64(1),
            Token::Str("nanos"),
            Token::U32(2),
            Token::StructEnd,
        ],
    );
    test(
        Duration::new(1, 2),
        &[
            Token::Seq { len: Some(2) },
            Token::I64(1),
            Token::I64(2),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_system_time() {
    test(
        UNIX_EPOCH + Duration::new(1, 2),
        &[
            Token::Struct {
                name: "SystemTime",
                len: 2,
            },
            Token::Str("secs_since_epoch"),
            Token::U64(1),
            Token::Str("nanos_since_epoch"),
            Token::U32(2),
            Token::StructEnd,
        ],
    );
    test(
        UNIX_EPOCH + Duration::new(1, 2),
        &[
            Token::Seq { len: Some(2) },
            Token::I64(1),
            Token::I64(2),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_range() {
    test(
        1u32..2u32,
        &[
            Token::Struct {
                name: "Range",
                len: 2,
            },
            Token::Str("start"),
            Token::U32(1),
            Token::Str("end"),
            Token::U32(2),
            Token::StructEnd,
        ],
    );
    test(
        1u32..2u32,
        &[
            Token::Seq { len: Some(2) },
            Token::U64(1),
            Token::U64(2),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_range_inclusive() {
    test(
        1u32..=2u32,
        &[
            Token::Struct {
                name: "RangeInclusive",
                len: 2,
            },
            Token::Str("start"),
            Token::U32(1),
            Token::Str("end"),
            Token::U32(2),
            Token::StructEnd,
        ],
    );
    test(
        1u32..=2u32,
        &[
            Token::Seq { len: Some(2) },
            Token::U64(1),
            Token::U64(2),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_range_from() {
    test(
        1u32..,
        &[
            Token::Struct {
                name: "RangeFrom",
                len: 1,
            },
            Token::Str("start"),
            Token::U32(1),
            Token::StructEnd,
        ],
    );
    test(
        1u32..,
        &[Token::Seq { len: Some(1) }, Token::U32(1), Token::SeqEnd],
    );
}

#[test]
fn test_range_to() {
    test(
        ..2u32,
        &[
            Token::Struct {
                name: "RangeTo",
                len: 1,
            },
            Token::Str("end"),
            Token::U32(2),
            Token::StructEnd,
        ],
    );
    test(
        ..2u32,
        &[Token::Seq { len: Some(1) }, Token::U32(2), Token::SeqEnd],
    );
}

#[test]
fn test_bound() {
    test(
        Bound::Unbounded::<()>,
        &[
            Token::Enum { name: "Bound" },
            Token::Str("Unbounded"),
            Token::Unit,
        ],
    );
    test(
        Bound::Included(0),
        &[
            Token::Enum { name: "Bound" },
            Token::Str("Included"),
            Token::U8(0),
        ],
    );
    test(
        Bound::Excluded(0),
        &[
            Token::Enum { name: "Bound" },
            Token::Str("Excluded"),
            Token::U8(0),
        ],
    );
}

#[test]
fn test_path() {
    test(
        Path::new("/usr/local/lib"),
        &[Token::BorrowedStr("/usr/local/lib")],
    );
    test(
        Path::new("/usr/local/lib"),
        &[Token::BorrowedBytes(b"/usr/local/lib")],
    );
}

#[test]
fn test_path_buf() {
    test(
        PathBuf::from("/usr/local/lib"),
        &[Token::Str("/usr/local/lib")],
    );
    test(
        PathBuf::from("/usr/local/lib"),
        &[Token::String("/usr/local/lib")],
    );
    test(
        PathBuf::from("/usr/local/lib"),
        &[Token::Bytes(b"/usr/local/lib")],
    );
    test(
        PathBuf::from("/usr/local/lib"),
        &[Token::ByteBuf(b"/usr/local/lib")],
    );
}

#[test]
fn test_boxed_path() {
    test(
        PathBuf::from("/usr/local/lib").into_boxed_path(),
        &[Token::Str("/usr/local/lib")],
    );
    test(
        PathBuf::from("/usr/local/lib").into_boxed_path(),
        &[Token::String("/usr/local/lib")],
    );
    test(
        PathBuf::from("/usr/local/lib").into_boxed_path(),
        &[Token::Bytes(b"/usr/local/lib")],
    );
    test(
        PathBuf::from("/usr/local/lib").into_boxed_path(),
        &[Token::ByteBuf(b"/usr/local/lib")],
    );
}

#[test]
fn test_cstring() {
    test(CString::new("abc").unwrap(), &[Token::Bytes(b"abc")]);
}

#[test]
fn test_rc() {
    test(Rc::new(true), &[Token::Bool(true)]);
}

#[test]
fn test_rc_weak_some() {
    test(
        SkipPartialEq(RcWeak::<bool>::new()),
        &[Token::Some, Token::Bool(true)],
    );
}

#[test]
fn test_rc_weak_none() {
    test(SkipPartialEq(RcWeak::<bool>::new()), &[Token::None]);
}

#[test]
fn test_arc() {
    test(Arc::new(true), &[Token::Bool(true)]);
}

#[test]
fn test_arc_weak_some() {
    test(
        SkipPartialEq(ArcWeak::<bool>::new()),
        &[Token::Some, Token::Bool(true)],
    );
}

#[test]
fn test_arc_weak_none() {
    test(SkipPartialEq(ArcWeak::<bool>::new()), &[Token::None]);
}

#[test]
fn test_wrapping() {
    test(Wrapping(1usize), &[Token::U32(1)]);
    test(Wrapping(1usize), &[Token::U64(1)]);
}

#[test]
fn test_saturating() {
    test(Saturating(1usize), &[Token::U32(1)]);
    test(Saturating(1usize), &[Token::U64(1)]);
    test(Saturating(0u8), &[Token::I8(0)]);
    test(Saturating(0u16), &[Token::I16(0)]);

    // saturate input values at the minimum or maximum value
    test(Saturating(u8::MAX), &[Token::U16(u16::MAX)]);
    test(Saturating(u8::MAX), &[Token::U16(u8::MAX as u16 + 1)]);
    test(Saturating(u16::MAX), &[Token::U32(u32::MAX)]);
    test(Saturating(u32::MAX), &[Token::U64(u64::MAX)]);
    test(Saturating(u8::MIN), &[Token::I8(i8::MIN)]);
    test(Saturating(u16::MIN), &[Token::I16(i16::MIN)]);
    test(Saturating(u32::MIN), &[Token::I32(i32::MIN)]);
    test(Saturating(i8::MIN), &[Token::I16(i16::MIN)]);
    test(Saturating(i16::MIN), &[Token::I32(i32::MIN)]);
    test(Saturating(i32::MIN), &[Token::I64(i64::MIN)]);

    test(Saturating(u8::MIN), &[Token::I8(-1)]);
    test(Saturating(u16::MIN), &[Token::I16(-1)]);

    #[cfg(target_pointer_width = "64")]
    {
        test(Saturating(usize::MIN), &[Token::U64(u64::MIN)]);
        test(Saturating(usize::MAX), &[Token::U64(u64::MAX)]);
        test(Saturating(isize::MIN), &[Token::I64(i64::MIN)]);
        test(Saturating(isize::MAX), &[Token::I64(i64::MAX)]);
        test(Saturating(0usize), &[Token::I64(i64::MIN)]);

        test(
            Saturating(9_223_372_036_854_775_807usize),
            &[Token::I64(i64::MAX)],
        );
    }
}

#[test]
fn test_rc_dst() {
    test(Rc::<str>::from("s"), &[Token::Str("s")]);
    test(
        Rc::<[bool]>::from(&[true][..]),
        &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_arc_dst() {
    test(Arc::<str>::from("s"), &[Token::Str("s")]);
    test(
        Arc::<[bool]>::from(&[true][..]),
        &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_ignored_any() {
    test(IgnoredAny, &[Token::Str("s")]);
    test(
        IgnoredAny,
        &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    );
    test(
        IgnoredAny,
        &[Token::Enum { name: "E" }, Token::Str("Rust"), Token::Unit],
    );
}

#[test]
fn test_net_ipv4addr_readable() {
    test(
        "1.2.3.4".parse::<net::Ipv4Addr>().unwrap().readable(),
        &[Token::Str("1.2.3.4")],
    );
}

#[test]
fn test_net_ipv6addr_readable() {
    test(
        "::1".parse::<net::Ipv6Addr>().unwrap().readable(),
        &[Token::Str("::1")],
    );
}

#[test]
fn test_net_ipaddr_readable() {
    test(
        "1.2.3.4".parse::<net::IpAddr>().unwrap().readable(),
        &[Token::Str("1.2.3.4")],
    );
}

#[test]
fn test_net_socketaddr_readable() {
    test(
        "1.2.3.4:1234"
            .parse::<net::SocketAddr>()
            .unwrap()
            .readable(),
        &[Token::Str("1.2.3.4:1234")],
    );
    test(
        "1.2.3.4:1234"
            .parse::<net::SocketAddrV4>()
            .unwrap()
            .readable(),
        &[Token::Str("1.2.3.4:1234")],
    );
    test(
        "[::1]:1234"
            .parse::<net::SocketAddrV6>()
            .unwrap()
            .readable(),
        &[Token::Str("[::1]:1234")],
    );
}

#[test]
fn test_net_ipv4addr_compact() {
    test(
        net::Ipv4Addr::from(*b"1234").compact(),
        &seq![
            Token::Tuple { len: 4 },
            b"1234".iter().copied().map(Token::U8),
            Token::TupleEnd
        ],
    );
}

#[test]
fn test_net_ipv6addr_compact() {
    test(
        net::Ipv6Addr::from(*b"1234567890123456").compact(),
        &seq![
            Token::Tuple { len: 4 },
            b"1234567890123456".iter().copied().map(Token::U8),
            Token::TupleEnd
        ],
    );
}

#[test]
fn test_net_ipaddr_compact() {
    test(
        net::IpAddr::from(*b"1234").compact(),
        &seq![
            Token::NewtypeVariant {
                name: "IpAddr",
                variant: "V4"
            },
            Token::Tuple { len: 4 },
            b"1234".iter().copied().map(Token::U8),
            Token::TupleEnd
        ],
    );
}

#[test]
fn test_net_socketaddr_compact() {
    test(
        net::SocketAddr::from((*b"1234567890123456", 1234)).compact(),
        &seq![
            Token::NewtypeVariant {
                name: "SocketAddr",
                variant: "V6"
            },
            Token::Tuple { len: 2 },
            Token::Tuple { len: 16 },
            b"1234567890123456".iter().copied().map(Token::U8),
            Token::TupleEnd,
            Token::U16(1234),
            Token::TupleEnd
        ],
    );
    test(
        net::SocketAddr::from((*b"1234", 1234)).compact(),
        &seq![
            Token::NewtypeVariant {
                name: "SocketAddr",
                variant: "V4"
            },
            Token::Tuple { len: 2 },
            Token::Tuple { len: 4 },
            b"1234".iter().copied().map(Token::U8),
            Token::TupleEnd,
            Token::U16(1234),
            Token::TupleEnd
        ],
    );
    test(
        net::SocketAddrV4::new(net::Ipv4Addr::from(*b"1234"), 1234).compact(),
        &seq![
            Token::Tuple { len: 2 },
            Token::Tuple { len: 4 },
            b"1234".iter().copied().map(Token::U8),
            Token::TupleEnd,
            Token::U16(1234),
            Token::TupleEnd
        ],
    );
    test(
        net::SocketAddrV6::new(net::Ipv6Addr::from(*b"1234567890123456"), 1234, 0, 0).compact(),
        &seq![
            Token::Tuple { len: 2 },
            Token::Tuple { len: 16 },
            b"1234567890123456".iter().copied().map(Token::U8),
            Token::TupleEnd,
            Token::U16(1234),
            Token::TupleEnd
        ],
    );
}

#[cfg(feature = "unstable")]
#[test]
fn test_never_result() {
    test(
        Ok::<u8, !>(0),
        &[
            Token::NewtypeVariant {
                name: "Result",
                variant: "Ok",
            },
            Token::U8(0),
        ],
    );
}

#[cfg(unix)]
#[test]
fn test_osstring() {
    use std::os::unix::ffi::OsStringExt;

    let value = OsString::from_vec(vec![1, 2, 3]);
    let tokens = [
        Token::Enum { name: "OsString" },
        Token::Str("Unix"),
        Token::Seq { len: Some(2) },
        Token::U8(1),
        Token::U8(2),
        Token::U8(3),
        Token::SeqEnd,
    ];

    assert_de_tokens(&value, &tokens);
    assert_de_tokens_ignore(&tokens);
}

#[cfg(windows)]
#[test]
fn test_osstring() {
    use std::os::windows::ffi::OsStringExt;

    let value = OsString::from_wide(&[1, 2, 3]);
    let tokens = [
        Token::Enum { name: "OsString" },
        Token::Str("Windows"),
        Token::Seq { len: Some(2) },
        Token::U16(1),
        Token::U16(2),
        Token::U16(3),
        Token::SeqEnd,
    ];

    assert_de_tokens(&value, &tokens);
    assert_de_tokens_ignore(&tokens);
}

#[test]
fn test_cstr() {
    assert_de_tokens::<Box<CStr>>(
        &CString::new("abc").unwrap().into_boxed_c_str(),
        &[Token::Bytes(b"abc")],
    );
}

#[test]
fn test_atomics() {
    fn test<L, A, T>(load: L, val: T)
    where
        L: Fn(&A, Ordering) -> T,
        A: DeserializeOwned,
        T: PartialEq + Debug + Copy + for<'de> IntoDeserializer<'de>,
    {
        match A::deserialize(val.into_deserializer()) {
            Ok(v) => {
                let loaded = load(&v, Ordering::Relaxed);
                assert_eq!(val, loaded);
            }
            Err(e) => panic!("tokens failed to deserialize: {}", e),
        }
    }

    test(AtomicBool::load, true);
    test(AtomicI8::load, -127i8);
    test(AtomicI16::load, -510i16);
    test(AtomicI32::load, -131072i32);
    test(AtomicIsize::load, -131072isize);
    test(AtomicU8::load, 127u8);
    test(AtomicU16::load, 510u16);
    test(AtomicU32::load, 131072u32);
    test(AtomicUsize::load, 131072usize);

    #[cfg(target_arch = "x86_64")]
    {
        test(AtomicI64::load, -8589934592i64);
        test(AtomicU64::load, 8589934592u64);
    }
}
