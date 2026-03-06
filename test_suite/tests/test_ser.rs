#![allow(clippy::derive_partial_eq_without_eq, clippy::unreadable_literal)]
#![cfg_attr(feature = "unstable", feature(never_type))]

use serde_derive::Serialize;
use serde_test::{assert_ser_tokens, assert_ser_tokens_error, Configure, Token};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ffi::CString;
use std::net;
use std::num::{Saturating, Wrapping};
use std::ops::Bound;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak as RcWeak};
#[cfg(unix)]
use std::str;
use std::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI8, AtomicIsize, AtomicU16, AtomicU32, AtomicU8,
    AtomicUsize,
};
#[cfg(target_arch = "x86_64")]
use std::sync::atomic::{AtomicI64, AtomicU64};
use std::sync::{Arc, Mutex, RwLock, Weak as ArcWeak};
use std::time::{Duration, UNIX_EPOCH};

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

#[test]
fn test_unit() {
    assert_ser_tokens(&(), &[Token::Unit]);
}

#[test]
fn test_bool() {
    assert_ser_tokens(&true, &[Token::Bool(true)]);
    assert_ser_tokens(&false, &[Token::Bool(false)]);
}

#[test]
fn test_isizes() {
    assert_ser_tokens(&0i8, &[Token::I8(0)]);
    assert_ser_tokens(&0i16, &[Token::I16(0)]);
    assert_ser_tokens(&0i32, &[Token::I32(0)]);
    assert_ser_tokens(&0i64, &[Token::I64(0)]);
}

#[test]
fn test_usizes() {
    assert_ser_tokens(&0u8, &[Token::U8(0)]);
    assert_ser_tokens(&0u16, &[Token::U16(0)]);
    assert_ser_tokens(&0u32, &[Token::U32(0)]);
    assert_ser_tokens(&0u64, &[Token::U64(0)]);
}

#[test]
fn test_floats() {
    assert_ser_tokens(&0f32, &[Token::F32(0.)]);
    assert_ser_tokens(&0f64, &[Token::F64(0.)]);
}

#[test]
fn test_char() {
    assert_ser_tokens(&'a', &[Token::Char('a')]);
}

#[test]
fn test_str() {
    assert_ser_tokens(&"abc", &[Token::Str("abc")]);
    assert_ser_tokens(&"abc".to_owned(), &[Token::Str("abc")]);
}

#[test]
fn test_option() {
    assert_ser_tokens(&None::<i32>, &[Token::None]);
    assert_ser_tokens(&Some(1), &[Token::Some, Token::I32(1)]);
}

#[test]
fn test_result() {
    assert_ser_tokens(
        &Ok::<i32, i32>(0),
        &[
            Token::NewtypeVariant {
                name: "Result",
                variant: "Ok",
            },
            Token::I32(0),
        ],
    );
    assert_ser_tokens(
        &Err::<i32, i32>(1),
        &[
            Token::NewtypeVariant {
                name: "Result",
                variant: "Err",
            },
            Token::I32(1),
        ],
    );
}

#[test]
fn test_slice() {
    assert_ser_tokens(&[0][..0], &[Token::Seq { len: Some(0) }, Token::SeqEnd]);
    assert_ser_tokens(
        &[1, 2, 3][..],
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
fn test_array() {
    assert_ser_tokens(&[0; 0], &[Token::Tuple { len: 0 }, Token::TupleEnd]);
    assert_ser_tokens(
        &[1, 2, 3],
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
fn test_vec() {
    assert_ser_tokens(
        &Vec::<isize>::new(),
        &[Token::Seq { len: Some(0) }, Token::SeqEnd],
    );
    assert_ser_tokens(
        &vec![vec![], vec![1], vec![2, 3]],
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
}

#[test]
fn test_btreeset() {
    assert_ser_tokens(
        &BTreeSet::<isize>::new(),
        &[Token::Seq { len: Some(0) }, Token::SeqEnd],
    );
    assert_ser_tokens(
        &btreeset![1],
        &[Token::Seq { len: Some(1) }, Token::I32(1), Token::SeqEnd],
    );
}

#[test]
fn test_hashset() {
    assert_ser_tokens(
        &HashSet::<isize>::new(),
        &[Token::Seq { len: Some(0) }, Token::SeqEnd],
    );
    assert_ser_tokens(
        &hashset![1],
        &[Token::Seq { len: Some(1) }, Token::I32(1), Token::SeqEnd],
    );
    assert_ser_tokens(
        &hashset![foldhash::fast::FixedState; 1],
        &[Token::Seq { len: Some(1) }, Token::I32(1), Token::SeqEnd],
    );
}

#[test]
fn test_tuple() {
    assert_ser_tokens(
        &(1,),
        &[Token::Tuple { len: 1 }, Token::I32(1), Token::TupleEnd],
    );
    assert_ser_tokens(
        &(1, 2, 3),
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
    assert_ser_tokens(
        &btreemap![1 => 2],
        &[
            Token::Map { len: Some(1) },
            Token::I32(1),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    assert_ser_tokens(
        &btreemap![1 => 2, 3 => 4],
        &[
            Token::Map { len: Some(2) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::I32(4),
            Token::MapEnd,
        ],
    );
    assert_ser_tokens(
        &btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]],
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
}

#[test]
fn test_hashmap() {
    assert_ser_tokens(
        &HashMap::<isize, isize>::new(),
        &[Token::Map { len: Some(0) }, Token::MapEnd],
    );
    assert_ser_tokens(
        &hashmap![1 => 2],
        &[
            Token::Map { len: Some(1) },
            Token::I32(1),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
    assert_ser_tokens(
        &hashmap![foldhash::fast::FixedState; 1 => 2],
        &[
            Token::Map { len: Some(1) },
            Token::I32(1),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_unit_struct() {
    assert_ser_tokens(&UnitStruct, &[Token::UnitStruct { name: "UnitStruct" }]);
}

#[test]
fn test_tuple_struct() {
    assert_ser_tokens(
        &TupleStruct(1, 2, 3),
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
fn test_struct() {
    assert_ser_tokens(
        &Struct { a: 1, b: 2, c: 3 },
        &[
            Token::Struct {
                name: "Struct",
                len: 3,
            },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("b"),
            Token::I32(2),
            Token::Str("c"),
            Token::I32(3),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_enum() {
    assert_ser_tokens(
        &Enum::Unit,
        &[Token::UnitVariant {
            name: "Enum",
            variant: "Unit",
        }],
    );
    assert_ser_tokens(
        &Enum::One(42),
        &[
            Token::NewtypeVariant {
                name: "Enum",
                variant: "One",
            },
            Token::I32(42),
        ],
    );
    assert_ser_tokens(
        &Enum::Seq(1, 2),
        &[
            Token::TupleVariant {
                name: "Enum",
                variant: "Seq",
                len: 2,
            },
            Token::I32(1),
            Token::I32(2),
            Token::TupleVariantEnd,
        ],
    );
    assert_ser_tokens(
        &Enum::Map { a: 1, b: 2 },
        &[
            Token::StructVariant {
                name: "Enum",
                variant: "Map",
                len: 2,
            },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("b"),
            Token::I32(2),
            Token::StructVariantEnd,
        ],
    );
    assert_ser_tokens(
        &Enum::OneWithSkipped(NotSerializable),
        &[Token::UnitVariant {
            name: "Enum",
            variant: "OneWithSkipped",
        }],
    );
}

#[test]
fn test_box() {
    assert_ser_tokens(&Box::new(0i32), &[Token::I32(0)]);
}

#[test]
fn test_boxed_slice() {
    assert_ser_tokens(
        &Box::new([0, 1, 2]),
        &[
            Token::Tuple { len: 3 },
            Token::I32(0),
            Token::I32(1),
            Token::I32(2),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn test_duration() {
    assert_ser_tokens(
        &Duration::new(1, 2),
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
}

#[test]
fn test_system_time() {
    let system_time = UNIX_EPOCH + Duration::new(1, 200);
    assert_ser_tokens(
        &system_time,
        &[
            Token::Struct {
                name: "SystemTime",
                len: 2,
            },
            Token::Str("secs_since_epoch"),
            Token::U64(1),
            Token::Str("nanos_since_epoch"),
            Token::U32(200),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_range() {
    assert_ser_tokens(
        &(1u32..2u32),
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
}

#[test]
fn test_range_inclusive() {
    assert_ser_tokens(
        &(1u32..=2u32),
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
}

#[test]
fn test_range_from() {
    assert_ser_tokens(
        &(1u32..),
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
}

#[test]
fn test_range_to() {
    assert_ser_tokens(
        &(..2u32),
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
}

#[test]
fn test_range_full() {
    assert_ser_tokens(
        &(..),
        &[
            Token::Struct {
                name: "RangeFull",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_range_to_inclusive() {
    assert_ser_tokens(
        &(..=2u32),
        &[
            Token::Struct {
                name: "RangeToInclusive",
                len: 1,
            },
            Token::Str("end"),
            Token::U32(2),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_bound() {
    assert_ser_tokens(
        &Bound::Unbounded::<()>,
        &[
            Token::Enum { name: "Bound" },
            Token::Str("Unbounded"),
            Token::Unit,
        ],
    );
    assert_ser_tokens(
        &Bound::Included(0u8),
        &[
            Token::Enum { name: "Bound" },
            Token::Str("Included"),
            Token::U8(0),
        ],
    );
    assert_ser_tokens(
        &Bound::Excluded(0u8),
        &[
            Token::Enum { name: "Bound" },
            Token::Str("Excluded"),
            Token::U8(0),
        ],
    );
}

#[test]
fn test_path() {
    assert_ser_tokens(
        &Path::new("/usr/local/lib"),
        &[Token::Str("/usr/local/lib")],
    );
}

#[test]
fn test_path_buf() {
    assert_ser_tokens(
        &PathBuf::from("/usr/local/lib"),
        &[Token::Str("/usr/local/lib")],
    );
}

#[test]
fn test_cstring() {
    assert_ser_tokens(&CString::new("abc").unwrap(), &[Token::Bytes(b"abc")]);
}

#[test]
fn test_cstr() {
    let cstring = CString::new("abc").unwrap();
    assert_ser_tokens(cstring.as_c_str(), &[Token::Bytes(b"abc")]);
}

#[test]
fn test_rc() {
    assert_ser_tokens(&Rc::new(true), &[Token::Bool(true)]);
}

#[test]
fn test_rc_weak_some() {
    let rc = Rc::new(true);
    assert_ser_tokens(&Rc::downgrade(&rc), &[Token::Some, Token::Bool(true)]);
}

#[test]
fn test_rc_weak_none() {
    assert_ser_tokens(&RcWeak::<bool>::new(), &[Token::None]);
}

#[test]
fn test_arc() {
    assert_ser_tokens(&Arc::new(true), &[Token::Bool(true)]);
}

#[test]
fn test_arc_weak_some() {
    let arc = Arc::new(true);
    assert_ser_tokens(&Arc::downgrade(&arc), &[Token::Some, Token::Bool(true)]);
}

#[test]
fn test_arc_weak_none() {
    assert_ser_tokens(&ArcWeak::<bool>::new(), &[Token::None]);
}

#[test]
fn test_wrapping() {
    assert_ser_tokens(&Wrapping(1usize), &[Token::U64(1)]);
}

#[test]
fn test_saturating() {
    assert_ser_tokens(&Saturating(1usize), &[Token::U64(1)]);
}

#[test]
fn test_rc_dst() {
    assert_ser_tokens(&Rc::<str>::from("s"), &[Token::Str("s")]);
    assert_ser_tokens(
        &Rc::<[bool]>::from(&[true][..]),
        &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_arc_dst() {
    assert_ser_tokens(&Arc::<str>::from("s"), &[Token::Str("s")]);
    assert_ser_tokens(
        &Arc::<[bool]>::from(&[true][..]),
        &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_fmt_arguments() {
    assert_ser_tokens(&format_args!("{}{}", 1, 'a'), &[Token::Str("1a")]);
}

#[test]
fn test_atomic() {
    assert_ser_tokens(&AtomicBool::new(false), &[Token::Bool(false)]);
    assert_ser_tokens(&AtomicBool::new(true), &[Token::Bool(true)]);
    assert_ser_tokens(&AtomicI8::new(63i8), &[Token::I8(63i8)]);
    assert_ser_tokens(&AtomicI16::new(-318i16), &[Token::I16(-318i16)]);
    assert_ser_tokens(&AtomicI32::new(65792i32), &[Token::I32(65792i32)]);
    assert_ser_tokens(&AtomicIsize::new(-65792isize), &[Token::I64(-65792i64)]);
    assert_ser_tokens(&AtomicU8::new(192u8), &[Token::U8(192u8)]);
    assert_ser_tokens(&AtomicU16::new(510u16), &[Token::U16(510u16)]);
    assert_ser_tokens(&AtomicU32::new(131072u32), &[Token::U32(131072u32)]);
    assert_ser_tokens(&AtomicUsize::new(655360usize), &[Token::U64(655360u64)]);
}

#[cfg(target_arch = "x86_64")]
#[test]
fn test_atomic64() {
    assert_ser_tokens(
        &AtomicI64::new(-4295032832i64),
        &[Token::I64(-4295032832i64)],
    );
    assert_ser_tokens(
        &AtomicU64::new(12884901888u64),
        &[Token::U64(12884901888u64)],
    );
}

#[test]
fn test_net_ipv4addr_readable() {
    assert_ser_tokens(
        &"1.2.3.4".parse::<net::Ipv4Addr>().unwrap().readable(),
        &[Token::Str("1.2.3.4")],
    );
}

#[test]
fn test_net_ipv6addr_readable() {
    assert_ser_tokens(
        &"::1".parse::<net::Ipv6Addr>().unwrap().readable(),
        &[Token::Str("::1")],
    );
}

#[test]
fn test_net_ipaddr_readable() {
    assert_ser_tokens(
        &"1.2.3.4".parse::<net::IpAddr>().unwrap().readable(),
        &[Token::Str("1.2.3.4")],
    );
}

#[test]
fn test_net_socketaddr_readable() {
    assert_ser_tokens(
        &"1.2.3.4:1234"
            .parse::<net::SocketAddr>()
            .unwrap()
            .readable(),
        &[Token::Str("1.2.3.4:1234")],
    );
    assert_ser_tokens(
        &"1.2.3.4:1234"
            .parse::<net::SocketAddrV4>()
            .unwrap()
            .readable(),
        &[Token::Str("1.2.3.4:1234")],
    );
    assert_ser_tokens(
        &"[::1]:1234"
            .parse::<net::SocketAddrV6>()
            .unwrap()
            .readable(),
        &[Token::Str("[::1]:1234")],
    );
}

#[test]
fn test_net_ipv4addr_compact() {
    assert_ser_tokens(
        &net::Ipv4Addr::from(*b"1234").compact(),
        &seq![
            Token::Tuple { len: 4 },
            b"1234".iter().copied().map(Token::U8),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn test_net_ipv6addr_compact() {
    assert_ser_tokens(
        &net::Ipv6Addr::from(*b"1234567890123456").compact(),
        &seq![
            Token::Tuple { len: 16 },
            b"1234567890123456".iter().copied().map(Token::U8),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn test_net_ipaddr_compact() {
    assert_ser_tokens(
        &net::IpAddr::from(*b"1234").compact(),
        &seq![
            Token::NewtypeVariant {
                name: "IpAddr",
                variant: "V4"
            },
            Token::Tuple { len: 4 },
            b"1234".iter().copied().map(Token::U8),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn test_net_socketaddr_compact() {
    assert_ser_tokens(
        &net::SocketAddr::from((*b"1234567890123456", 1234)).compact(),
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
            Token::TupleEnd,
        ],
    );
    assert_ser_tokens(
        &net::SocketAddrV4::new(net::Ipv4Addr::from(*b"1234"), 1234).compact(),
        &seq![
            Token::Tuple { len: 2 },
            Token::Tuple { len: 4 },
            b"1234".iter().copied().map(Token::U8),
            Token::TupleEnd,
            Token::U16(1234),
            Token::TupleEnd,
        ],
    );
    assert_ser_tokens(
        &net::SocketAddrV6::new(net::Ipv6Addr::from(*b"1234567890123456"), 1234, 0, 0).compact(),
        &seq![
            Token::Tuple { len: 2 },
            Token::Tuple { len: 16 },
            b"1234567890123456".iter().copied().map(Token::U8),
            Token::TupleEnd,
            Token::U16(1234),
            Token::TupleEnd,
        ],
    );
}

#[cfg(feature = "unstable")]
#[test]
fn test_never_result() {
    assert_ser_tokens(
        &Ok::<u8, !>(0),
        &[
            Token::NewtypeVariant {
                name: "Result",
                variant: "Ok",
            },
            Token::U8(0),
        ],
    );
}

#[test]
#[cfg(unix)]
fn test_cannot_serialize_paths() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    assert_ser_tokens_error(
        &Path::new(OsStr::from_bytes(b"Hello \xF0\x90\x80World")),
        &[],
        "path contains invalid UTF-8 characters",
    );
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

#[test]
fn test_integer128() {
    assert_ser_tokens_error(&1i128, &[], "i128 is not supported");

    assert_ser_tokens_error(&1u128, &[], "u128 is not supported");
}

#[test]
fn test_refcell_dst() {
    assert_ser_tokens(
        &RefCell::new([true]) as &RefCell<[bool]>,
        &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_mutex_dst() {
    assert_ser_tokens(
        &Mutex::new([true]) as &Mutex<[bool]>,
        &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_rwlock_dst() {
    assert_ser_tokens(
        &RwLock::new([true]) as &RwLock<[bool]>,
        &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    );
}
