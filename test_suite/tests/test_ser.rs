// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::net;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::ffi::CString;

#[cfg(unix)]
use std::str;

extern crate serde;

extern crate serde_test;
use self::serde_test::{Error, Token, assert_ser_tokens, assert_ser_tokens_error};

extern crate fnv;
use self::fnv::FnvHasher;

#[macro_use]
mod macros;

//////////////////////////////////////////////////////////////////////////

#[derive(Serialize)]
struct UnitStruct;

#[derive(Serialize)]
struct TupleStruct(i32, i32, i32);

#[derive(Serialize)]
struct Struct {
    a: i32,
    b: i32,
    c: i32,
}

#[derive(Serialize, PartialEq, Debug)]
enum Enum {
    Unit,
    One(i32),
    Seq(i32, i32),
    Map { a: i32, b: i32 },
    #[serde(skip_serializing)]
    SkippedUnit,
    #[serde(skip_serializing)]
    SkippedOne(i32),
    #[serde(skip_serializing)]
    SkippedSeq(i32, i32),
    #[serde(skip_serializing)]
    SkippedMap { _a: i32, _b: i32 },
}

//////////////////////////////////////////////////////////////////////////

macro_rules! declare_tests {
    ($($name:ident { $($value:expr => $tokens:expr,)+ })+) => {
        $(
            #[test]
            fn $name() {
                $(
                    assert_ser_tokens(&$value, $tokens);
                )+
            }
        )+
    }
}

declare_tests! {
    test_unit {
        () => &[Token::Unit],
    }
    test_bool {
        true => &[Token::Bool(true)],
        false => &[Token::Bool(false)],
    }
    test_isizes {
        0i8 => &[Token::I8(0)],
        0i16 => &[Token::I16(0)],
        0i32 => &[Token::I32(0)],
        0i64 => &[Token::I64(0)],
    }
    test_usizes {
        0u8 => &[Token::U8(0)],
        0u16 => &[Token::U16(0)],
        0u32 => &[Token::U32(0)],
        0u64 => &[Token::U64(0)],
    }
    test_floats {
        0f32 => &[Token::F32(0.)],
        0f64 => &[Token::F64(0.)],
    }
    test_char {
        'a' => &[Token::Char('a')],
    }
    test_str {
        "abc" => &[Token::Str("abc")],
        "abc".to_owned() => &[Token::Str("abc")],
    }
    test_option {
        None::<i32> => &[Token::None],
        Some(1) => &[
            Token::Some,
            Token::I32(1),
        ],
    }
    test_result {
        Ok::<i32, i32>(0) => &[
            Token::NewtypeVariant("Result", "Ok"),
            Token::I32(0),
        ],
        Err::<i32, i32>(1) => &[
            Token::NewtypeVariant("Result", "Err"),
            Token::I32(1),
        ],
    }
    test_slice {
        &[0][..0] => &[
            Token::Seq(Some(0)),
            Token::SeqEnd,
        ],
        &[1, 2, 3][..] => &[
            Token::Seq(Some(3)),
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_array {
        [0; 0] => &[
            Token::SeqFixedSize(0),
            Token::SeqEnd,
        ],
        [1, 2, 3] => &[
            Token::SeqFixedSize(3),
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_vec {
        Vec::<isize>::new() => &[
            Token::Seq(Some(0)),
            Token::SeqEnd,
        ],
        vec![vec![], vec![1], vec![2, 3]] => &[
            Token::Seq(Some(3)),
                Token::Seq(Some(0)),
                Token::SeqEnd,

                Token::Seq(Some(1)),
                    Token::I32(1),
                Token::SeqEnd,

                Token::Seq(Some(2)),
                    Token::I32(2),
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
    }
    test_hashset {
        HashSet::<isize>::new() => &[
            Token::Seq(Some(0)),
            Token::SeqEnd,
        ],
        hashset![1] => &[
            Token::Seq(Some(1)),
                Token::I32(1),
            Token::SeqEnd,
        ],
        hashset![FnvHasher @ 1] => &[
            Token::Seq(Some(1)),
                Token::I32(1),
            Token::SeqEnd,
        ],
    }
    test_tuple {
        (1,) => &[
            Token::Tuple(1),
                Token::I32(1),
            Token::TupleEnd,
        ],
        (1, 2, 3) => &[
            Token::Tuple(3),
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleEnd,
        ],
    }
    test_btreemap {
        btreemap![1 => 2] => &[
            Token::Map(Some(1)),
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        btreemap![1 => 2, 3 => 4] => &[
            Token::Map(Some(2)),
                Token::I32(1),
                Token::I32(2),

                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]] => &[
            Token::Map(Some(2)),
                Token::I32(1),
                Token::Map(Some(0)),
                Token::MapEnd,

                Token::I32(2),
                Token::Map(Some(2)),
                    Token::I32(3),
                    Token::I32(4),

                    Token::I32(5),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
    }
    test_hashmap {
        HashMap::<isize, isize>::new() => &[
            Token::Map(Some(0)),
            Token::MapEnd,
        ],
        hashmap![1 => 2] => &[
            Token::Map(Some(1)),
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        hashmap![FnvHasher @ 1 => 2] => &[
            Token::Map(Some(1)),
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
    }
    test_unit_struct {
        UnitStruct => &[Token::UnitStruct("UnitStruct")],
    }
    test_tuple_struct {
        TupleStruct(1, 2, 3) => &[
            Token::TupleStruct("TupleStruct", 3),
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleStructEnd,
        ],
    }
    test_struct {
        Struct { a: 1, b: 2, c: 3 } => &[
            Token::Struct("Struct", 3),
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),

                Token::Str("c"),
                Token::I32(3),
            Token::StructEnd,
        ],
    }
    test_enum {
        Enum::Unit => &[Token::UnitVariant("Enum", "Unit")],
        Enum::One(42) => &[Token::NewtypeVariant("Enum", "One"), Token::I32(42)],
        Enum::Seq(1, 2) => &[
            Token::TupleVariant("Enum", "Seq", 2),
                Token::I32(1),
                Token::I32(2),
            Token::TupleVariantEnd,
        ],
        Enum::Map { a: 1, b: 2 } => &[
            Token::StructVariant("Enum", "Map", 2),
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::StructVariantEnd,
        ],
    }
    test_box {
        Box::new(0i32) => &[Token::I32(0)],
    }
    test_boxed_slice {
        Box::new([0, 1, 2]) => &[
            Token::SeqFixedSize(3),
            Token::I32(0),
            Token::I32(1),
            Token::I32(2),
            Token::SeqEnd,
        ],
    }
    test_duration {
        Duration::new(1, 2) => &[
            Token::Struct("Duration", 2),
                Token::Str("secs"),
                Token::U64(1),

                Token::Str("nanos"),
                Token::U32(2),
            Token::StructEnd,
        ],
    }
    test_range {
        1u32..2u32 => &[
            Token::Struct("Range", 2),
                Token::Str("start"),
                Token::U32(1),

                Token::Str("end"),
                Token::U32(2),
            Token::StructEnd,
        ],
    }
    test_net_ipv4addr {
        "1.2.3.4".parse::<net::Ipv4Addr>().unwrap() => &[Token::Str("1.2.3.4")],
    }
    test_net_ipv6addr {
        "::1".parse::<net::Ipv6Addr>().unwrap() => &[Token::Str("::1")],
    }
    test_net_socketaddr {
        "1.2.3.4:1234".parse::<net::SocketAddr>().unwrap() => &[Token::Str("1.2.3.4:1234")],
        "1.2.3.4:1234".parse::<net::SocketAddrV4>().unwrap() => &[Token::Str("1.2.3.4:1234")],
        "[::1]:1234".parse::<net::SocketAddrV6>().unwrap() => &[Token::Str("[::1]:1234")],
    }
    test_path {
        Path::new("/usr/local/lib") => &[
            Token::Str("/usr/local/lib"),
        ],
    }
    test_path_buf {
        PathBuf::from("/usr/local/lib") => &[
            Token::Str("/usr/local/lib"),
        ],
    }
    test_cstring {
        CString::new("abc").unwrap() => &[
            Token::Bytes(b"abc"),
        ],
    }
    test_cstr {
        (&*CString::new("abc").unwrap()) => &[
            Token::Bytes(b"abc"),
        ],
    }
}

#[cfg(feature = "unstable")]
#[test]
fn test_net_ipaddr() {
    assert_ser_tokens(
        &"1.2.3.4".parse::<net::IpAddr>().unwrap(),
        &[Token::Str("1.2.3.4")],
    );
}

#[test]
#[cfg(unix)]
fn test_cannot_serialize_paths() {
    let path = unsafe { str::from_utf8_unchecked(b"Hello \xF0\x90\x80World") };
    assert_ser_tokens_error(
        &Path::new(path),
        &[],
        Error::Message("path contains invalid UTF-8 characters".to_owned()),
    );

    let mut path_buf = PathBuf::new();
    path_buf.push(path);

    assert_ser_tokens_error(
        &path_buf,
        &[],
        Error::Message("path contains invalid UTF-8 characters".to_owned()),
    );
}

#[test]
fn test_enum_skipped() {
    assert_ser_tokens_error(
        &Enum::SkippedUnit,
        &[],
        Error::Message("the enum variant Enum::SkippedUnit cannot be serialized".to_owned(),),
    );
    assert_ser_tokens_error(
        &Enum::SkippedOne(42),
        &[],
        Error::Message("the enum variant Enum::SkippedOne cannot be serialized".to_owned(),),
    );
    assert_ser_tokens_error(
        &Enum::SkippedSeq(1, 2),
        &[],
        Error::Message("the enum variant Enum::SkippedSeq cannot be serialized".to_owned(),),
    );
    assert_ser_tokens_error(
        &Enum::SkippedMap { _a: 1, _b: 2 },
        &[],
        Error::Message("the enum variant Enum::SkippedMap cannot be serialized".to_owned(),),
    );
}
