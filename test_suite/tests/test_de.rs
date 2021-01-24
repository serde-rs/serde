#![allow(clippy::decimal_literal_representation, clippy::unreadable_literal)]
#![cfg_attr(feature = "unstable", feature(never_type))]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
use std::ffi::{CStr, CString, OsString};
use std::fmt::Debug;
use std::net;
use std::num::Wrapping;
use std::ops::Bound;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak as RcWeak};
use std::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI8, AtomicIsize, AtomicU16, AtomicU32, AtomicU8,
    AtomicUsize, Ordering,
};
use std::sync::{Arc, Weak as ArcWeak};
use std::time::{Duration, UNIX_EPOCH};

#[cfg(target_arch = "x86_64")]
use std::sync::atomic::{AtomicI64, AtomicU64};

use fnv::FnvHasher;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_test::{assert_de_tokens, assert_de_tokens_error, Configure, Token};

#[macro_use]
mod macros;

//////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
struct UnitStruct;

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
#[serde(deny_unknown_fields)]
struct StructDenyUnknown {
    a: i32,
    #[serde(skip_deserializing)]
    b: i32,
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
struct StructSkipDefaultGeneric<T> {
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
enum EnumSkipAll {
    #[allow(dead_code)]
    #[serde(skip_deserializing)]
    Skipped,
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

macro_rules! declare_tests {
    (
        $readable:tt
        $($name:ident { $($value:expr => $tokens:expr,)+ })+
    ) => {
        $(
            #[test]
            fn $name() {
                $(
                    // Test ser/de roundtripping
                    assert_de_tokens(&$value.$readable(), $tokens);

                    // Test that the tokens are ignorable
                    assert_de_tokens_ignore($tokens);
                )+
            }
        )+
    };

    ($(
        $(#[$cfg:meta])*
        $name:ident { $($value:expr => $tokens:expr,)+ }
    )+) => {
        $(
            $(#[$cfg])*
            #[test]
            fn $name() {
                $(
                    // Test ser/de roundtripping
                    assert_de_tokens(&$value, $tokens);

                    // Test that the tokens are ignorable
                    assert_de_tokens_ignore($tokens);
                )+
            }
        )+
    }
}

macro_rules! declare_error_tests {
    ($($name:ident<$target:ty> { $tokens:expr, $expected:expr, })+) => {
        $(
            #[test]
            fn $name() {
                assert_de_tokens_error::<$target>($tokens, $expected);
            }
        )+
    }
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
    .chain(ignorable_tokens.to_vec().into_iter())
    .chain(vec![Token::MapEnd].into_iter())
    .collect();

    let mut de = serde_test::Deserializer::new(&concated_tokens);
    let base = IgnoreBase::deserialize(&mut de).unwrap();
    assert_eq!(base, IgnoreBase { a: 1 });
}

//////////////////////////////////////////////////////////////////////////

declare_tests! {
    test_bool {
        true => &[Token::Bool(true)],
        false => &[Token::Bool(false)],
    }
    test_isize {
        0isize => &[Token::I8(0)],
        0isize => &[Token::I16(0)],
        0isize => &[Token::I32(0)],
        0isize => &[Token::I64(0)],
        0isize => &[Token::U8(0)],
        0isize => &[Token::U16(0)],
        0isize => &[Token::U32(0)],
        0isize => &[Token::U64(0)],
    }
    test_ints {
        0i8 => &[Token::I8(0)],
        0i16 => &[Token::I16(0)],
        0i32 => &[Token::I32(0)],
        0i64 => &[Token::I64(0)],
    }
    test_uints {
        0u8 => &[Token::U8(0)],
        0u16 => &[Token::U16(0)],
        0u32 => &[Token::U32(0)],
        0u64 => &[Token::U64(0)],
    }
    test_floats {
        0f32 => &[Token::F32(0.)],
        0f64 => &[Token::F64(0.)],
    }
    #[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
    test_small_int_to_128 {
        1i128 => &[Token::I8(1)],
        1i128 => &[Token::I16(1)],
        1i128 => &[Token::I32(1)],
        1i128 => &[Token::I64(1)],

        1i128 => &[Token::U8(1)],
        1i128 => &[Token::U16(1)],
        1i128 => &[Token::U32(1)],
        1i128 => &[Token::U64(1)],

        1u128 => &[Token::I8(1)],
        1u128 => &[Token::I16(1)],
        1u128 => &[Token::I32(1)],
        1u128 => &[Token::I64(1)],

        1u128 => &[Token::U8(1)],
        1u128 => &[Token::U16(1)],
        1u128 => &[Token::U32(1)],
        1u128 => &[Token::U64(1)],
    }
    test_char {
        'a' => &[Token::Char('a')],
        'a' => &[Token::Str("a")],
        'a' => &[Token::String("a")],
    }
    test_string {
        "abc".to_owned() => &[Token::Str("abc")],
        "abc".to_owned() => &[Token::String("abc")],
        "a".to_owned() => &[Token::Char('a')],
    }
    test_option {
        None::<i32> => &[Token::Unit],
        None::<i32> => &[Token::None],
        Some(1) => &[
            Token::Some,
            Token::I32(1),
        ],
    }
    test_result {
        Ok::<i32, i32>(0) => &[
            Token::Enum { name: "Result" },
            Token::Str("Ok"),
            Token::I32(0),
        ],
        Err::<i32, i32>(1) => &[
            Token::Enum { name: "Result" },
            Token::Str("Err"),
            Token::I32(1),
        ],
    }
    test_unit {
        () => &[Token::Unit],
    }
    test_unit_struct {
        UnitStruct => &[Token::Unit],
        UnitStruct => &[
            Token::UnitStruct { name: "UnitStruct" },
        ],
    }
    test_newtype_struct {
        NewtypeStruct(1) => &[
            Token::NewtypeStruct { name: "NewtypeStruct" },
            Token::I32(1),
        ],
    }
    test_tuple_struct {
        TupleStruct(1, 2, 3) => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
        TupleStruct(1, 2, 3) => &[
            Token::Seq { len: None },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
        TupleStruct(1, 2, 3) => &[
            Token::TupleStruct { name: "TupleStruct", len: 3 },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleStructEnd,
        ],
        TupleStruct(1, 2, 3) => &[
            Token::TupleStruct { name: "TupleStruct", len: 3 },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleStructEnd,
        ],
    }
    test_btreeset {
        BTreeSet::<isize>::new() => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        btreeset![btreeset![], btreeset![1], btreeset![2, 3]] => &[
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
        BTreeSet::<isize>::new() => &[
            Token::TupleStruct { name: "Anything", len: 0 },
            Token::TupleStructEnd,
        ],
    }
    test_hashset {
        HashSet::<isize>::new() => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        hashset![1, 2, 3] => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
        HashSet::<isize>::new() => &[
            Token::TupleStruct { name: "Anything", len: 0 },
            Token::TupleStructEnd,
        ],
        hashset![FnvHasher @ 1, 2, 3] => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_vec {
        Vec::<isize>::new() => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        vec![vec![], vec![1], vec![2, 3]] => &[
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
        Vec::<isize>::new() => &[
            Token::TupleStruct { name: "Anything", len: 0 },
            Token::TupleStructEnd,
        ],
    }
    test_array {
        [0; 0] => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        [0; 0] => &[
            Token::Tuple { len: 0 },
            Token::TupleEnd,
        ],
        ([0; 0], [1], [2, 3]) => &[
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
        ([0; 0], [1], [2, 3]) => &[
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
        [0; 0] => &[
            Token::TupleStruct { name: "Anything", len: 0 },
            Token::TupleStructEnd,
        ],
    }
    test_tuple {
        (1,) => &[
            Token::Seq { len: Some(1) },
                Token::I32(1),
            Token::SeqEnd,
        ],
        (1, 2, 3) => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
        (1,) => &[
            Token::Tuple { len: 1 },
                Token::I32(1),
            Token::TupleEnd,
        ],
        (1, 2, 3) => &[
            Token::Tuple { len: 3 },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleEnd,
        ],
    }
    test_btreemap {
        BTreeMap::<isize, isize>::new() => &[
            Token::Map { len: Some(0) },
            Token::MapEnd,
        ],
        btreemap![1 => 2] => &[
            Token::Map { len: Some(1) },
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        btreemap![1 => 2, 3 => 4] => &[
            Token::Map { len: Some(2) },
                Token::I32(1),
                Token::I32(2),

                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]] => &[
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
        BTreeMap::<isize, isize>::new() => &[
            Token::Struct { name: "Anything", len: 0 },
            Token::StructEnd,
        ],
    }
    test_hashmap {
        HashMap::<isize, isize>::new() => &[
            Token::Map { len: Some(0) },
            Token::MapEnd,
        ],
        hashmap![1 => 2] => &[
            Token::Map { len: Some(1) },
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        hashmap![1 => 2, 3 => 4] => &[
            Token::Map { len: Some(2) },
                Token::I32(1),
                Token::I32(2),

                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        hashmap![1 => hashmap![], 2 => hashmap![3 => 4, 5 => 6]] => &[
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
        HashMap::<isize, isize>::new() => &[
            Token::Struct { name: "Anything", len: 0 },
            Token::StructEnd,
        ],
        hashmap![FnvHasher @ 1 => 2, 3 => 4] => &[
            Token::Map { len: Some(2) },
                Token::I32(1),
                Token::I32(2),

                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
    }
    test_struct {
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::U8(0),
                Token::I32(1),

                Token::U8(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::U16(0),
                Token::I32(1),

                Token::U16(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::U32(0),
                Token::I32(1),

                Token::U32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::U64(0),
                Token::I32(1),

                Token::U64(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        // Mixed key types
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::U8(0),
                Token::I32(1),

                Token::U64(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::U8(0),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Struct { name: "Struct", len: 2 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
            Token::SeqEnd,
        ],
    }
    test_struct_borrowed_keys {
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::BorrowedStr("a"),
                Token::I32(1),

                Token::BorrowedStr("b"),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Struct { name: "Struct", len: 2 },
                Token::BorrowedStr("a"),
                Token::I32(1),

                Token::BorrowedStr("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
    }
    test_struct_owned_keys {
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::String("a"),
                Token::I32(1),

                Token::String("b"),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Struct { name: "Struct", len: 2 },
                Token::String("a"),
                Token::I32(1),

                Token::String("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
    }
    test_struct_with_skip {
        Struct { a: 1, b: 2, c: 0 } => &[
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
        Struct { a: 1, b: 2, c: 0 } => &[
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
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Struct { name: "Struct", len: 2 },
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
    }
    test_struct_skip_all {
        StructSkipAll { a: 0 } => &[
            Token::Struct { name: "StructSkipAll", len: 0 },
            Token::StructEnd,
        ],
        StructSkipAll { a: 0 } => &[
            Token::Struct { name: "StructSkipAll", len: 0 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
    }
    test_struct_skip_default {
        StructSkipDefault { a: 16 } => &[
            Token::Struct { name: "StructSkipDefault", len: 0 },
            Token::StructEnd,
        ],
    }
    test_struct_skip_all_deny_unknown {
        StructSkipAllDenyUnknown { a: 0 } => &[
            Token::Struct { name: "StructSkipAllDenyUnknown", len: 0 },
            Token::StructEnd,
        ],
    }
    test_struct_default {
        StructDefault { a: 50, b: "overwritten".to_string() } => &[
            Token::Struct { name: "StructDefault", len: 2 },
                Token::Str("a"),
                Token::I32(50),

                Token::Str("b"),
                Token::String("overwritten"),
            Token::StructEnd,
        ],
        StructDefault { a: 100, b: "default".to_string() } => &[
            Token::Struct { name: "StructDefault",  len: 2 },
            Token::StructEnd,
        ],
    }
    test_enum_unit {
        Enum::Unit => &[
            Token::UnitVariant { name: "Enum", variant: "Unit" },
        ],
    }
    test_enum_simple {
        Enum::Simple(1) => &[
            Token::NewtypeVariant { name: "Enum", variant: "Simple" },
            Token::I32(1),
        ],
    }
    test_enum_simple_with_skipped {
        Enum::SimpleWithSkipped(NotDeserializable) => &[
            Token::UnitVariant { name: "Enum", variant: "SimpleWithSkipped" },
        ],
    }
    test_enum_seq {
        Enum::Seq(1, 2, 3) => &[
            Token::TupleVariant { name: "Enum", variant: "Seq", len: 3 },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleVariantEnd,
        ],
    }
    test_enum_map {
        Enum::Map { a: 1, b: 2, c: 3 } => &[
            Token::StructVariant { name: "Enum", variant: "Map", len: 3 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),

                Token::Str("c"),
                Token::I32(3),
            Token::StructVariantEnd,
        ],
    }
    test_enum_unit_usize {
        Enum::Unit => &[
            Token::Enum { name: "Enum" },
            Token::U32(0),
            Token::Unit,
        ],
    }
    test_enum_unit_bytes {
        Enum::Unit => &[
            Token::Enum { name: "Enum" },
            Token::Bytes(b"Unit"),
            Token::Unit,
        ],
    }
    test_enum_other_unit {
        EnumOther::Unit => &[
            Token::Enum { name: "EnumOther" },
            Token::Str("Unit"),
            Token::Unit,
        ],
        EnumOther::Unit => &[
            Token::Enum { name: "EnumOther" },
            Token::U8(0),
            Token::Unit,
        ],
        EnumOther::Unit => &[
            Token::Enum { name: "EnumOther" },
            Token::U16(0),
            Token::Unit,
        ],
        EnumOther::Unit => &[
            Token::Enum { name: "EnumOther" },
            Token::U32(0),
            Token::Unit,
        ],
        EnumOther::Unit => &[
            Token::Enum { name: "EnumOther" },
            Token::U64(0),
            Token::Unit,
        ],
    }
    test_enum_other {
        EnumOther::Other => &[
            Token::Enum { name: "EnumOther" },
            Token::Str("Foo"),
            Token::Unit,
        ],
        EnumOther::Other => &[
            Token::Enum { name: "EnumOther" },
            Token::U8(42),
            Token::Unit,
        ],
        EnumOther::Other => &[
            Token::Enum { name: "EnumOther" },
            Token::U16(42),
            Token::Unit,
        ],
        EnumOther::Other => &[
            Token::Enum { name: "EnumOther" },
            Token::U32(42),
            Token::Unit,
        ],
        EnumOther::Other => &[
            Token::Enum { name: "EnumOther" },
            Token::U64(42),
            Token::Unit,
        ],
    }
    test_box {
        Box::new(0i32) => &[Token::I32(0)],
    }
    test_boxed_slice {
        Box::new([0, 1, 2]) => &[
            Token::Seq { len: Some(3) },
            Token::I32(0),
            Token::I32(1),
            Token::I32(2),
            Token::SeqEnd,
        ],
    }
    test_duration {
        Duration::new(1, 2) => &[
            Token::Struct { name: "Duration", len: 2 },
                Token::Str("secs"),
                Token::U64(1),

                Token::Str("nanos"),
                Token::U32(2),
            Token::StructEnd,
        ],
        Duration::new(1, 2) => &[
            Token::Seq { len: Some(2) },
                Token::I64(1),
                Token::I64(2),
            Token::SeqEnd,
        ],
    }
    test_system_time {
        UNIX_EPOCH + Duration::new(1, 2) => &[
            Token::Struct { name: "SystemTime", len: 2 },
                Token::Str("secs_since_epoch"),
                Token::U64(1),

                Token::Str("nanos_since_epoch"),
                Token::U32(2),
            Token::StructEnd,
        ],
        UNIX_EPOCH + Duration::new(1, 2) => &[
            Token::Seq { len: Some(2) },
                Token::I64(1),
                Token::I64(2),
            Token::SeqEnd,
        ],
    }
    test_range {
        1u32..2u32 => &[
            Token::Struct { name: "Range", len: 2 },
                Token::Str("start"),
                Token::U32(1),

                Token::Str("end"),
                Token::U32(2),
            Token::StructEnd,
        ],
        1u32..2u32 => &[
            Token::Seq { len: Some(2) },
                Token::U64(1),
                Token::U64(2),
            Token::SeqEnd,
        ],
    }
    test_range_inclusive {
        1u32..=2u32 => &[
            Token::Struct { name: "RangeInclusive", len: 2 },
                Token::Str("start"),
                Token::U32(1),

                Token::Str("end"),
                Token::U32(2),
            Token::StructEnd,
        ],
        1u32..=2u32 => &[
            Token::Seq { len: Some(2) },
                Token::U64(1),
                Token::U64(2),
            Token::SeqEnd,
        ],
    }
    test_bound {
        Bound::Unbounded::<()> => &[
            Token::Enum { name: "Bound" },
            Token::Str("Unbounded"),
            Token::Unit,
        ],
        Bound::Included(0) => &[
            Token::Enum { name: "Bound" },
            Token::Str("Included"),
            Token::U8(0),
        ],
        Bound::Excluded(0) => &[
            Token::Enum { name: "Bound" },
            Token::Str("Excluded"),
            Token::U8(0),
        ],
    }
    test_path {
        Path::new("/usr/local/lib") => &[
            Token::BorrowedStr("/usr/local/lib"),
        ],
        Path::new("/usr/local/lib") => &[
            Token::BorrowedBytes(b"/usr/local/lib"),
        ],
    }
    test_path_buf {
        PathBuf::from("/usr/local/lib") => &[
            Token::Str("/usr/local/lib"),
        ],
        PathBuf::from("/usr/local/lib") => &[
            Token::String("/usr/local/lib"),
        ],
        PathBuf::from("/usr/local/lib") => &[
            Token::Bytes(b"/usr/local/lib"),
        ],
        PathBuf::from("/usr/local/lib") => &[
            Token::ByteBuf(b"/usr/local/lib"),
        ],
    }
    test_boxed_path {
        PathBuf::from("/usr/local/lib").into_boxed_path() => &[
            Token::Str("/usr/local/lib"),
        ],
        PathBuf::from("/usr/local/lib").into_boxed_path() => &[
            Token::String("/usr/local/lib"),
        ],
        PathBuf::from("/usr/local/lib").into_boxed_path() => &[
            Token::Bytes(b"/usr/local/lib"),
        ],
        PathBuf::from("/usr/local/lib").into_boxed_path() => &[
            Token::ByteBuf(b"/usr/local/lib"),
        ],
    }
    test_cstring {
        CString::new("abc").unwrap() => &[
            Token::Bytes(b"abc"),
        ],
    }
    test_rc {
        Rc::new(true) => &[
            Token::Bool(true),
        ],
    }
    test_rc_weak_some {
        SkipPartialEq(RcWeak::<bool>::new()) => &[
            Token::Some,
            Token::Bool(true),
        ],
    }
    test_rc_weak_none {
        SkipPartialEq(RcWeak::<bool>::new()) => &[
            Token::None,
        ],
    }
    test_arc {
        Arc::new(true) => &[
            Token::Bool(true),
        ],
    }
    test_arc_weak_some {
        SkipPartialEq(ArcWeak::<bool>::new()) => &[
            Token::Some,
            Token::Bool(true),
        ],
    }
    test_arc_weak_none {
        SkipPartialEq(ArcWeak::<bool>::new()) => &[
            Token::None,
        ],
    }
    test_wrapping {
        Wrapping(1usize) => &[
            Token::U32(1),
        ],
        Wrapping(1usize) => &[
            Token::U64(1),
        ],
    }
    test_rc_dst {
        Rc::<str>::from("s") => &[
            Token::Str("s"),
        ],
        Rc::<[bool]>::from(&[true][..]) => &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    }
    test_arc_dst {
        Arc::<str>::from("s") => &[
            Token::Str("s"),
        ],
        Arc::<[bool]>::from(&[true][..]) => &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    }
    test_ignored_any {
        IgnoredAny => &[
            Token::Str("s"),
        ],
        IgnoredAny => &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
        IgnoredAny => &[
            Token::Enum { name: "E" },
            Token::Str("Rust"),
            Token::Unit,
        ],
    }
}

declare_tests! {
    readable

    test_net_ipv4addr_readable {
        "1.2.3.4".parse::<net::Ipv4Addr>().unwrap() => &[Token::Str("1.2.3.4")],
    }
    test_net_ipv6addr_readable {
        "::1".parse::<net::Ipv6Addr>().unwrap() => &[Token::Str("::1")],
    }
    test_net_ipaddr_readable {
        "1.2.3.4".parse::<net::IpAddr>().unwrap() => &[Token::Str("1.2.3.4")],
    }
    test_net_socketaddr_readable {
        "1.2.3.4:1234".parse::<net::SocketAddr>().unwrap() => &[Token::Str("1.2.3.4:1234")],
        "1.2.3.4:1234".parse::<net::SocketAddrV4>().unwrap() => &[Token::Str("1.2.3.4:1234")],
        "[::1]:1234".parse::<net::SocketAddrV6>().unwrap() => &[Token::Str("[::1]:1234")],
    }
}

declare_tests! {
    compact

    test_net_ipv4addr_compact {
        net::Ipv4Addr::from(*b"1234") => &seq![
            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd
        ],
    }
    test_net_ipv6addr_compact {
        net::Ipv6Addr::from(*b"1234567890123456") => &seq![
            Token::Tuple { len: 4 },
            seq b"1234567890123456".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd
        ],
    }
    test_net_ipaddr_compact {
        net::IpAddr::from(*b"1234") => &seq![
            Token::NewtypeVariant { name: "IpAddr", variant: "V4" },

            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd
        ],
    }
    test_net_socketaddr_compact {
        net::SocketAddr::from((*b"1234567890123456", 1234)) => &seq![
            Token::NewtypeVariant { name: "SocketAddr", variant: "V6" },

            Token::Tuple { len: 2 },

            Token::Tuple { len: 16 },
            seq b"1234567890123456".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd
        ],
        net::SocketAddr::from((*b"1234", 1234)) => &seq![
            Token::NewtypeVariant { name: "SocketAddr", variant: "V4" },

            Token::Tuple { len: 2 },

            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd
        ],
        net::SocketAddrV4::new(net::Ipv4Addr::from(*b"1234"), 1234) => &seq![
            Token::Tuple { len: 2 },

            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd
        ],
        net::SocketAddrV6::new(net::Ipv6Addr::from(*b"1234567890123456"), 1234, 0, 0) => &seq![
            Token::Tuple { len: 2 },

            Token::Tuple { len: 16 },
            seq b"1234567890123456".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd
        ],
    }
}

#[cfg(feature = "unstable")]
declare_tests! {
    test_never_result {
        Ok::<u8, !>(0) => &[
            Token::NewtypeVariant { name: "Result", variant: "Ok" },
            Token::U8(0),
        ],
    }
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

#[test]
fn test_atomics() {
    fn test<L, A, T>(load: L, val: T, token: Token)
    where
        L: Fn(&A, Ordering) -> T,
        A: DeserializeOwned,
        T: PartialEq + Debug,
    {
        let tokens = &[token];
        let mut de = serde_test::Deserializer::new(tokens);
        match A::deserialize(&mut de) {
            Ok(v) => {
                let loaded = load(&v, Ordering::SeqCst);
                assert_eq!(val, loaded);
            }
            Err(e) => panic!("tokens failed to deserialize: {}", e),
        };
        if de.remaining() > 0 {
            panic!("{} remaining tokens", de.remaining());
        }
    }

    test(AtomicBool::load, true, Token::Bool(true));
    test(AtomicI8::load, -127, Token::I8(-127i8));
    test(AtomicI16::load, -510, Token::I16(-510i16));
    test(AtomicI32::load, -131072, Token::I32(-131072i32));
    test(AtomicIsize::load, -131072isize, Token::I32(-131072));
    test(AtomicU8::load, 127, Token::U8(127u8));
    test(AtomicU16::load, 510u16, Token::U16(510u16));
    test(AtomicU32::load, 131072u32, Token::U32(131072u32));
    test(AtomicUsize::load, 131072usize, Token::U32(131072));

    #[cfg(target_arch = "x86_64")]
    {
        test(AtomicI64::load, -8589934592, Token::I64(-8589934592));
        test(AtomicU64::load, 8589934592u64, Token::U64(8589934592));
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
}
