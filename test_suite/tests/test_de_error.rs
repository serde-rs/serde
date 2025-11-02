#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::empty_enums,
    clippy::unreadable_literal
)]
#![cfg_attr(feature = "unstable", feature(never_type))]

use serde::de::{Deserialize, IntoDeserializer};
use serde_derive::Deserialize;
use serde_test::{assert_de_tokens_error, Token};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping,
};
use std::time::{Duration, SystemTime};

#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
struct UnitStruct;

#[derive(PartialEq, Debug, Deserialize)]
struct Struct {
    a: i32,
    b: i32,
    #[serde(skip_deserializing)]
    c: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructDenyUnknown {
    a: i32,
    #[serde(skip_deserializing)]
    b: i32,
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
enum EnumSkipAll {
    #[allow(dead_code)]
    #[serde(skip_deserializing)]
    Skipped,
}

#[test]
fn test_i8() {
    let test = assert_de_tokens_error::<i8>;

    // from signed
    test(
        &[Token::I16(-129)],
        "invalid value: integer `-129`, expected i8",
    );
    test(
        &[Token::I32(-129)],
        "invalid value: integer `-129`, expected i8",
    );
    test(
        &[Token::I64(-129)],
        "invalid value: integer `-129`, expected i8",
    );
    test(
        &[Token::I16(128)],
        "invalid value: integer `128`, expected i8",
    );
    test(
        &[Token::I32(128)],
        "invalid value: integer `128`, expected i8",
    );
    test(
        &[Token::I64(128)],
        "invalid value: integer `128`, expected i8",
    );

    // from unsigned
    test(
        &[Token::U8(128)],
        "invalid value: integer `128`, expected i8",
    );
    test(
        &[Token::U16(128)],
        "invalid value: integer `128`, expected i8",
    );
    test(
        &[Token::U32(128)],
        "invalid value: integer `128`, expected i8",
    );
    test(
        &[Token::U64(128)],
        "invalid value: integer `128`, expected i8",
    );
}

#[test]
fn test_i16() {
    let test = assert_de_tokens_error::<i16>;

    // from signed
    test(
        &[Token::I32(-32769)],
        "invalid value: integer `-32769`, expected i16",
    );
    test(
        &[Token::I64(-32769)],
        "invalid value: integer `-32769`, expected i16",
    );
    test(
        &[Token::I32(32768)],
        "invalid value: integer `32768`, expected i16",
    );
    test(
        &[Token::I64(32768)],
        "invalid value: integer `32768`, expected i16",
    );

    // from unsigned
    test(
        &[Token::U16(32768)],
        "invalid value: integer `32768`, expected i16",
    );
    test(
        &[Token::U32(32768)],
        "invalid value: integer `32768`, expected i16",
    );
    test(
        &[Token::U64(32768)],
        "invalid value: integer `32768`, expected i16",
    );
}

#[test]
fn test_i32() {
    let test = assert_de_tokens_error::<i32>;

    // from signed
    test(
        &[Token::I64(-2147483649)],
        "invalid value: integer `-2147483649`, expected i32",
    );
    test(
        &[Token::I64(2147483648)],
        "invalid value: integer `2147483648`, expected i32",
    );

    // from unsigned
    test(
        &[Token::U32(2147483648)],
        "invalid value: integer `2147483648`, expected i32",
    );
    test(
        &[Token::U64(2147483648)],
        "invalid value: integer `2147483648`, expected i32",
    );
}

#[test]
fn test_i64() {
    let test = assert_de_tokens_error::<i64>;

    // from unsigned
    test(
        &[Token::U64(9223372036854775808)],
        "invalid value: integer `9223372036854775808`, expected i64",
    );
}

#[test]
fn test_i128() {
    let deserializer = <i128 as IntoDeserializer>::into_deserializer(1);
    let error = <&str>::deserialize(deserializer).unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid type: integer `1` as i128, expected a borrowed string",
    );
}

#[test]
fn test_u8() {
    let test = assert_de_tokens_error::<u8>;

    // from signed
    test(&[Token::I8(-1)], "invalid value: integer `-1`, expected u8");
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected u8",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected u8",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected u8",
    );
    test(
        &[Token::I16(256)],
        "invalid value: integer `256`, expected u8",
    );
    test(
        &[Token::I32(256)],
        "invalid value: integer `256`, expected u8",
    );
    test(
        &[Token::I64(256)],
        "invalid value: integer `256`, expected u8",
    );

    // from unsigned
    test(
        &[Token::U16(256)],
        "invalid value: integer `256`, expected u8",
    );
    test(
        &[Token::U32(256)],
        "invalid value: integer `256`, expected u8",
    );
    test(
        &[Token::U64(256)],
        "invalid value: integer `256`, expected u8",
    );
}

#[test]
fn test_u16() {
    let test = assert_de_tokens_error::<u16>;

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected u16",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected u16",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected u16",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected u16",
    );
    test(
        &[Token::I32(65536)],
        "invalid value: integer `65536`, expected u16",
    );
    test(
        &[Token::I64(65536)],
        "invalid value: integer `65536`, expected u16",
    );

    // from unsigned
    test(
        &[Token::U32(65536)],
        "invalid value: integer `65536`, expected u16",
    );
    test(
        &[Token::U64(65536)],
        "invalid value: integer `65536`, expected u16",
    );
}

#[test]
fn test_u32() {
    let test = assert_de_tokens_error::<u32>;

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected u32",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected u32",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected u32",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected u32",
    );
    test(
        &[Token::I64(4294967296)],
        "invalid value: integer `4294967296`, expected u32",
    );

    // from unsigned
    test(
        &[Token::U64(4294967296)],
        "invalid value: integer `4294967296`, expected u32",
    );
}

#[test]
fn test_u64() {
    let test = assert_de_tokens_error::<u64>;

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected u64",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected u64",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected u64",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected u64",
    );

    let deserializer = <u64 as IntoDeserializer>::into_deserializer(1);
    let error = <&str>::deserialize(deserializer).unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid type: integer `1`, expected a borrowed string",
    );
}

#[test]
fn test_u128() {
    let test = assert_de_tokens_error::<u128>;

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected u128",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected u128",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected u128",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected u128",
    );

    let deserializer = <u128 as IntoDeserializer>::into_deserializer(1);
    let error = <&str>::deserialize(deserializer).unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid type: integer `1` as u128, expected a borrowed string",
    );
}

#[test]
fn test_usize() {
    let test = assert_de_tokens_error::<usize>;

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected usize",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected usize",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected usize",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected usize",
    );
}

#[test]
fn test_nonzero_i8() {
    let test = assert_de_tokens_error::<NonZeroI8>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero i8",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero i8",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero i8",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero i8",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero i8",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero i8",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero i8",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero i8",
    );

    // from signed
    test(
        &[Token::I16(-129)],
        "invalid value: integer `-129`, expected a nonzero i8",
    );
    test(
        &[Token::I32(-129)],
        "invalid value: integer `-129`, expected a nonzero i8",
    );
    test(
        &[Token::I64(-129)],
        "invalid value: integer `-129`, expected a nonzero i8",
    );
    test(
        &[Token::I16(128)],
        "invalid value: integer `128`, expected a nonzero i8",
    );
    test(
        &[Token::I32(128)],
        "invalid value: integer `128`, expected a nonzero i8",
    );
    test(
        &[Token::I64(128)],
        "invalid value: integer `128`, expected a nonzero i8",
    );

    // from unsigned
    test(
        &[Token::U8(128)],
        "invalid value: integer `128`, expected a nonzero i8",
    );
    test(
        &[Token::U16(128)],
        "invalid value: integer `128`, expected a nonzero i8",
    );
    test(
        &[Token::U32(128)],
        "invalid value: integer `128`, expected a nonzero i8",
    );
    test(
        &[Token::U64(128)],
        "invalid value: integer `128`, expected a nonzero i8",
    );
}

#[test]
fn test_nonzero_i16() {
    let test = assert_de_tokens_error::<NonZeroI16>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero i16",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero i16",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero i16",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero i16",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero i16",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero i16",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero i16",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero i16",
    );

    // from signed
    test(
        &[Token::I32(-32769)],
        "invalid value: integer `-32769`, expected a nonzero i16",
    );
    test(
        &[Token::I64(-32769)],
        "invalid value: integer `-32769`, expected a nonzero i16",
    );
    test(
        &[Token::I32(32768)],
        "invalid value: integer `32768`, expected a nonzero i16",
    );
    test(
        &[Token::I64(32768)],
        "invalid value: integer `32768`, expected a nonzero i16",
    );

    // from unsigned
    test(
        &[Token::U16(32768)],
        "invalid value: integer `32768`, expected a nonzero i16",
    );
    test(
        &[Token::U32(32768)],
        "invalid value: integer `32768`, expected a nonzero i16",
    );
    test(
        &[Token::U64(32768)],
        "invalid value: integer `32768`, expected a nonzero i16",
    );
}

#[test]
fn test_nonzero_i32() {
    let test = assert_de_tokens_error::<NonZeroI32>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero i32",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero i32",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero i32",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero i32",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero i32",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero i32",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero i32",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero i32",
    );

    // from signed
    test(
        &[Token::I64(-2147483649)],
        "invalid value: integer `-2147483649`, expected a nonzero i32",
    );
    test(
        &[Token::I64(2147483648)],
        "invalid value: integer `2147483648`, expected a nonzero i32",
    );

    // from unsigned
    test(
        &[Token::U32(2147483648)],
        "invalid value: integer `2147483648`, expected a nonzero i32",
    );
    test(
        &[Token::U64(2147483648)],
        "invalid value: integer `2147483648`, expected a nonzero i32",
    );
}

#[test]
fn test_nonzero_i64() {
    let test = assert_de_tokens_error::<NonZeroI64>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero i64",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero i64",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero i64",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero i64",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero i64",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero i64",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero i64",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero i64",
    );

    // from unsigned
    test(
        &[Token::U64(9223372036854775808)],
        "invalid value: integer `9223372036854775808`, expected a nonzero i64",
    );
}

#[test]
fn test_nonzero_i128() {
    let test = assert_de_tokens_error::<NonZeroI128>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero i128",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero i128",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero i128",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero i128",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero i128",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero i128",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero i128",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero i128",
    );
}

#[test]
fn test_nonzero_isize() {
    let test = assert_de_tokens_error::<NonZeroIsize>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero isize",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero isize",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero isize",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero isize",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero isize",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero isize",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero isize",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero isize",
    );
}

#[test]
fn test_nonzero_u8() {
    let test = assert_de_tokens_error::<NonZeroU8>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero u8",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero u8",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero u8",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero u8",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero u8",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero u8",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero u8",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero u8",
    );

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected a nonzero u8",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected a nonzero u8",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected a nonzero u8",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected a nonzero u8",
    );
    test(
        &[Token::I16(256)],
        "invalid value: integer `256`, expected a nonzero u8",
    );
    test(
        &[Token::I32(256)],
        "invalid value: integer `256`, expected a nonzero u8",
    );
    test(
        &[Token::I64(256)],
        "invalid value: integer `256`, expected a nonzero u8",
    );

    // from unsigned
    test(
        &[Token::U16(256)],
        "invalid value: integer `256`, expected a nonzero u8",
    );
    test(
        &[Token::U32(256)],
        "invalid value: integer `256`, expected a nonzero u8",
    );
    test(
        &[Token::U64(256)],
        "invalid value: integer `256`, expected a nonzero u8",
    );
}

#[test]
fn test_nonzero_u16() {
    let test = assert_de_tokens_error::<NonZeroU16>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero u16",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero u16",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero u16",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero u16",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero u16",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero u16",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero u16",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero u16",
    );

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected a nonzero u16",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected a nonzero u16",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected a nonzero u16",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected a nonzero u16",
    );
    test(
        &[Token::I32(65536)],
        "invalid value: integer `65536`, expected a nonzero u16",
    );
    test(
        &[Token::I64(65536)],
        "invalid value: integer `65536`, expected a nonzero u16",
    );

    // from unsigned
    test(
        &[Token::U32(65536)],
        "invalid value: integer `65536`, expected a nonzero u16",
    );
    test(
        &[Token::U64(65536)],
        "invalid value: integer `65536`, expected a nonzero u16",
    );
}

#[test]
fn test_nonzero_u32() {
    let test = assert_de_tokens_error::<NonZeroU32>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero u32",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero u32",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero u32",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero u32",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero u32",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero u32",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero u32",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero u32",
    );

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected a nonzero u32",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected a nonzero u32",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected a nonzero u32",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected a nonzero u32",
    );
    test(
        &[Token::I64(4294967296)],
        "invalid value: integer `4294967296`, expected a nonzero u32",
    );

    // from unsigned
    test(
        &[Token::U64(4294967296)],
        "invalid value: integer `4294967296`, expected a nonzero u32",
    );
}

#[test]
fn test_nonzero_u64() {
    let test = assert_de_tokens_error::<NonZeroU64>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero u64",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero u64",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero u64",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero u64",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero u64",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero u64",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero u64",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero u64",
    );

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected a nonzero u64",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected a nonzero u64",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected a nonzero u64",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected a nonzero u64",
    );
}

#[test]
fn test_nonzero_u128() {
    let test = assert_de_tokens_error::<NonZeroU128>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero u128",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero u128",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero u128",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero u128",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero u128",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero u128",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero u128",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero u128",
    );

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected a nonzero u128",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected a nonzero u128",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected a nonzero u128",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected a nonzero u128",
    );
}

#[test]
fn test_nonzero_usize() {
    let test = assert_de_tokens_error::<NonZeroUsize>;

    // from zero
    test(
        &[Token::I8(0)],
        "invalid value: integer `0`, expected a nonzero usize",
    );
    test(
        &[Token::I16(0)],
        "invalid value: integer `0`, expected a nonzero usize",
    );
    test(
        &[Token::I32(0)],
        "invalid value: integer `0`, expected a nonzero usize",
    );
    test(
        &[Token::I64(0)],
        "invalid value: integer `0`, expected a nonzero usize",
    );
    test(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a nonzero usize",
    );
    test(
        &[Token::U16(0)],
        "invalid value: integer `0`, expected a nonzero usize",
    );
    test(
        &[Token::U32(0)],
        "invalid value: integer `0`, expected a nonzero usize",
    );
    test(
        &[Token::U64(0)],
        "invalid value: integer `0`, expected a nonzero usize",
    );

    // from signed
    test(
        &[Token::I8(-1)],
        "invalid value: integer `-1`, expected a nonzero usize",
    );
    test(
        &[Token::I16(-1)],
        "invalid value: integer `-1`, expected a nonzero usize",
    );
    test(
        &[Token::I32(-1)],
        "invalid value: integer `-1`, expected a nonzero usize",
    );
    test(
        &[Token::I64(-1)],
        "invalid value: integer `-1`, expected a nonzero usize",
    );
}

#[test]
fn test_unknown_field() {
    assert_de_tokens_error::<StructDenyUnknown>(
        &[
            Token::Struct {
                name: "StructDenyUnknown",
                len: 1,
            },
            Token::Str("a"),
            Token::I32(0),
            Token::Str("d"),
        ],
        "unknown field `d`, expected `a`",
    );
}

#[test]
fn test_skipped_field_is_unknown() {
    assert_de_tokens_error::<StructDenyUnknown>(
        &[
            Token::Struct {
                name: "StructDenyUnknown",
                len: 1,
            },
            Token::Str("b"),
        ],
        "unknown field `b`, expected `a`",
    );
}

#[test]
fn test_skip_all_deny_unknown() {
    assert_de_tokens_error::<StructSkipAllDenyUnknown>(
        &[
            Token::Struct {
                name: "StructSkipAllDenyUnknown",
                len: 0,
            },
            Token::Str("a"),
        ],
        "unknown field `a`, there are no fields",
    );
}

#[test]
fn test_unknown_variant() {
    assert_de_tokens_error::<Enum>(
    &[
        Token::UnitVariant { name: "Enum", variant: "Foo" },
    ],
    "unknown variant `Foo`, expected one of `Unit`, `Simple`, `Seq`, `Map`, `SimpleWithSkipped`",
    );
}

#[test]
fn test_enum_skipped_variant() {
    assert_de_tokens_error::<Enum>(
    &[
        Token::UnitVariant { name: "Enum", variant: "Skipped" },
    ],
    "unknown variant `Skipped`, expected one of `Unit`, `Simple`, `Seq`, `Map`, `SimpleWithSkipped`",
    );
}

#[test]
fn test_enum_skip_all() {
    assert_de_tokens_error::<EnumSkipAll>(
        &[Token::UnitVariant {
            name: "EnumSkipAll",
            variant: "Skipped",
        }],
        "unknown variant `Skipped`, there are no variants",
    );
}

#[test]
fn test_duplicate_field_struct() {
    assert_de_tokens_error::<Struct>(
        &[
            Token::Map { len: Some(3) },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("a"),
        ],
        "duplicate field `a`",
    );
}

#[test]
fn test_duplicate_field_enum() {
    assert_de_tokens_error::<Enum>(
        &[
            Token::StructVariant {
                name: "Enum",
                variant: "Map",
                len: 3,
            },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("a"),
        ],
        "duplicate field `a`",
    );
}

#[test]
fn test_enum_out_of_range() {
    assert_de_tokens_error::<Enum>(
        &[Token::Enum { name: "Enum" }, Token::U32(5), Token::Unit],
        "invalid value: integer `5`, expected variant index 0 <= i < 5",
    );
}

#[test]
fn test_short_tuple() {
    assert_de_tokens_error::<(u8, u8, u8)>(
        &[Token::Tuple { len: 1 }, Token::U8(1), Token::TupleEnd],
        "invalid length 1, expected a tuple of size 3",
    );
}

#[test]
fn test_short_array() {
    assert_de_tokens_error::<[u8; 3]>(
        &[Token::Seq { len: Some(1) }, Token::U8(1), Token::SeqEnd],
        "invalid length 1, expected an array of length 3",
    );
}

#[test]
fn test_cstring_internal_null() {
    assert_de_tokens_error::<CString>(
        &[Token::Bytes(b"a\0c")],
        "nul byte found in provided data at position: 1",
    );
}

#[test]
fn test_cstring_internal_null_end() {
    assert_de_tokens_error::<CString>(
        &[Token::Bytes(b"ac\0")],
        "nul byte found in provided data at position: 2",
    );
}

#[test]
fn test_unit_from_empty_seq() {
    assert_de_tokens_error::<()>(
        &[Token::Seq { len: Some(0) }, Token::SeqEnd],
        "invalid type: sequence, expected unit",
    );
}

#[test]
fn test_unit_from_empty_seq_without_len() {
    assert_de_tokens_error::<()>(
        &[Token::Seq { len: None }, Token::SeqEnd],
        "invalid type: sequence, expected unit",
    );
}

#[test]
fn test_unit_from_tuple_struct() {
    assert_de_tokens_error::<()>(
        &[
            Token::TupleStruct {
                name: "Anything",
                len: 0,
            },
            Token::TupleStructEnd,
        ],
        "invalid type: sequence, expected unit",
    );
}

#[test]
fn test_string_from_unit() {
    assert_de_tokens_error::<String>(
        &[Token::Unit],
        "invalid type: unit value, expected a string",
    );
}

#[test]
fn test_btreeset_from_unit() {
    assert_de_tokens_error::<BTreeSet<isize>>(
        &[Token::Unit],
        "invalid type: unit value, expected a sequence",
    );
}

#[test]
fn test_btreeset_from_unit_struct() {
    assert_de_tokens_error::<BTreeSet<isize>>(
        &[Token::UnitStruct { name: "Anything" }],
        "invalid type: unit value, expected a sequence",
    );
}

#[test]
fn test_hashset_from_unit() {
    assert_de_tokens_error::<HashSet<isize>>(
        &[Token::Unit],
        "invalid type: unit value, expected a sequence",
    );
}

#[test]
fn test_hashset_from_unit_struct() {
    assert_de_tokens_error::<HashSet<isize>>(
        &[Token::UnitStruct { name: "Anything" }],
        "invalid type: unit value, expected a sequence",
    );
}

#[test]
fn test_vec_from_unit() {
    assert_de_tokens_error::<Vec<isize>>(
        &[Token::Unit],
        "invalid type: unit value, expected a sequence",
    );
}

#[test]
fn test_vec_from_unit_struct() {
    assert_de_tokens_error::<Vec<isize>>(
        &[Token::UnitStruct { name: "Anything" }],
        "invalid type: unit value, expected a sequence",
    );
}

#[test]
fn test_zero_array_from_unit() {
    assert_de_tokens_error::<[isize; 0]>(
        &[Token::Unit],
        "invalid type: unit value, expected an empty array",
    );
}

#[test]
fn test_zero_array_from_unit_struct() {
    assert_de_tokens_error::<[isize; 0]>(
        &[Token::UnitStruct { name: "Anything" }],
        "invalid type: unit value, expected an empty array",
    );
}

#[test]
fn test_btreemap_from_unit() {
    assert_de_tokens_error::<BTreeMap<isize, isize>>(
        &[Token::Unit],
        "invalid type: unit value, expected a map",
    );
}

#[test]
fn test_btreemap_from_unit_struct() {
    assert_de_tokens_error::<BTreeMap<isize, isize>>(
        &[Token::UnitStruct { name: "Anything" }],
        "invalid type: unit value, expected a map",
    );
}

#[test]
fn test_hashmap_from_unit() {
    assert_de_tokens_error::<HashMap<isize, isize>>(
        &[Token::Unit],
        "invalid type: unit value, expected a map",
    );
}

#[test]
fn test_hashmap_from_unit_struct() {
    assert_de_tokens_error::<HashMap<isize, isize>>(
        &[Token::UnitStruct { name: "Anything" }],
        "invalid type: unit value, expected a map",
    );
}

#[test]
fn test_bool_from_string() {
    assert_de_tokens_error::<bool>(
        &[Token::Str("false")],
        "invalid type: string \"false\", expected a boolean",
    );
}

#[test]
fn test_number_from_string() {
    assert_de_tokens_error::<isize>(
        &[Token::Str("1")],
        "invalid type: string \"1\", expected isize",
    );
}

#[test]
fn test_integer_from_float() {
    assert_de_tokens_error::<isize>(
        &[Token::F32(0.0)],
        "invalid type: floating point `0.0`, expected isize",
    );
}

#[test]
fn test_nan_no_decimal_point() {
    assert_de_tokens_error::<isize>(
        &[Token::F32(f32::NAN)],
        "invalid type: floating point `NaN`, expected isize",
    );
}

#[test]
fn test_unit_struct_from_seq() {
    assert_de_tokens_error::<UnitStruct>(
        &[Token::Seq { len: Some(0) }, Token::SeqEnd],
        "invalid type: sequence, expected unit struct UnitStruct",
    );
}

#[test]
fn test_wrapping_overflow() {
    assert_de_tokens_error::<Wrapping<u16>>(
        &[Token::U32(65_536)],
        "invalid value: integer `65536`, expected u16",
    );
}

#[test]
fn test_duration_overflow_seq() {
    assert_de_tokens_error::<Duration>(
        &[
            Token::Seq { len: Some(2) },
            Token::U64(u64::MAX),
            Token::U32(1_000_000_000),
            Token::SeqEnd,
        ],
        "overflow deserializing Duration",
    );
}

#[test]
fn test_duration_overflow_struct() {
    assert_de_tokens_error::<Duration>(
        &[
            Token::Struct {
                name: "Duration",
                len: 2,
            },
            Token::Str("secs"),
            Token::U64(u64::MAX),
            Token::Str("nanos"),
            Token::U32(1_000_000_000),
            Token::StructEnd,
        ],
        "overflow deserializing Duration",
    );
}

#[test]
fn test_systemtime_overflow_seq() {
    assert_de_tokens_error::<SystemTime>(
        &[
            Token::Seq { len: Some(2) },
            Token::U64(u64::MAX),
            Token::U32(1_000_000_000),
            Token::SeqEnd,
        ],
        "overflow deserializing SystemTime epoch offset",
    );
}

#[test]
fn test_systemtime_overflow_struct() {
    assert_de_tokens_error::<SystemTime>(
        &[
            Token::Struct {
                name: "SystemTime",
                len: 2,
            },
            Token::Str("secs_since_epoch"),
            Token::U64(u64::MAX),
            Token::Str("nanos_since_epoch"),
            Token::U32(1_000_000_000),
            Token::StructEnd,
        ],
        "overflow deserializing SystemTime epoch offset",
    );
}

#[test]
fn test_systemtime_overflow() {
    assert_de_tokens_error::<SystemTime>(
        &[
            Token::Seq { len: Some(2) },
            Token::U64(u64::MAX),
            Token::U32(0),
            Token::SeqEnd,
        ],
        "overflow deserializing SystemTime",
    );
}

#[test]
fn test_cstr_internal_null() {
    assert_de_tokens_error::<Box<CStr>>(
        &[Token::Bytes(b"a\0c")],
        "nul byte found in provided data at position: 1",
    );
}

#[test]
fn test_cstr_internal_null_end() {
    assert_de_tokens_error::<Box<CStr>>(
        &[Token::Bytes(b"ac\0")],
        "nul byte found in provided data at position: 2",
    );
}

#[cfg(feature = "unstable")]
#[test]
fn test_never_type() {
    assert_de_tokens_error::<!>(&[], "cannot deserialize `!`");

    assert_de_tokens_error::<Result<u8, !>>(
        &[Token::NewtypeVariant {
            name: "Result",
            variant: "Err",
        }],
        "cannot deserialize `!`",
    );
}
