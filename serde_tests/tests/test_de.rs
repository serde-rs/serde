use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use num::FromPrimitive;
use num::bigint::{BigInt, BigUint};
use num::complex::Complex;
use num::rational::Ratio;

use serde::de::{Deserializer, Visitor};

use token::{Error, Token, assert_de_tokens, assert_de_tokens_ignore};

//////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
struct UnitStruct;

#[derive(PartialEq, Debug, Deserialize)]
struct TupleStruct(i32, i32, i32);

#[derive(PartialEq, Debug, Deserialize)]
struct Struct {
    a: i32,
    b: i32,
    c: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
enum Enum {
    Unit,
    Simple(i32),
    Seq(i32, i32, i32),
    Map { a: i32, b: i32, c: i32 }
}

//////////////////////////////////////////////////////////////////////////

macro_rules! declare_test {
    ($name:ident { $($value:expr => $tokens:expr,)+ }) => {
        #[test]
        fn $name() {
            $(
                // Test ser/de roundtripping
                assert_de_tokens(&$value, $tokens);

                // Test that the tokens are ignorable
                assert_de_tokens_ignore($tokens);
            )+
        }
    }
}

macro_rules! declare_tests {
    ($($name:ident { $($value:expr => $tokens:expr,)+ })+) => {
        $(
            declare_test!($name { $($value => $tokens,)+ });
        )+
    }
}

//////////////////////////////////////////////////////////////////////////

declare_tests! {
    test_bool {
        true => vec![Token::Bool(true)],
        false => vec![Token::Bool(false)],
    }
    test_isize {
        0isize => vec![Token::Isize(0)],
        0isize => vec![Token::I8(0)],
        0isize => vec![Token::I16(0)],
        0isize => vec![Token::I32(0)],
        0isize => vec![Token::I64(0)],
        0isize => vec![Token::Usize(0)],
        0isize => vec![Token::U8(0)],
        0isize => vec![Token::U16(0)],
        0isize => vec![Token::U32(0)],
        0isize => vec![Token::U64(0)],
        0isize => vec![Token::F32(0.)],
        0isize => vec![Token::F64(0.)],
    }
    test_ints {
        0isize => vec![Token::Isize(0)],
        0i8 => vec![Token::I8(0)],
        0i16 => vec![Token::I16(0)],
        0i32 => vec![Token::I32(0)],
        0i64 => vec![Token::I64(0)],
    }
    test_uints {
        0usize => vec![Token::Usize(0)],
        0u8 => vec![Token::U8(0)],
        0u16 => vec![Token::U16(0)],
        0u32 => vec![Token::U32(0)],
        0u64 => vec![Token::U64(0)],
    }
    test_floats {
        0f32 => vec![Token::F32(0.)],
        0f64 => vec![Token::F64(0.)],
    }
    test_char {
        'a' => vec![Token::Char('a')],
        'a' => vec![Token::Str("a")],
        'a' => vec![Token::String("a".to_string())],
    }
    test_string {
        "abc".to_string() => vec![Token::Str("abc")],
        "abc".to_string() => vec![Token::String("abc".to_string())],
        "a".to_string() => vec![Token::Char('a')],
    }
    test_option {
        None::<i32> => vec![Token::Unit],
        None::<i32> => vec![Token::Option(false)],
        Some(1) => vec![Token::I32(1)],
        Some(1) => vec![
            Token::Option(true),
            Token::I32(1),
        ],
    }
    test_result {
        Ok::<i32, i32>(0) => vec![
            Token::EnumStart("Result"),
            Token::Str("Ok"),
            Token::I32(0),
        ],
        Err::<i32, i32>(1) => vec![
            Token::EnumStart("Result"),
            Token::Str("Err"),
            Token::I32(1),
        ],
    }
    test_unit {
        () => vec![Token::Unit],
        () => vec![
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        () => vec![
            Token::SeqStart(None),
            Token::SeqEnd,
        ],
        () => vec![
            Token::TupleStructStart("Anything", Some(0)),
            Token::SeqEnd,
        ],
    }
    test_unit_struct {
        UnitStruct => vec![Token::Unit],
        UnitStruct => vec![
            Token::UnitStruct("UnitStruct"),
        ],
        UnitStruct => vec![
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        UnitStruct => vec![
            Token::SeqStart(None),
            Token::SeqEnd,
        ],
    }
    test_tuple_struct {
        TupleStruct(1, 2, 3) => vec![
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
        TupleStruct(1, 2, 3) => vec![
            Token::SeqStart(None),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
        TupleStruct(1, 2, 3) => vec![
            Token::TupleStructStart("TupleStruct", Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
        TupleStruct(1, 2, 3) => vec![
            Token::TupleStructStart("TupleStruct", None),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_btreeset {
        BTreeSet::<isize>::new() => vec![
            Token::Unit,
        ],
        BTreeSet::<isize>::new() => vec![
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        btreeset![btreeset![], btreeset![1], btreeset![2, 3]] => vec![
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::SeqStart(Some(0)),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(1)),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(2)),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        BTreeSet::<isize>::new() => vec![
            Token::UnitStruct("Anything"),
        ],
        BTreeSet::<isize>::new() => vec![
            Token::TupleStructStart("Anything", Some(0)),
            Token::SeqEnd,
        ],
    }
    test_hashset {
        HashSet::<isize>::new() => vec![
            Token::Unit,
        ],
        HashSet::<isize>::new() => vec![
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        hashset![1, 2, 3] => vec![
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
        HashSet::<isize>::new() => vec![
            Token::UnitStruct("Anything"),
        ],
        HashSet::<isize>::new() => vec![
            Token::TupleStructStart("Anything", Some(0)),
            Token::SeqEnd,
        ],
    }
    test_vec {
        Vec::<isize>::new() => vec![
            Token::Unit,
        ],
        Vec::<isize>::new() => vec![
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        vec![vec![], vec![1], vec![2, 3]] => vec![
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::SeqStart(Some(0)),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(1)),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(2)),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        Vec::<isize>::new() => vec![
            Token::UnitStruct("Anything"),
        ],
        Vec::<isize>::new() => vec![
            Token::TupleStructStart("Anything", Some(0)),
            Token::SeqEnd,
        ],
    }
    test_array {
        [0; 0] => vec![
            Token::Unit,
        ],
        [0; 0] => vec![
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        ([0; 0], [1], [2, 3]) => vec![
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::SeqStart(Some(0)),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(1)),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(2)),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        [0; 0] => vec![
            Token::UnitStruct("Anything"),
        ],
        [0; 0] => vec![
            Token::TupleStructStart("Anything", Some(0)),
            Token::SeqEnd,
        ],
    }
    test_tuple {
        (1,) => vec![
            Token::SeqStart(Some(1)),
                Token::SeqSep,
                Token::I32(1),
            Token::SeqEnd,
        ],
        (1, 2, 3) => vec![
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_btreemap {
        BTreeMap::<isize, isize>::new() => vec![
            Token::Unit,
        ],
        BTreeMap::<isize, isize>::new() => vec![
            Token::MapStart(Some(0)),
            Token::MapEnd,
        ],
        btreemap![1 => 2] => vec![
            Token::MapStart(Some(1)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        btreemap![1 => 2, 3 => 4] => vec![
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),

                Token::MapSep,
                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]] => vec![
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::MapStart(Some(0)),
                Token::MapEnd,

                Token::MapSep,
                Token::I32(2),
                Token::MapStart(Some(2)),
                    Token::MapSep,
                    Token::I32(3),
                    Token::I32(4),

                    Token::MapSep,
                    Token::I32(5),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
        BTreeMap::<isize, isize>::new() => vec![
            Token::UnitStruct("Anything"),
        ],
        BTreeMap::<isize, isize>::new() => vec![
            Token::StructStart("Anything", Some(0)),
            Token::MapEnd,
        ],
    }
    test_hashmap {
        HashMap::<isize, isize>::new() => vec![
            Token::Unit,
        ],
        HashMap::<isize, isize>::new() => vec![
            Token::MapStart(Some(0)),
            Token::MapEnd,
        ],
        hashmap![1 => 2] => vec![
            Token::MapStart(Some(1)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        hashmap![1 => 2, 3 => 4] => vec![
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),

                Token::MapSep,
                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        hashmap![1 => hashmap![], 2 => hashmap![3 => 4, 5 => 6]] => vec![
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::MapStart(Some(0)),
                Token::MapEnd,

                Token::MapSep,
                Token::I32(2),
                Token::MapStart(Some(2)),
                    Token::MapSep,
                    Token::I32(3),
                    Token::I32(4),

                    Token::MapSep,
                    Token::I32(5),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
        HashMap::<isize, isize>::new() => vec![
            Token::UnitStruct("Anything"),
        ],
        HashMap::<isize, isize>::new() => vec![
            Token::StructStart("Anything", Some(0)),
            Token::MapEnd,
        ],
    }
    test_struct {
        Struct { a: 1, b: 2, c: 3 } => vec![
            Token::MapStart(Some(3)),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("b"),
                Token::I32(2),

                Token::MapSep,
                Token::Str("c"),
                Token::I32(3),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 3 } => vec![
            Token::StructStart("Struct", Some(3)),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("b"),
                Token::I32(2),

                Token::MapSep,
                Token::Str("c"),
                Token::I32(3),
            Token::MapEnd,
        ],
    }
    test_enum_unit {
        Enum::Unit => vec![
            Token::EnumUnit("Enum", "Unit"),
        ],
    }
    test_enum_simple {
        Enum::Simple(1) => vec![
            Token::EnumNewtype("Enum", "Simple"),
            Token::I32(1),
        ],
    }
    test_enum_seq {
        Enum::Seq(1, 2, 3) => vec![
            Token::EnumSeqStart("Enum", "Seq", Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_enum_map {
        Enum::Map { a: 1, b: 2, c: 3 } => vec![
            Token::EnumMapStart("Enum", "Map", Some(3)),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("b"),
                Token::I32(2),

                Token::MapSep,
                Token::Str("c"),
                Token::I32(3),
            Token::MapEnd,
        ],
    }
    test_enum_unit_usize {
        Enum::Unit => vec![
            Token::EnumStart("Enum"),
            Token::Usize(0),
            Token::Unit,
        ],
    }
    test_enum_unit_bytes {
        Enum::Unit => vec![
            Token::EnumStart("Enum"),
            Token::Bytes(b"Unit"),
            Token::Unit,
        ],
    }
    test_num_bigint {
        BigInt::from_i64(123).unwrap() => vec![Token::Str("123")],
        BigInt::from_i64(-123).unwrap() => vec![Token::Str("-123")],
    }
    test_num_biguint {
        BigUint::from_i64(123).unwrap() => vec![Token::Str("123")],
    }
    test_num_complex {
        Complex::new(1, 2) => vec![
            Token::SeqStart(Some(2)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),
            Token::SeqEnd,
        ],
    }
    test_num_ratio {
        Ratio::new(1, 2) => vec![
            Token::SeqStart(Some(2)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),
            Token::SeqEnd,
        ],
    }
}
