extern crate serde_test;
use self::serde_test::{
    Token,
    assert_tokens,
    assert_ser_tokens,
    assert_de_tokens,
};

use std::marker::PhantomData;

// That tests that the derived Serialize implementation doesn't trigger
// any warning about `serializer` not being used, in case of empty enums.
#[derive(Serialize)]
#[allow(dead_code)]
#[deny(unused_variables)]
enum Void {}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NamedUnit;

#[derive(Debug, PartialEq, Serialize)]
struct SerNamedTuple<'a, 'b, A: 'a, B: 'b, C>(&'a A, &'b mut B, C);

#[derive(Debug, PartialEq, Deserialize)]
struct DeNamedTuple<A, B, C>(A, B, C);

#[derive(Debug, PartialEq, Serialize)]
struct SerNamedMap<'a, 'b, A: 'a, B: 'b, C> {
    a: &'a A,
    b: &'b mut B,
    c: C,
}

#[derive(Debug, PartialEq, Deserialize)]
struct DeNamedMap<A, B, C> {
    a: A,
    b: B,
    c: C,
}

#[derive(Debug, PartialEq, Serialize)]
enum SerEnum<'a, B: 'a, C: 'a, D> where D: 'a {
    Unit,
    Seq(
        i8,
        B,
        &'a C,
        &'a mut D,
    ),
    Map {
        a: i8,
        b: B,
        c: &'a C,
        d: &'a mut D,
    },

    // Make sure we can support more than one variant.
    _Unit2,
    _Seq2(
        i8,
        B,
        &'a C,
        &'a mut D,
    ),
    _Map2 {
        a: i8,
        b: B,
        c: &'a C,
        d: &'a mut D,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum DeEnum<B, C, D> {
    Unit,
    Seq(
        i8,
        B,
        C,
        D,
    ),
    Map {
        a: i8,
        b: B,
        c: C,
        d: D,
    },

    // Make sure we can support more than one variant.
    _Unit2,
    _Seq2(
        i8,
        B,
        C,
        D,
    ),
    _Map2 {
        a: i8,
        b: B,
        c: C,
        d: D,
    },
}

#[derive(Serialize)]
enum Lifetimes<'a> {
    LifetimeSeq(&'a i32),
    NoLifetimeSeq(i32),
    LifetimeMap { a: &'a i32 },
    NoLifetimeMap { a: i32 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericStruct<T> {
    x: T,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericNewTypeStruct<T>(T);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericTupleStruct<T, U>(T, U);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum GenericEnum<T, U> {
    Unit,
    NewType(T),
    Seq(T, U),
    Map { x: T, y: U },
}

trait AssociatedType {
    type X;
}

impl AssociatedType for i32 {
    type X = i32;
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DefaultTyParam<T: AssociatedType<X=i32> = i32> {
    phantom: PhantomData<T>
}

#[test]
fn test_named_unit() {
    assert_tokens(
        &NamedUnit,
        &[Token::UnitStruct("NamedUnit")]
    );
}

#[test]
fn test_ser_named_tuple() {
    let a = 5;
    let mut b = 6;
    let c = 7;
    assert_ser_tokens(
        &SerNamedTuple(&a, &mut b, c),
        &[
            Token::TupleStructStart("SerNamedTuple", 3),
            Token::TupleStructSep,
            Token::I32(5),

            Token::TupleStructSep,
            Token::I32(6),

            Token::TupleStructSep,
            Token::I32(7),

            Token::TupleStructEnd,
        ],
    );
}

#[test]
fn test_de_named_tuple() {
    assert_de_tokens(
        &DeNamedTuple(5, 6, 7),
        &[
            Token::SeqStart(Some(3)),
            Token::SeqSep,
            Token::I32(5),

            Token::SeqSep,
            Token::I32(6),

            Token::SeqSep,
            Token::I32(7),

            Token::SeqEnd,
        ]
    );

    assert_de_tokens(
        &DeNamedTuple(5, 6, 7),
        &[
            Token::TupleStructStart("DeNamedTuple", 3),
            Token::TupleStructSep,
            Token::I32(5),

            Token::TupleStructSep,
            Token::I32(6),

            Token::TupleStructSep,
            Token::I32(7),

            Token::TupleStructEnd,
        ]
    );
}

#[test]
fn test_ser_named_map() {
    let a = 5;
    let mut b = 6;
    let c = 7;

    assert_ser_tokens(
        &SerNamedMap {
            a: &a,
            b: &mut b,
            c: c,
        },
        &[
            Token::StructStart("SerNamedMap", 3),

            Token::StructSep,
            Token::Str("a"),
            Token::I32(5),

            Token::StructSep,
            Token::Str("b"),
            Token::I32(6),

            Token::StructSep,
            Token::Str("c"),
            Token::I32(7),

            Token::StructEnd,
        ]
    );
}

#[test]
fn test_de_named_map() {
    assert_de_tokens(
        &DeNamedMap {
            a: 5,
            b: 6,
            c: 7,
        },
        &[
            Token::StructStart("DeNamedMap", 3),

            Token::StructSep,
            Token::Str("a"),
            Token::I32(5),

            Token::StructSep,
            Token::Str("b"),
            Token::I32(6),

            Token::StructSep,
            Token::Str("c"),
            Token::I32(7),

            Token::StructEnd,
        ]
    );
}

#[test]
fn test_ser_enum_unit() {
    assert_ser_tokens(
        &SerEnum::Unit::<u32, u32, u32>,
        &[
            Token::EnumStart("SerEnum", &["Unit", "Seq", "Map", "_Unit2", "_Seq2", "_Map2"]),
            Token::EnumUnit(0),
        ]
    );
}

#[test]
fn test_ser_enum_seq() {
    let a = 1;
    let b = 2;
    let c = 3;
    let mut d = 4;

    assert_ser_tokens(
        &SerEnum::Seq(
            a,
            b,
            &c,
            &mut d,
        ),
        &[
            Token::EnumStart("SerEnum", &["Unit", "Seq", "Map", "_Unit2", "_Seq2", "_Map2"]),
            Token::EnumSeqStart(1, 4),

            Token::EnumSeqSep,
            Token::I8(1),

            Token::EnumSeqSep,
            Token::I32(2),

            Token::EnumSeqSep,
            Token::I32(3),

            Token::EnumSeqSep,
            Token::I32(4),

            Token::EnumSeqEnd,
        ],
    );
}

#[test]
fn test_ser_enum_map() {
    let a = 1;
    let b = 2;
    let c = 3;
    let mut d = 4;

    assert_ser_tokens(
        &SerEnum::Map {
            a: a,
            b: b,
            c: &c,
            d: &mut d,
        },
        &[
            Token::EnumStart("SerEnum", &["Unit", "Seq", "Map", "_Unit2", "_Seq2", "_Map2"]),
            Token::EnumMapStart(2, 4),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::EnumMapSep,
            Token::Str("b"),
            Token::I32(2),

            Token::EnumMapSep,
            Token::Str("c"),
            Token::I32(3),

            Token::EnumMapSep,
            Token::Str("d"),
            Token::I32(4),

            Token::EnumMapEnd,
        ],
    );
}

const DEENUM: &'static[&'static str] = &["Unit", "Seq", "Map", "_Unit2", "_Seq2", "_Map2"];

#[test]
fn test_de_enum_unit() {
    assert_tokens(
        &DeEnum::Unit::<u32, u32, u32>,
        &[
            Token::EnumStart("DeEnum", DEENUM),
            Token::EnumUnit(0),
        ],
    );
}

#[test]
fn test_de_enum_seq() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;

    assert_tokens(
        &DeEnum::Seq(
            a,
            b,
            c,
            d,
        ),
        &[
            Token::EnumStart("DeEnum", DEENUM),
            Token::EnumSeqStart(1, 4),

            Token::EnumSeqSep,
            Token::I8(1),

            Token::EnumSeqSep,
            Token::I32(2),

            Token::EnumSeqSep,
            Token::I32(3),

            Token::EnumSeqSep,
            Token::I32(4),

            Token::EnumSeqEnd,
        ],
    );
}

#[test]
fn test_de_enum_map() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;

    assert_tokens(
        &DeEnum::Map {
            a: a,
            b: b,
            c: c,
            d: d,
        },
        &[
            Token::EnumStart("DeEnum", DEENUM),
            Token::EnumMapStart(2, 4),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::EnumMapSep,
            Token::Str("b"),
            Token::I32(2),

            Token::EnumMapSep,
            Token::Str("c"),
            Token::I32(3),

            Token::EnumMapSep,
            Token::Str("d"),
            Token::I32(4),

            Token::EnumMapEnd,
        ],
    );
}

const LIFETIMES: &'static[&'static str] = &["LifetimeSeq", "NoLifetimeSeq", "LifetimeMap", "NoLifetimeMap"];

#[test]
fn test_lifetimes() {
    let value = 5;

    assert_ser_tokens(
        &Lifetimes::LifetimeSeq(&value),
        &[
            Token::EnumStart("Lifetimes", LIFETIMES),
            Token::EnumNewType(0),
            Token::I32(5),
        ]
    );

    assert_ser_tokens(
        &Lifetimes::NoLifetimeSeq(5),
        &[
            Token::EnumStart("Lifetimes", LIFETIMES),
            Token::EnumNewType(1),
            Token::I32(5),
        ]
    );

    assert_ser_tokens(
        &Lifetimes::LifetimeMap { a: &value },
        &[
            Token::EnumStart("Lifetimes", LIFETIMES),
            Token::EnumMapStart(2, 1),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I32(5),

            Token::EnumMapEnd,
        ]
    );

    assert_ser_tokens(
        &Lifetimes::NoLifetimeMap { a: 5 },
        &[
            Token::EnumStart("Lifetimes", LIFETIMES),
            Token::EnumMapStart(3, 1),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I32(5),

            Token::EnumMapEnd,
        ]
    );
}

#[test]
fn test_generic_struct() {
    assert_tokens(
        &GenericStruct { x: 5u32 },
        &[
            Token::StructStart("GenericStruct", 1),

            Token::StructSep,
            Token::Str("x"),
            Token::U32(5),

            Token::StructEnd,
        ]
    );
}

#[test]
fn test_generic_newtype_struct() {
    assert_tokens(
        &GenericNewTypeStruct(5u32),
        &[
            Token::StructNewType("GenericNewTypeStruct"),
            Token::U32(5),
        ]
    );
}

#[test]
fn test_generic_tuple_struct() {
    assert_tokens(
        &GenericTupleStruct(5u32, 6u32),
        &[
            Token::TupleStructStart("GenericTupleStruct", 2),

            Token::TupleStructSep,
            Token::U32(5),

            Token::TupleStructSep,
            Token::U32(6),

            Token::TupleStructEnd,
        ]
    );
}

const GENERIC: &'static [&'static str] = &["Unit", "NewType", "Seq", "Map"];

#[test]
fn test_generic_enum_unit() {
    assert_tokens(
        &GenericEnum::Unit::<u32, u32>,
        &[
            Token::EnumStart("GenericEnum", GENERIC),
            Token::EnumUnit(0),
        ]
    );
}

#[test]
fn test_generic_enum_newtype() {
    assert_tokens(
        &GenericEnum::NewType::<u32, u32>(5),
        &[
            Token::EnumStart("GenericEnum", GENERIC),
            Token::EnumNewType(1),
            Token::U32(5),
        ]
    );
}

#[test]
fn test_generic_enum_seq() {
    assert_tokens(
        &GenericEnum::Seq::<u32, u32>(5, 6),
        &[
            Token::EnumStart("GenericEnum", GENERIC),
            Token::EnumSeqStart(2, 2),

            Token::EnumSeqSep,
            Token::U32(5),

            Token::EnumSeqSep,
            Token::U32(6),

            Token::EnumSeqEnd,
        ]
    );
}

#[test]
fn test_generic_enum_map() {
    assert_tokens(
        &GenericEnum::Map::<u32, u32> { x: 5, y: 6 },
        &[
            Token::EnumStart("GenericEnum", GENERIC),
            Token::EnumMapStart(3, 2),

            Token::EnumMapSep,
            Token::Str("x"),
            Token::U32(5),

            Token::EnumMapSep,
            Token::Str("y"),
            Token::U32(6),

            Token::EnumMapEnd,
        ]
    );
}

#[test]
fn test_default_ty_param() {
    assert_tokens(
        &DefaultTyParam::<i32> { phantom: PhantomData },
        &[
            Token::StructStart("DefaultTyParam", 1),

            Token::StructSep,
            Token::Str("phantom"),
            Token::UnitStruct("PhantomData"),

            Token::StructEnd,
        ]
    );
}

#[test]
fn test_enum_state_field() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum SomeEnum {
        Key { key: char, state: bool },
    }

    const SOME: &'static [&'static str] = &["Key"];

    assert_tokens(
        &SomeEnum::Key { key: 'a', state: true },
        &[
            Token::EnumStart("SomeEnum", SOME),
            Token::EnumMapStart(0, 2),

            Token::EnumMapSep,
            Token::Str("key"),
            Token::Char('a'),

            Token::EnumMapSep,
            Token::Str("state"),
            Token::Bool(true),

            Token::EnumMapEnd,
        ]
    );
}
