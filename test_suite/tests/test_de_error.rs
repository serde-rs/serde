#![allow(clippy::empty_enum)]
#![cfg_attr(feature = "unstable", feature(never_type))]

use serde::Deserialize;
use serde_test::{assert_de_tokens_error, Token};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::num::Wrapping;
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

macro_rules! declare_error_tests {
    ($(
        $(#[$cfg:meta])*
        $name:ident<$target:ty> { $tokens:expr, $expected:expr, }
    )+) => {
        $(
            $(#[$cfg])*
            #[test]
            fn $name() {
                assert_de_tokens_error::<$target>($tokens, $expected);
            }
        )+
    }
}

declare_error_tests! {
    test_unknown_field<StructDenyUnknown> {
        &[
            Token::Struct { name: "StructDenyUnknown", len: 1 },
                Token::Str("a"),
                Token::I32(0),

                Token::Str("d"),
        ],
        "unknown field `d`, expected `a`",
    }
    test_skipped_field_is_unknown<StructDenyUnknown> {
        &[
            Token::Struct { name: "StructDenyUnknown", len: 1 },
                Token::Str("b"),
        ],
        "unknown field `b`, expected `a`",
    }
    test_skip_all_deny_unknown<StructSkipAllDenyUnknown> {
        &[
            Token::Struct { name: "StructSkipAllDenyUnknown", len: 0 },
                Token::Str("a"),
        ],
        "unknown field `a`, there are no fields",
    }
    test_unknown_variant<Enum> {
        &[
            Token::UnitVariant { name: "Enum", variant: "Foo" },
        ],
        "unknown variant `Foo`, expected one of `Unit`, `Simple`, `Seq`, `Map`, `SimpleWithSkipped`",
    }
    test_enum_skipped_variant<Enum> {
        &[
            Token::UnitVariant { name: "Enum", variant: "Skipped" },
        ],
        "unknown variant `Skipped`, expected one of `Unit`, `Simple`, `Seq`, `Map`, `SimpleWithSkipped`",
    }
    test_enum_skip_all<EnumSkipAll> {
        &[
            Token::UnitVariant { name: "EnumSkipAll", variant: "Skipped" },
        ],
        "unknown variant `Skipped`, there are no variants",
    }
    test_duplicate_field_struct<Struct> {
        &[
            Token::Map { len: Some(3) },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("a"),
        ],
        "duplicate field `a`",
    }
    test_duplicate_field_enum<Enum> {
        &[
            Token::StructVariant { name: "Enum", variant: "Map", len: 3 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("a"),
        ],
        "duplicate field `a`",
    }
    test_enum_out_of_range<Enum> {
        &[
            Token::Enum { name: "Enum" },
            Token::U32(5),
            Token::Unit,
        ],
        "invalid value: integer `5`, expected variant index 0 <= i < 5",
    }
    test_short_tuple<(u8, u8, u8)> {
        &[
            Token::Tuple { len: 1 },
            Token::U8(1),
            Token::TupleEnd,
        ],
        "invalid length 1, expected a tuple of size 3",
    }
    test_short_array<[u8; 3]> {
        &[
            Token::Seq { len: Some(1) },
            Token::U8(1),
            Token::SeqEnd,
        ],
        "invalid length 1, expected an array of length 3",
    }
    test_cstring_internal_null<CString> {
        &[
            Token::Bytes(b"a\0c"),
        ],
        "nul byte found in provided data at position: 1",
    }
    test_cstring_internal_null_end<CString> {
        &[
            Token::Bytes(b"ac\0"),
        ],
        "nul byte found in provided data at position: 2",
    }
    test_unit_from_empty_seq<()> {
        &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        "invalid type: sequence, expected unit",
    }
    test_unit_from_empty_seq_without_len<()> {
        &[
            Token::Seq { len: None },
            Token::SeqEnd,
        ],
        "invalid type: sequence, expected unit",
    }
    test_unit_from_tuple_struct<()> {
        &[
            Token::TupleStruct { name: "Anything", len: 0 },
            Token::TupleStructEnd,
        ],
        "invalid type: sequence, expected unit",
    }
    test_string_from_unit<String> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a string",
    }
    test_btreeset_from_unit<BTreeSet<isize>> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_btreeset_from_unit_struct<BTreeSet<isize>> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_hashset_from_unit<HashSet<isize>> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_hashset_from_unit_struct<HashSet<isize>> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_vec_from_unit<Vec<isize>> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_vec_from_unit_struct<Vec<isize>> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_zero_array_from_unit<[isize; 0]> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected an empty array",
    }
    test_zero_array_from_unit_struct<[isize; 0]> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected an empty array",
    }
    test_btreemap_from_unit<BTreeMap<isize, isize>> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a map",
    }
    test_btreemap_from_unit_struct<BTreeMap<isize, isize>> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected a map",
    }
    test_hashmap_from_unit<HashMap<isize, isize>> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a map",
    }
    test_hashmap_from_unit_struct<HashMap<isize, isize>> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected a map",
    }
    test_bool_from_string<bool> {
        &[
            Token::Str("false"),
        ],
        "invalid type: string \"false\", expected a boolean",
    }
    test_number_from_string<isize> {
        &[
            Token::Str("1"),
        ],
        "invalid type: string \"1\", expected isize",
    }
    test_integer_from_float<isize> {
        &[
            Token::F32(0.0),
        ],
        "invalid type: floating point `0`, expected isize",
    }
    test_unit_struct_from_seq<UnitStruct> {
        &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        "invalid type: sequence, expected unit struct UnitStruct",
    }
    test_wrapping_overflow<Wrapping<u16>> {
        &[
            Token::U32(65_536),
        ],
        "invalid value: integer `65536`, expected u16",
    }
    test_duration_overflow_seq<Duration> {
        &[
            Token::Seq { len: Some(2) },
                Token::U64(u64::max_value()),
                Token::U32(1_000_000_000),
            Token::SeqEnd,
        ],
        "overflow deserializing Duration",
    }
    test_duration_overflow_struct<Duration> {
        &[
            Token::Struct { name: "Duration", len: 2 },
                Token::Str("secs"),
                Token::U64(u64::max_value()),

                Token::Str("nanos"),
                Token::U32(1_000_000_000),
            Token::StructEnd,
        ],
        "overflow deserializing Duration",
    }
    test_systemtime_overflow_seq<SystemTime> {
        &[
            Token::Seq { len: Some(2) },
                Token::U64(u64::max_value()),
                Token::U32(1_000_000_000),
            Token::SeqEnd,
        ],
        "overflow deserializing SystemTime epoch offset",
    }
    test_systemtime_overflow_struct<SystemTime> {
        &[
            Token::Struct { name: "SystemTime", len: 2 },
                Token::Str("secs_since_epoch"),
                Token::U64(u64::max_value()),

                Token::Str("nanos_since_epoch"),
                Token::U32(1_000_000_000),
            Token::StructEnd,
        ],
        "overflow deserializing SystemTime epoch offset",
    }
    #[cfg(systemtime_checked_add)]
    test_systemtime_overflow<SystemTime> {
        &[
            Token::Seq { len: Some(2) },
                Token::U64(u64::max_value()),
                Token::U32(0),
            Token::SeqEnd,
        ],
        "overflow deserializing SystemTime",
    }
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
