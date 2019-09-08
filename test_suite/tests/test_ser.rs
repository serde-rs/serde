#![allow(clippy::unreadable_literal)]
#![cfg_attr(feature = "unstable", feature(never_type))]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ffi::CString;
use std::mem;
use std::net;
use std::num::Wrapping;
use std::ops::Bound;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak as RcWeak};
use std::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI8, AtomicIsize, AtomicU16, AtomicU32, AtomicU8,
    AtomicUsize,
};
use std::sync::{Arc, Weak as ArcWeak};
use std::time::{Duration, UNIX_EPOCH};

#[cfg(unix)]
use std::str;
#[cfg(target_arch = "x86_64")]
use std::sync::atomic::{AtomicI64, AtomicU64};

use fnv::FnvHasher;
use serde::Serialize;
use serde_test::{assert_ser_tokens, assert_ser_tokens_error, Configure, Token};

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

#[derive(PartialEq, Debug)]
struct NotSerializable;

#[derive(Serialize, PartialEq, Debug)]
enum Enum {
    Unit,
    One(i32),
    Seq(i32, i32),
    Map {
        a: i32,
        b: i32,
    },
    #[serde(skip_serializing)]
    SkippedUnit,
    #[serde(skip_serializing)]
    SkippedOne(i32),
    #[serde(skip_serializing)]
    SkippedSeq(i32, i32),
    #[serde(skip_serializing)]
    SkippedMap {
        _a: i32,
        _b: i32,
    },
    OneWithSkipped(#[serde(skip_serializing)] NotSerializable),
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
                    assert_ser_tokens(&$value.$readable(), $tokens);
                )+
            }
        )+
    };

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
            Token::NewtypeVariant { name: "Result", variant: "Ok" },
            Token::I32(0),
        ],
        Err::<i32, i32>(1) => &[
            Token::NewtypeVariant { name: "Result", variant: "Err" },
            Token::I32(1),
        ],
    }
    test_slice {
        &[0][..0] => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        &[1, 2, 3][..] => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_array {
        [0; 0] => &[
            Token::Tuple { len: 0 },
            Token::TupleEnd,
        ],
        [1, 2, 3] => &[
            Token::Tuple { len: 3 },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleEnd,
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
    }
    test_btreeset {
        BTreeSet::<isize>::new() => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        btreeset![1] => &[
            Token::Seq { len: Some(1) },
                Token::I32(1),
            Token::SeqEnd,
        ],
    }
    test_hashset {
        HashSet::<isize>::new() => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        hashset![1] => &[
            Token::Seq { len: Some(1) },
                Token::I32(1),
            Token::SeqEnd,
        ],
        hashset![FnvHasher @ 1] => &[
            Token::Seq { len: Some(1) },
                Token::I32(1),
            Token::SeqEnd,
        ],
    }
    test_tuple {
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
        hashmap![FnvHasher @ 1 => 2] => &[
            Token::Map { len: Some(1) },
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
    }
    test_unit_struct {
        UnitStruct => &[Token::UnitStruct { name: "UnitStruct" }],
    }
    test_tuple_struct {
        TupleStruct(1, 2, 3) => &[
            Token::TupleStruct { name: "TupleStruct", len: 3 },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleStructEnd,
        ],
    }
    test_struct {
        Struct { a: 1, b: 2, c: 3 } => &[
            Token::Struct { name: "Struct", len: 3 },
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
        Enum::Unit => &[
            Token::UnitVariant { name: "Enum", variant: "Unit" },
        ],
        Enum::One(42) => &[
            Token::NewtypeVariant { name: "Enum", variant: "One" },
            Token::I32(42),
        ],
        Enum::Seq(1, 2) => &[
            Token::TupleVariant { name: "Enum", variant: "Seq", len: 2 },
                Token::I32(1),
                Token::I32(2),
            Token::TupleVariantEnd,
        ],
        Enum::Map { a: 1, b: 2 } => &[
            Token::StructVariant { name: "Enum", variant: "Map", len: 2 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::StructVariantEnd,
        ],
        Enum::OneWithSkipped(NotSerializable) => &[
            Token::UnitVariant { name: "Enum", variant: "OneWithSkipped" },
        ],
    }
    test_box {
        Box::new(0i32) => &[Token::I32(0)],
    }
    test_boxed_slice {
        Box::new([0, 1, 2]) => &[
            Token::Tuple { len: 3 },
            Token::I32(0),
            Token::I32(1),
            Token::I32(2),
            Token::TupleEnd,
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
    }
    test_system_time {
        UNIX_EPOCH + Duration::new(1, 200) => &[
            Token::Struct { name: "SystemTime", len: 2 },
                Token::Str("secs_since_epoch"),
                Token::U64(1),

                Token::Str("nanos_since_epoch"),
                Token::U32(200),
            Token::StructEnd,
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
    }
    test_bound {
        Bound::Unbounded::<()> => &[
            Token::Enum { name: "Bound" },
            Token::Str("Unbounded"),
            Token::Unit,
        ],
        Bound::Included(0u8) => &[
            Token::Enum { name: "Bound" },
            Token::Str("Included"),
            Token::U8(0),
        ],
        Bound::Excluded(0u8) => &[
            Token::Enum { name: "Bound" },
            Token::Str("Excluded"),
            Token::U8(0),
        ],
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
    test_rc {
        Rc::new(true) => &[
            Token::Bool(true),
        ],
    }
    test_rc_weak_some {
        {
            let rc = Rc::new(true);
            mem::forget(rc.clone());
            Rc::downgrade(&rc)
        } => &[
            Token::Some,
            Token::Bool(true),
        ],
    }
    test_rc_weak_none {
        RcWeak::<bool>::new() => &[
            Token::None,
        ],
    }
    test_arc {
        Arc::new(true) => &[
            Token::Bool(true),
        ],
    }
    test_arc_weak_some {
        {
            let arc = Arc::new(true);
            mem::forget(arc.clone());
            Arc::downgrade(&arc)
        } => &[
            Token::Some,
            Token::Bool(true),
        ],
    }
    test_arc_weak_none {
        ArcWeak::<bool>::new() => &[
            Token::None,
        ],
    }
    test_wrapping {
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
    test_fmt_arguments {
        format_args!("{}{}", 1, 'a') => &[
            Token::Str("1a"),
        ],
    }
    test_atomic {
        AtomicBool::new(false) => &[Token::Bool(false)],
        AtomicBool::new(true) => &[Token::Bool(true)],
        AtomicI8::new(63i8) => &[Token::I8(63i8)],
        AtomicI16::new(-318i16) => &[Token::I16(-318i16)],
        AtomicI32::new(65792i32) => &[Token::I32(65792i32)],
        AtomicIsize::new(-65792isize) => &[Token::I64(-65792i64)],
        AtomicU8::new(192u8) => &[Token::U8(192u8)],
        AtomicU16::new(510u16) => &[Token::U16(510u16)],
        AtomicU32::new(131072u32) => &[Token::U32(131072u32)],
        AtomicUsize::new(655360usize) => &[Token::U64(655360u64)],
    }
}

#[cfg(target_arch = "x86_64")]
declare_tests! {
    test_atomic64 {
        AtomicI64::new(-4295032832i64) => &[Token::I64(-4295032832i64)],
        AtomicU64::new(12884901888u64) => &[Token::U64(12884901888u64)],
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
            Token::TupleEnd,
        ],
    }
    test_net_ipv6addr_compact {
        net::Ipv6Addr::from(*b"1234567890123456") => &seq![
            Token::Tuple { len: 16 },
            seq b"1234567890123456".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,
        ],
    }
    test_net_ipaddr_compact {
        net::IpAddr::from(*b"1234") => &seq![
            Token::NewtypeVariant { name: "IpAddr", variant: "V4" },

            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,
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
            Token::TupleEnd,
        ],
        net::SocketAddrV4::new(net::Ipv4Addr::from(*b"1234"), 1234) => &seq![
            Token::Tuple { len: 2 },

            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd,
        ],
        net::SocketAddrV6::new(net::Ipv6Addr::from(*b"1234567890123456"), 1234, 0, 0) => &seq![
            Token::Tuple { len: 2 },

            Token::Tuple { len: 16 },
            seq b"1234567890123456".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd,
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

#[test]
#[cfg(unix)]
fn test_cannot_serialize_paths() {
    let path = unsafe { str::from_utf8_unchecked(b"Hello \xF0\x90\x80World") };
    assert_ser_tokens_error(
        &Path::new(path),
        &[],
        "path contains invalid UTF-8 characters",
    );

    let mut path_buf = PathBuf::new();
    path_buf.push(path);

    assert_ser_tokens_error(&path_buf, &[], "path contains invalid UTF-8 characters");
}

#[test]
fn test_cannot_serialize_mutably_borrowed_ref_cell() {
    let ref_cell = RefCell::new(42);
    let _reference = ref_cell.borrow_mut();
    assert_ser_tokens_error(&ref_cell, &[], "already mutably borrowed");
}

#[test]
fn test_enum_skipped() {
    assert_ser_tokens_error(
        &Enum::SkippedUnit,
        &[],
        "the enum variant Enum::SkippedUnit cannot be serialized",
    );
    assert_ser_tokens_error(
        &Enum::SkippedOne(42),
        &[],
        "the enum variant Enum::SkippedOne cannot be serialized",
    );
    assert_ser_tokens_error(
        &Enum::SkippedSeq(1, 2),
        &[],
        "the enum variant Enum::SkippedSeq cannot be serialized",
    );
    assert_ser_tokens_error(
        &Enum::SkippedMap { _a: 1, _b: 2 },
        &[],
        "the enum variant Enum::SkippedMap cannot be serialized",
    );
}

#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
#[test]
fn test_integer128() {
    assert_ser_tokens_error(&1i128, &[], "i128 is not supported");

    assert_ser_tokens_error(&1u128, &[], "u128 is not supported");
}
