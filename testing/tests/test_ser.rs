use std::collections::{BTreeMap, HashMap, HashSet};
use std::net;
use std::path::{Path, PathBuf};
use std::str;

extern crate serde;
extern crate serde_test;
use self::serde_test::{
    Error,
    Token,
    Serializer,
    assert_ser_tokens,
    assert_ser_tokens_error,
};

extern crate fnv;
use self::fnv::FnvHasher;

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

#[derive(Serialize)]
enum Enum {
    Unit,
    One(i32),
    Seq(i32, i32),
    Map { a: i32, b: i32 },
}

//////////////////////////////////////////////////////////////////////////

declare_ser_tests! {
    test_unit {
        () => &[Token::Unit],
    }
    test_bool {
        true => &[Token::Bool(true)],
        false => &[Token::Bool(false)],
    }
    test_isizes {
        0isize => &[Token::Isize(0)],
        0i8 => &[Token::I8(0)],
        0i16 => &[Token::I16(0)],
        0i32 => &[Token::I32(0)],
        0i64 => &[Token::I64(0)],
    }
    test_usizes {
        0usize => &[Token::Usize(0)],
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
        None::<i32> => &[Token::Option(false)],
        Some(1) => &[
            Token::Option(true),
            Token::I32(1),
        ],
    }
    test_result {
        Ok::<i32, i32>(0) => &[
            Token::EnumNewType("Result", "Ok"),
            Token::I32(0),
        ],
        Err::<i32, i32>(1) => &[
            Token::EnumNewType("Result", "Err"),
            Token::I32(1),
        ],
    }
    test_slice {
        &[0][..0] => &[
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        &[1, 2, 3][..] => &[
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
    test_array {
        [0; 0] => &[
            Token::SeqArrayStart(0),
            Token::SeqEnd,
        ],
        [1, 2, 3] => &[
            Token::SeqArrayStart(3),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_vec {
        Vec::<isize>::new() => &[
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        vec![vec![], vec![1], vec![2, 3]] => &[
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
    }
    test_hashset {
        HashSet::<isize>::new() => &[
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        hashset![1] => &[
            Token::SeqStart(Some(1)),
                Token::SeqSep,
                Token::I32(1),
            Token::SeqEnd,
        ],
        hashset![FnvHasher @ 1] => &[
            Token::SeqStart(Some(1)),
                Token::SeqSep,
                Token::I32(1),
            Token::SeqEnd,
        ],
    }
    test_tuple {
        (1,) => &[
            Token::TupleStart(1),
                Token::TupleSep,
                Token::I32(1),
            Token::TupleEnd,
        ],
        (1, 2, 3) => &[
            Token::TupleStart(3),
                Token::TupleSep,
                Token::I32(1),

                Token::TupleSep,
                Token::I32(2),

                Token::TupleSep,
                Token::I32(3),
            Token::TupleEnd,
        ],
    }
    test_btreemap {
        btreemap![1 => 2] => &[
            Token::MapStart(Some(1)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        btreemap![1 => 2, 3 => 4] => &[
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),

                Token::MapSep,
                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]] => &[
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
    }
    test_hashmap {
        HashMap::<isize, isize>::new() => &[
            Token::MapStart(Some(0)),
            Token::MapEnd,
        ],
        hashmap![1 => 2] => &[
            Token::MapStart(Some(1)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        hashmap![FnvHasher @ 1 => 2] => &[
            Token::MapStart(Some(1)),
                Token::MapSep,
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
            Token::TupleStructStart("TupleStruct", 3),
                Token::TupleStructSep,
                Token::I32(1),

                Token::TupleStructSep,
                Token::I32(2),

                Token::TupleStructSep,
                Token::I32(3),
            Token::TupleStructEnd,
        ],
    }
    test_struct {
        Struct { a: 1, b: 2, c: 3 } => &[
            Token::StructStart("Struct", 3),
                Token::StructSep,
                Token::Str("a"),
                Token::I32(1),

                Token::StructSep,
                Token::Str("b"),
                Token::I32(2),

                Token::StructSep,
                Token::Str("c"),
                Token::I32(3),
            Token::StructEnd,
        ],
    }
    test_enum {
        Enum::Unit => &[Token::EnumUnit("Enum", "Unit")],
        Enum::One(42) => &[Token::EnumNewType("Enum", "One"), Token::I32(42)],
        Enum::Seq(1, 2) => &[
            Token::EnumSeqStart("Enum", "Seq", 2),
                Token::EnumSeqSep,
                Token::I32(1),

                Token::EnumSeqSep,
                Token::I32(2),
            Token::EnumSeqEnd,
        ],
        Enum::Map { a: 1, b: 2 } => &[
            Token::EnumMapStart("Enum", "Map", 2),
                Token::EnumMapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::EnumMapSep,
                Token::Str("b"),
                Token::I32(2),
            Token::EnumMapEnd,
        ],
    }
    test_box {
        Box::new(0i32) => &[Token::I32(0)],
    }
    test_boxed_slice {
        Box::new([0, 1, 2]) => &[
            Token::SeqArrayStart(3),
            Token::SeqSep,
            Token::I32(0),
            Token::SeqSep,
            Token::I32(1),
            Token::SeqSep,
            Token::I32(2),
            Token::SeqEnd,
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
}

#[cfg(feature = "unstable")]
#[test]
fn test_net_ipaddr() {
    assert_ser_tokens(
        "1.2.3.4".parse::<net::IpAddr>().unwrap(),
        &[Token::Str("1.2.3.4")],
    );
}

#[test]
fn test_cannot_serialize_paths() {
    let path = unsafe {
        str::from_utf8_unchecked(b"Hello \xF0\x90\x80World")
    };
    assert_ser_tokens_error(
        &Path::new(path),
        &[],
        Error::InvalidValue("Path contains invalid UTF-8 characters".to_owned()));

    let mut path_buf = PathBuf::new();
    path_buf.push(path);

    assert_ser_tokens_error(
        &path_buf,
        &[],
        Error::InvalidValue("Path contains invalid UTF-8 characters".to_owned()));
}
