#![deny(trivial_numeric_casts)]
#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::enum_variant_names,
    clippy::redundant_field_names,
    clippy::too_many_lines
)]

mod bytes;

use serde::{Deserialize, Serialize};
use serde_test::{
    assert_de_tokens, assert_de_tokens_error, assert_ser_tokens, assert_tokens, Token,
};

use std::collections::BTreeMap;
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
enum SerEnum<'a, B: 'a, C: 'a, D>
where
    D: 'a,
{
    Unit,
    Seq(i8, B, &'a C, &'a mut D),
    Map { a: i8, b: B, c: &'a C, d: &'a mut D },

    // Make sure we can support more than one variant.
    _Unit2,
    _Seq2(i8, B, &'a C, &'a mut D),
    _Map2 { a: i8, b: B, c: &'a C, d: &'a mut D },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum DeEnum<B, C, D> {
    Unit,
    Seq(i8, B, C, D),
    Map { a: i8, b: B, c: C, d: D },

    // Make sure we can support more than one variant.
    _Unit2,
    _Seq2(i8, B, C, D),
    _Map2 { a: i8, b: B, c: C, d: D },
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
struct DefaultTyParam<T: AssociatedType<X = i32> = i32> {
    phantom: PhantomData<T>,
}

#[test]
fn test_named_unit() {
    assert_tokens(&NamedUnit, &[Token::UnitStruct { name: "NamedUnit" }]);
}

#[test]
fn test_ser_named_tuple() {
    let a = 5;
    let mut b = 6;
    let c = 7;
    assert_ser_tokens(
        &SerNamedTuple(&a, &mut b, c),
        &[
            Token::TupleStruct {
                name: "SerNamedTuple",
                len: 3,
            },
            Token::I32(5),
            Token::I32(6),
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
            Token::Seq { len: Some(3) },
            Token::I32(5),
            Token::I32(6),
            Token::I32(7),
            Token::SeqEnd,
        ],
    );

    assert_de_tokens(
        &DeNamedTuple(5, 6, 7),
        &[
            Token::TupleStruct {
                name: "DeNamedTuple",
                len: 3,
            },
            Token::I32(5),
            Token::I32(6),
            Token::I32(7),
            Token::TupleStructEnd,
        ],
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
            Token::Struct {
                name: "SerNamedMap",
                len: 3,
            },
            Token::Str("a"),
            Token::I32(5),
            Token::Str("b"),
            Token::I32(6),
            Token::Str("c"),
            Token::I32(7),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_de_named_map() {
    assert_de_tokens(
        &DeNamedMap { a: 5, b: 6, c: 7 },
        &[
            Token::Struct {
                name: "DeNamedMap",
                len: 3,
            },
            Token::Str("a"),
            Token::I32(5),
            Token::Str("b"),
            Token::I32(6),
            Token::Str("c"),
            Token::I32(7),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_ser_enum_unit() {
    assert_ser_tokens(
        &SerEnum::Unit::<u32, u32, u32>,
        &[Token::UnitVariant {
            name: "SerEnum",
            variant: "Unit",
        }],
    );
}

#[test]
fn test_ser_enum_seq() {
    let a = 1;
    let b = 2;
    let c = 3;
    let mut d = 4;

    assert_ser_tokens(
        &SerEnum::Seq(a, b, &c, &mut d),
        &[
            Token::TupleVariant {
                name: "SerEnum",
                variant: "Seq",
                len: 4,
            },
            Token::I8(1),
            Token::I32(2),
            Token::I32(3),
            Token::I32(4),
            Token::TupleVariantEnd,
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
            Token::StructVariant {
                name: "SerEnum",
                variant: "Map",
                len: 4,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("b"),
            Token::I32(2),
            Token::Str("c"),
            Token::I32(3),
            Token::Str("d"),
            Token::I32(4),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_de_enum_unit() {
    assert_tokens(
        &DeEnum::Unit::<u32, u32, u32>,
        &[Token::UnitVariant {
            name: "DeEnum",
            variant: "Unit",
        }],
    );
}

#[test]
fn test_de_enum_seq() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;

    assert_tokens(
        &DeEnum::Seq(a, b, c, d),
        &[
            Token::TupleVariant {
                name: "DeEnum",
                variant: "Seq",
                len: 4,
            },
            Token::I8(1),
            Token::I32(2),
            Token::I32(3),
            Token::I32(4),
            Token::TupleVariantEnd,
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
            Token::StructVariant {
                name: "DeEnum",
                variant: "Map",
                len: 4,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("b"),
            Token::I32(2),
            Token::Str("c"),
            Token::I32(3),
            Token::Str("d"),
            Token::I32(4),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_lifetimes() {
    let value = 5;

    assert_ser_tokens(
        &Lifetimes::LifetimeSeq(&value),
        &[
            Token::NewtypeVariant {
                name: "Lifetimes",
                variant: "LifetimeSeq",
            },
            Token::I32(5),
        ],
    );

    assert_ser_tokens(
        &Lifetimes::NoLifetimeSeq(5),
        &[
            Token::NewtypeVariant {
                name: "Lifetimes",
                variant: "NoLifetimeSeq",
            },
            Token::I32(5),
        ],
    );

    assert_ser_tokens(
        &Lifetimes::LifetimeMap { a: &value },
        &[
            Token::StructVariant {
                name: "Lifetimes",
                variant: "LifetimeMap",
                len: 1,
            },
            Token::Str("a"),
            Token::I32(5),
            Token::StructVariantEnd,
        ],
    );

    assert_ser_tokens(
        &Lifetimes::NoLifetimeMap { a: 5 },
        &[
            Token::StructVariant {
                name: "Lifetimes",
                variant: "NoLifetimeMap",
                len: 1,
            },
            Token::Str("a"),
            Token::I32(5),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_generic_struct() {
    assert_tokens(
        &GenericStruct { x: 5u32 },
        &[
            Token::Struct {
                name: "GenericStruct",
                len: 1,
            },
            Token::Str("x"),
            Token::U32(5),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_generic_newtype_struct() {
    assert_tokens(
        &GenericNewTypeStruct(5u32),
        &[
            Token::NewtypeStruct {
                name: "GenericNewTypeStruct",
            },
            Token::U32(5),
        ],
    );
}

#[test]
fn test_untagged_newtype_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    enum E {
        Newtype(GenericNewTypeStruct<u32>),
        Null,
    }

    assert_tokens(
        &E::Newtype(GenericNewTypeStruct(5u32)),
        &[
            Token::NewtypeStruct {
                name: "GenericNewTypeStruct",
            },
            Token::U32(5),
        ],
    );
}

#[test]
fn test_adjacently_tagged_newtype_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "t", content = "c")]
    enum E {
        Newtype(GenericNewTypeStruct<u32>),
        Null,
    }

    assert_de_tokens(
        &E::Newtype(GenericNewTypeStruct(5u32)),
        &[
            Token::Struct { name: "E", len: 2 },
            Token::Str("c"),
            Token::NewtypeStruct {
                name: "GenericNewTypeStruct",
            },
            Token::U32(5),
            Token::Str("t"),
            Token::Str("Newtype"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_generic_tuple_struct() {
    assert_tokens(
        &GenericTupleStruct(5u32, 6u32),
        &[
            Token::TupleStruct {
                name: "GenericTupleStruct",
                len: 2,
            },
            Token::U32(5),
            Token::U32(6),
            Token::TupleStructEnd,
        ],
    );
}

#[test]
fn test_generic_enum_unit() {
    assert_tokens(
        &GenericEnum::Unit::<u32, u32>,
        &[Token::UnitVariant {
            name: "GenericEnum",
            variant: "Unit",
        }],
    );
}

#[test]
fn test_generic_enum_newtype() {
    assert_tokens(
        &GenericEnum::NewType::<u32, u32>(5),
        &[
            Token::NewtypeVariant {
                name: "GenericEnum",
                variant: "NewType",
            },
            Token::U32(5),
        ],
    );
}

#[test]
fn test_generic_enum_seq() {
    assert_tokens(
        &GenericEnum::Seq::<u32, u32>(5, 6),
        &[
            Token::TupleVariant {
                name: "GenericEnum",
                variant: "Seq",
                len: 2,
            },
            Token::U32(5),
            Token::U32(6),
            Token::TupleVariantEnd,
        ],
    );
}

#[test]
fn test_generic_enum_map() {
    assert_tokens(
        &GenericEnum::Map::<u32, u32> { x: 5, y: 6 },
        &[
            Token::StructVariant {
                name: "GenericEnum",
                variant: "Map",
                len: 2,
            },
            Token::Str("x"),
            Token::U32(5),
            Token::Str("y"),
            Token::U32(6),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_default_ty_param() {
    assert_tokens(
        &DefaultTyParam::<i32> {
            phantom: PhantomData,
        },
        &[
            Token::Struct {
                name: "DefaultTyParam",
                len: 1,
            },
            Token::Str("phantom"),
            Token::UnitStruct {
                name: "PhantomData",
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_enum_state_field() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum SomeEnum {
        Key { key: char, state: bool },
    }

    assert_tokens(
        &SomeEnum::Key {
            key: 'a',
            state: true,
        },
        &[
            Token::StructVariant {
                name: "SomeEnum",
                variant: "Key",
                len: 2,
            },
            Token::Str("key"),
            Token::Char('a'),
            Token::Str("state"),
            Token::Bool(true),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_untagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    enum Untagged {
        A { a: u8 },
        B { b: u8 },
        C,
        D(u8),
        E(String),
        F(u8, u8),
    }

    assert_tokens(
        &Untagged::A { a: 1 },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("a"),
            Token::U8(1),
            Token::StructEnd,
        ],
    );

    assert_tokens(
        &Untagged::B { b: 2 },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("b"),
            Token::U8(2),
            Token::StructEnd,
        ],
    );

    // Serializes to unit, deserializes from either depending on format's
    // preference.
    assert_tokens(&Untagged::C, &[Token::Unit]);
    assert_de_tokens(&Untagged::C, &[Token::None]);

    assert_tokens(&Untagged::D(4), &[Token::U8(4)]);
    assert_tokens(&Untagged::E("e".to_owned()), &[Token::Str("e")]);

    assert_tokens(
        &Untagged::F(1, 2),
        &[
            Token::Tuple { len: 2 },
            Token::U8(1),
            Token::U8(2),
            Token::TupleEnd,
        ],
    );

    assert_de_tokens_error::<Untagged>(
        &[Token::Tuple { len: 1 }, Token::U8(1), Token::TupleEnd],
        "data did not match any variant of untagged enum Untagged",
    );

    assert_de_tokens_error::<Untagged>(
        &[
            Token::Tuple { len: 3 },
            Token::U8(1),
            Token::U8(2),
            Token::U8(3),
            Token::TupleEnd,
        ],
        "data did not match any variant of untagged enum Untagged",
    );
}

#[test]
fn test_internally_tagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Newtype(BTreeMap<String, String>);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Struct {
        f: u8,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum InternallyTagged {
        A { a: u8 },
        B,
        C(BTreeMap<String, String>),
        D(Newtype),
        E(Struct),
    }

    assert_tokens(
        &InternallyTagged::A { a: 1 },
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("A"),
            Token::Str("a"),
            Token::U8(1),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::A { a: 1 },
        &[
            Token::Seq { len: Some(2) },
            Token::Str("A"),
            Token::U8(1),
            Token::SeqEnd,
        ],
    );

    assert_tokens(
        &InternallyTagged::B,
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 1,
            },
            Token::Str("type"),
            Token::Str("B"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::B,
        &[Token::Seq { len: Some(1) }, Token::Str("B"), Token::SeqEnd],
    );

    assert_tokens(
        &InternallyTagged::C(BTreeMap::new()),
        &[
            Token::Map { len: Some(1) },
            Token::Str("type"),
            Token::Str("C"),
            Token::MapEnd,
        ],
    );

    assert_de_tokens_error::<InternallyTagged>(
        &[
            Token::Seq { len: Some(2) },
            Token::Str("C"),
            Token::Map { len: Some(0) },
            Token::MapEnd,
            Token::SeqEnd,
        ],
        "invalid type: sequence, expected a map",
    );

    assert_tokens(
        &InternallyTagged::D(Newtype(BTreeMap::new())),
        &[
            Token::Map { len: Some(1) },
            Token::Str("type"),
            Token::Str("D"),
            Token::MapEnd,
        ],
    );

    assert_tokens(
        &InternallyTagged::E(Struct { f: 6 }),
        &[
            Token::Struct {
                name: "Struct",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("E"),
            Token::Str("f"),
            Token::U8(6),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::E(Struct { f: 6 }),
        &[
            Token::Seq { len: Some(2) },
            Token::Str("E"),
            Token::U8(6),
            Token::SeqEnd,
        ],
    );

    assert_de_tokens_error::<InternallyTagged>(
        &[Token::Map { len: Some(0) }, Token::MapEnd],
        "missing field `type`",
    );

    assert_de_tokens_error::<InternallyTagged>(
        &[
            Token::Map { len: Some(1) },
            Token::Str("type"),
            Token::Str("Z"),
            Token::MapEnd,
        ],
        "unknown variant `Z`, expected one of `A`, `B`, `C`, `D`, `E`",
    );
}

#[test]
fn test_internally_tagged_enum_with_default_variant() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Newtype(BTreeMap<String, String>);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Struct {
        f: u8,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum InternallyTaggedWithDefaultStructVariant {
        #[serde(default)]
        A {
            a: u8,
        },
        E(Struct),
        B,
        C(BTreeMap<String, String>),
        D(Newtype),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum InternallyTaggedWithDefaultTupleVariant {
        A {
            a: u8,
        },
        #[serde(default)]
        E(Struct),
        B,
        C(BTreeMap<String, String>),
        D(Newtype),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum InternallyTaggedWithDefaultUnitVariant {
        A {
            a: u8,
        },
        E(Struct),
        #[serde(default)]
        B,
        C(BTreeMap<String, String>),
        D(Newtype),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum InternallyTaggedWithDefaultMapVariant {
        A {
            a: u8,
        },
        E(Struct),
        B,
        #[serde(default)]
        C(BTreeMap<String, String>),
        D(Newtype),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum InternallyTaggedWithDefaultNewtypeVariant {
        A {
            a: u8,
        },
        E(Struct),
        B,
        C(BTreeMap<String, String>),
        #[serde(default)]
        D(Newtype),
    }

    assert_de_tokens(
        &InternallyTaggedWithDefaultStructVariant::A { a: 1 },
        &[
            Token::Map { len: Some(1) },
            Token::Str("a"),
            Token::U8(1),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTaggedWithDefaultTupleVariant::E(Struct { f: 1 }),
        &[
            Token::Map { len: Some(1) },
            Token::Str("f"),
            Token::U8(1),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTaggedWithDefaultUnitVariant::B,
        &[Token::Map { len: Some(0) }, Token::MapEnd],
    );

    assert_de_tokens(
        &InternallyTaggedWithDefaultMapVariant::C(BTreeMap::new()),
        &[Token::Map { len: Some(0) }, Token::MapEnd],
    );

    assert_de_tokens(
        &InternallyTaggedWithDefaultNewtypeVariant::D(Newtype(BTreeMap::new())),
        &[Token::Map { len: Some(0) }, Token::MapEnd],
    );
}

#[test]
fn test_internally_tagged_bytes() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(tag = "type")]
    enum InternallyTagged {
        String {
            string: String,
        },
        Bytes {
            #[serde(with = "bytes")]
            bytes: Vec<u8>,
        },
    }

    assert_de_tokens(
        &InternallyTagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "String",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("String"),
            Token::Str("string"),
            Token::Str("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "String",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("String"),
            Token::Str("string"),
            Token::String("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "String",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("String"),
            Token::Str("string"),
            Token::Bytes(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "String",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("String"),
            Token::Str("string"),
            Token::ByteBuf(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Bytes",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("Bytes"),
            Token::Str("bytes"),
            Token::Str("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Bytes",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("Bytes"),
            Token::Str("bytes"),
            Token::String("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Bytes",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("Bytes"),
            Token::Str("bytes"),
            Token::Bytes(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Bytes",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("Bytes"),
            Token::Str("bytes"),
            Token::ByteBuf(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Bytes",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("Bytes"),
            Token::Str("bytes"),
            Token::Seq { len: Some(1) },
            Token::U8(0),
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_internally_tagged_struct_variant_containing_unit_variant() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub enum Level {
        Info,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "action")]
    pub enum Message {
        Log { level: Level },
    }

    assert_de_tokens(
        &Message::Log { level: Level::Info },
        &[
            Token::Struct {
                name: "Message",
                len: 2,
            },
            Token::Str("action"),
            Token::Str("Log"),
            Token::Str("level"),
            Token::BorrowedStr("Info"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Message::Log { level: Level::Info },
        &[
            Token::Map { len: Some(2) },
            Token::Str("action"),
            Token::Str("Log"),
            Token::Str("level"),
            Token::BorrowedStr("Info"),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &Message::Log { level: Level::Info },
        &[
            Token::Seq { len: Some(2) },
            Token::Str("Log"),
            Token::BorrowedStr("Info"),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_internally_tagged_borrow() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum Input<'a> {
        Package { name: &'a str },
    }

    assert_tokens(
        &Input::Package { name: "borrowed" },
        &[
            Token::Struct {
                name: "Input",
                len: 2,
            },
            Token::BorrowedStr("type"),
            Token::BorrowedStr("Package"),
            Token::BorrowedStr("name"),
            Token::BorrowedStr("borrowed"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_adjacently_tagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "t", content = "c")]
    enum AdjacentlyTagged<T> {
        Unit,
        Newtype(T),
        Tuple(u8, u8),
        Struct { f: u8 },
    }

    // unit with no content
    assert_ser_tokens(
        &AdjacentlyTagged::Unit::<u8>,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 1,
            },
            Token::Str("t"),
            Token::Str("Unit"),
            Token::StructEnd,
        ],
    );

    // unit with no content
    assert_de_tokens(
        &AdjacentlyTagged::Unit::<u8>,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::Str("Unit"),
            Token::StructEnd,
        ],
    );

    // unit with tag first
    assert_de_tokens(
        &AdjacentlyTagged::Unit::<u8>,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::Str("Unit"),
            Token::Str("c"),
            Token::Unit,
            Token::StructEnd,
        ],
    );

    // unit with content first
    assert_de_tokens(
        &AdjacentlyTagged::Unit::<u8>,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::Unit,
            Token::Str("t"),
            Token::Str("Unit"),
            Token::StructEnd,
        ],
    );

    // unit with excess content (f, g, h)
    assert_de_tokens(
        &AdjacentlyTagged::Unit::<u8>,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("f"),
            Token::Unit,
            Token::Str("t"),
            Token::Str("Unit"),
            Token::Str("g"),
            Token::Unit,
            Token::Str("c"),
            Token::Unit,
            Token::Str("h"),
            Token::Unit,
            Token::StructEnd,
        ],
    );

    // newtype with tag first
    assert_tokens(
        &AdjacentlyTagged::Newtype::<u8>(1),
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::Str("Newtype"),
            Token::Str("c"),
            Token::U8(1),
            Token::StructEnd,
        ],
    );

    // newtype with content first
    assert_de_tokens(
        &AdjacentlyTagged::Newtype::<u8>(1),
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::U8(1),
            Token::Str("t"),
            Token::Str("Newtype"),
            Token::StructEnd,
        ],
    );

    // optional newtype with no content field
    assert_de_tokens(
        &AdjacentlyTagged::Newtype::<Option<u8>>(None),
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 1,
            },
            Token::Str("t"),
            Token::Str("Newtype"),
            Token::StructEnd,
        ],
    );

    // tuple with tag first
    assert_tokens(
        &AdjacentlyTagged::Tuple::<u8>(1, 1),
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::Str("Tuple"),
            Token::Str("c"),
            Token::Tuple { len: 2 },
            Token::U8(1),
            Token::U8(1),
            Token::TupleEnd,
            Token::StructEnd,
        ],
    );

    // tuple with content first
    assert_de_tokens(
        &AdjacentlyTagged::Tuple::<u8>(1, 1),
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::Tuple { len: 2 },
            Token::U8(1),
            Token::U8(1),
            Token::TupleEnd,
            Token::Str("t"),
            Token::Str("Tuple"),
            Token::StructEnd,
        ],
    );

    // struct with tag first
    assert_tokens(
        &AdjacentlyTagged::Struct::<u8> { f: 1 },
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::Str("Struct"),
            Token::Str("c"),
            Token::Struct {
                name: "Struct",
                len: 1,
            },
            Token::Str("f"),
            Token::U8(1),
            Token::StructEnd,
            Token::StructEnd,
        ],
    );

    // struct with content first
    assert_de_tokens(
        &AdjacentlyTagged::Struct::<u8> { f: 1 },
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::Struct {
                name: "Struct",
                len: 1,
            },
            Token::Str("f"),
            Token::U8(1),
            Token::StructEnd,
            Token::Str("t"),
            Token::Str("Struct"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_adjacently_tagged_enum_deny_unknown_fields() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(tag = "t", content = "c", deny_unknown_fields)]
    enum AdjacentlyTagged {
        Unit,
    }

    assert_de_tokens(
        &AdjacentlyTagged::Unit,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::Str("Unit"),
            Token::Str("c"),
            Token::Unit,
            Token::StructEnd,
        ],
    );

    assert_de_tokens_error::<AdjacentlyTagged>(
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::Str("Unit"),
            Token::Str("c"),
            Token::Unit,
            Token::Str("h"),
        ],
        r#"invalid value: string "h", expected "t" or "c""#,
    );

    assert_de_tokens_error::<AdjacentlyTagged>(
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("h"),
        ],
        r#"invalid value: string "h", expected "t" or "c""#,
    );

    assert_de_tokens_error::<AdjacentlyTagged>(
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::Unit,
            Token::Str("h"),
        ],
        r#"invalid value: string "h", expected "t" or "c""#,
    );
}

#[test]
fn test_enum_in_internally_tagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum Outer {
        Inner(Inner),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Inner {
        Unit,
        Newtype(u8),
        Tuple(u8, u8),
        Struct { f: u8 },
    }

    assert_tokens(
        &Outer::Inner(Inner::Unit),
        &[
            Token::Map { len: Some(2) },
            Token::Str("type"),
            Token::Str("Inner"),
            Token::Str("Unit"),
            Token::Unit,
            Token::MapEnd,
        ],
    );

    assert_tokens(
        &Outer::Inner(Inner::Newtype(1)),
        &[
            Token::Map { len: Some(2) },
            Token::Str("type"),
            Token::Str("Inner"),
            Token::Str("Newtype"),
            Token::U8(1),
            Token::MapEnd,
        ],
    );

    assert_tokens(
        &Outer::Inner(Inner::Tuple(1, 1)),
        &[
            Token::Map { len: Some(2) },
            Token::Str("type"),
            Token::Str("Inner"),
            Token::Str("Tuple"),
            Token::TupleStruct {
                name: "Tuple",
                len: 2,
            },
            Token::U8(1),
            Token::U8(1),
            Token::TupleStructEnd,
            Token::MapEnd,
        ],
    );

    assert_tokens(
        &Outer::Inner(Inner::Struct { f: 1 }),
        &[
            Token::Map { len: Some(2) },
            Token::Str("type"),
            Token::Str("Inner"),
            Token::Str("Struct"),
            Token::Struct {
                name: "Struct",
                len: 1,
            },
            Token::Str("f"),
            Token::U8(1),
            Token::StructEnd,
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_internally_tagged_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub struct Struct {
        a: u8,
    }

    assert_tokens(
        &Struct { a: 1 },
        &[
            Token::Struct {
                name: "Struct",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("Struct"),
            Token::Str("a"),
            Token::U8(1),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Struct { a: 1 },
        &[
            Token::Struct {
                name: "Struct",
                len: 1,
            },
            Token::Str("a"),
            Token::U8(1),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_internally_tagged_braced_struct_with_zero_fields() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    struct S {}

    assert_tokens(
        &S {},
        &[
            Token::Struct { name: "S", len: 1 },
            Token::Str("type"),
            Token::Str("S"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_internally_tagged_struct_with_flattened_field() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "tag_struct")]
    pub struct Struct {
        #[serde(flatten)]
        pub flat: Enum,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "tag_enum", content = "content")]
    pub enum Enum {
        A(u64),
    }

    assert_tokens(
        &Struct { flat: Enum::A(0) },
        &[
            Token::Map { len: None },
            Token::Str("tag_struct"),
            Token::Str("Struct"),
            Token::Str("tag_enum"),
            Token::Str("A"),
            Token::Str("content"),
            Token::U64(0),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &Struct { flat: Enum::A(0) },
        &[
            Token::Map { len: None },
            Token::Str("tag_enum"),
            Token::Str("A"),
            Token::Str("content"),
            Token::U64(0),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_untagged_enum_with_flattened_integer_key() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Untagged {
        Variant {
            #[serde(flatten)]
            map: BTreeMap<u64, String>,
        },
    }

    assert_tokens(
        &Untagged::Variant {
            map: {
                let mut map = BTreeMap::new();
                map.insert(100, "BTreeMap".to_owned());
                map
            },
        },
        &[
            Token::Map { len: None },
            Token::U64(100),
            Token::Str("BTreeMap"),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_enum_in_untagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    enum Outer {
        Inner(Inner),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Inner {
        Unit,
        Newtype(u8),
        Tuple(u8, u8),
        Struct { f: u8 },
    }

    assert_tokens(
        &Outer::Inner(Inner::Unit),
        &[Token::UnitVariant {
            name: "Inner",
            variant: "Unit",
        }],
    );

    assert_tokens(
        &Outer::Inner(Inner::Newtype(1)),
        &[
            Token::NewtypeVariant {
                name: "Inner",
                variant: "Newtype",
            },
            Token::U8(1),
        ],
    );

    assert_tokens(
        &Outer::Inner(Inner::Tuple(1, 1)),
        &[
            Token::TupleVariant {
                name: "Inner",
                variant: "Tuple",
                len: 2,
            },
            Token::U8(1),
            Token::U8(1),
            Token::TupleVariantEnd,
        ],
    );

    assert_tokens(
        &Outer::Inner(Inner::Struct { f: 1 }),
        &[
            Token::StructVariant {
                name: "Inner",
                variant: "Struct",
                len: 1,
            },
            Token::Str("f"),
            Token::U8(1),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_untagged_bytes() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(untagged)]
    enum Untagged {
        String {
            string: String,
        },
        Bytes {
            #[serde(with = "bytes")]
            bytes: Vec<u8>,
        },
    }

    assert_de_tokens(
        &Untagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("string"),
            Token::Str("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("string"),
            Token::String("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("string"),
            Token::Bytes(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("string"),
            Token::ByteBuf(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("bytes"),
            Token::Str("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("bytes"),
            Token::String("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("bytes"),
            Token::Bytes(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("bytes"),
            Token::ByteBuf(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("bytes"),
            Token::Seq { len: Some(1) },
            Token::U8(0),
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_rename_all() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum E {
        #[serde(rename_all = "camelCase")]
        Serialize {
            serialize: bool,
            serialize_seq: bool,
        },
        #[serde(rename_all = "kebab-case")]
        SerializeSeq {
            serialize: bool,
            serialize_seq: bool,
        },
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        SerializeMap {
            serialize: bool,
            serialize_seq: bool,
        },
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "PascalCase")]
    struct S {
        serialize: bool,
        serialize_seq: bool,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "SCREAMING-KEBAB-CASE")]
    struct ScreamingKebab {
        serialize: bool,
        serialize_seq: bool,
    }

    assert_tokens(
        &E::Serialize {
            serialize: true,
            serialize_seq: true,
        },
        &[
            Token::StructVariant {
                name: "E",
                variant: "serialize",
                len: 2,
            },
            Token::Str("serialize"),
            Token::Bool(true),
            Token::Str("serializeSeq"),
            Token::Bool(true),
            Token::StructVariantEnd,
        ],
    );

    assert_tokens(
        &E::SerializeSeq {
            serialize: true,
            serialize_seq: true,
        },
        &[
            Token::StructVariant {
                name: "E",
                variant: "serialize_seq",
                len: 2,
            },
            Token::Str("serialize"),
            Token::Bool(true),
            Token::Str("serialize-seq"),
            Token::Bool(true),
            Token::StructVariantEnd,
        ],
    );

    assert_tokens(
        &E::SerializeMap {
            serialize: true,
            serialize_seq: true,
        },
        &[
            Token::StructVariant {
                name: "E",
                variant: "serialize_map",
                len: 2,
            },
            Token::Str("SERIALIZE"),
            Token::Bool(true),
            Token::Str("SERIALIZE_SEQ"),
            Token::Bool(true),
            Token::StructVariantEnd,
        ],
    );

    assert_tokens(
        &S {
            serialize: true,
            serialize_seq: true,
        },
        &[
            Token::Struct { name: "S", len: 2 },
            Token::Str("Serialize"),
            Token::Bool(true),
            Token::Str("SerializeSeq"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    );

    assert_tokens(
        &ScreamingKebab {
            serialize: true,
            serialize_seq: true,
        },
        &[
            Token::Struct {
                name: "ScreamingKebab",
                len: 2,
            },
            Token::Str("SERIALIZE"),
            Token::Bool(true),
            Token::Str("SERIALIZE-SEQ"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_untagged_newtype_variant_containing_unit_struct_not_map() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Unit;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    enum Message {
        Unit(Unit),
        Map(BTreeMap<String, String>),
    }

    assert_tokens(
        &Message::Map(BTreeMap::new()),
        &[Token::Map { len: Some(0) }, Token::MapEnd],
    );
}

#[test]
fn test_internally_tagged_newtype_variant_containing_unit_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Info;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "topic")]
    enum Message {
        Info(Info),
    }

    assert_tokens(
        &Message::Info(Info),
        &[
            Token::Map { len: Some(1) },
            Token::Str("topic"),
            Token::Str("Info"),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &Message::Info(Info),
        &[
            Token::Struct {
                name: "Message",
                len: 1,
            },
            Token::Str("topic"),
            Token::Str("Info"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Message::Info(Info),
        &[
            Token::Seq { len: Some(1) },
            Token::Str("Info"),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_packed_struct_can_derive_serialize() {
    #[derive(Copy, Clone, Serialize)]
    #[repr(packed, C)]
    struct PackedC {
        t: f32,
    }

    #[derive(Copy, Clone, Serialize)]
    #[repr(C, packed)]
    struct CPacked {
        t: f32,
    }

    #[derive(Copy, Clone, Serialize)]
    #[repr(C, packed(2))]
    struct CPacked2 {
        t: f32,
    }

    #[derive(Copy, Clone, Serialize)]
    #[repr(packed(2), C)]
    struct Packed2C {
        t: f32,
    }
}
