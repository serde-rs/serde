#![deny(trivial_numeric_casts)]
#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::enum_variant_names,
    clippy::redundant_field_names,
    clippy::too_many_lines
)]

use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_de_tokens, assert_ser_tokens, assert_tokens, Token};
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
            Token::UnitVariant {
                name: "Enum",
                variant: "A",
            },
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
fn test_rename_all_fields() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all_fields = "kebab-case")]
    enum E {
        V1,
        V2(bool),
        V3 {
            a_field: bool,
            another_field: bool,
            #[serde(rename = "last-field")]
            yet_another_field: bool,
        },
        #[serde(rename_all = "snake_case")]
        V4 {
            a_field: bool,
        },
    }

    assert_tokens(
        &E::V3 {
            a_field: true,
            another_field: true,
            yet_another_field: true,
        },
        &[
            Token::StructVariant {
                name: "E",
                variant: "V3",
                len: 3,
            },
            Token::Str("a-field"),
            Token::Bool(true),
            Token::Str("another-field"),
            Token::Bool(true),
            Token::Str("last-field"),
            Token::Bool(true),
            Token::StructVariantEnd,
        ],
    );

    assert_tokens(
        &E::V4 { a_field: true },
        &[
            Token::StructVariant {
                name: "E",
                variant: "V4",
                len: 1,
            },
            Token::Str("a_field"),
            Token::Bool(true),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_packed_struct_can_derive_serialize() {
    #[derive(Copy, Clone, Serialize)]
    #[repr(packed, C)]
    #[allow(dead_code)]
    struct PackedC {
        t: f32,
    }

    #[derive(Copy, Clone, Serialize)]
    #[repr(C, packed)]
    #[allow(dead_code)]
    struct CPacked {
        t: f32,
    }

    #[derive(Copy, Clone, Serialize)]
    #[repr(C, packed(2))]
    #[allow(dead_code)]
    struct CPacked2 {
        t: f32,
    }

    #[derive(Copy, Clone, Serialize)]
    #[repr(packed(2), C)]
    #[allow(dead_code)]
    struct Packed2C {
        t: f32,
    }
}

#[test]
fn test_alias_all_kebab_case() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(alias_all = "kebab-case")]
    struct TestStruct {
        field_one: i32,
        field_two: String,
        very_long_field_name: bool,
    }

    // Test deserializing with original snake_case names
    assert_de_tokens(
        &TestStruct {
            field_one: 42,
            field_two: "hello".to_string(),
            very_long_field_name: true,
        },
        &[
            Token::Struct {
                name: "TestStruct",
                len: 3,
            },
            Token::Str("field_one"),
            Token::I32(42),
            Token::Str("field_two"),
            Token::Str("hello"),
            Token::Str("very_long_field_name"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    );

    // Test deserializing with kebab-case aliases
    assert_de_tokens(
        &TestStruct {
            field_one: 42,
            field_two: "hello".to_string(),
            very_long_field_name: true,
        },
        &[
            Token::Struct {
                name: "TestStruct",
                len: 3,
            },
            Token::Str("field-one"),
            Token::I32(42),
            Token::Str("field-two"),
            Token::Str("hello"),
            Token::Str("very-long-field-name"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_alias_all_uppercase() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(alias_all = "UPPERCASE")]
    struct TestStructUppercase {
        field_one: i32,
        field_two: String,
    }

    // Test deserializing with original snake_case names
    assert_de_tokens(
        &TestStructUppercase {
            field_one: 42,
            field_two: "hello".to_string(),
        },
        &[
            Token::Struct {
                name: "TestStructUppercase",
                len: 2,
            },
            Token::Str("field_one"),
            Token::I32(42),
            Token::Str("field_two"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );

    // Test deserializing with UPPERCASE aliases
    assert_de_tokens(
        &TestStructUppercase {
            field_one: 42,
            field_two: "hello".to_string(),
        },
        &[
            Token::Struct {
                name: "TestStructUppercase",
                len: 2,
            },
            Token::Str("FIELD_ONE"),
            Token::I32(42),
            Token::Str("FIELD_TWO"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_alias_all_camel_case() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(alias_all = "camelCase")]
    struct TestStructCamelCase {
        field_one: i32,
        field_two: String,
    }

    // Test deserializing with original snake_case names
    assert_de_tokens(
        &TestStructCamelCase {
            field_one: 42,
            field_two: "hello".to_string(),
        },
        &[
            Token::Struct {
                name: "TestStructCamelCase",
                len: 2,
            },
            Token::Str("field_one"),
            Token::I32(42),
            Token::Str("field_two"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );

    // Test deserializing with camelCase aliases
    assert_de_tokens(
        &TestStructCamelCase {
            field_one: 42,
            field_two: "hello".to_string(),
        },
        &[
            Token::Struct {
                name: "TestStructCamelCase",
                len: 2,
            },
            Token::Str("fieldOne"),
            Token::I32(42),
            Token::Str("fieldTwo"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_alias_all_enum() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(alias_all = "kebab-case")]
    enum TestEnum {
        VariantOne { field_one: i32 },
        VariantTwo { field_two: String },
    }

    // Test deserializing with original PascalCase names
    assert_de_tokens(
        &TestEnum::VariantOne { field_one: 42 },
        &[
            Token::Enum { name: "TestEnum" },
            Token::Str("VariantOne"),
            Token::Struct {
                name: "VariantOne",
                len: 1,
            },
            Token::Str("field_one"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );

    // Test deserializing with kebab-case aliases
    assert_de_tokens(
        &TestEnum::VariantOne { field_one: 42 },
        &[
            Token::Enum { name: "TestEnum" },
            Token::Str("variant-one"),
            Token::Struct {
                name: "VariantOne",
                len: 1,
            },
            Token::Str("field-one"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );

    // Test deserializing with original PascalCase names
    assert_de_tokens(
        &TestEnum::VariantTwo {
            field_two: "hello".to_string(),
        },
        &[
            Token::Enum { name: "TestEnum" },
            Token::Str("VariantTwo"),
            Token::Struct {
                name: "VariantTwo",
                len: 1,
            },
            Token::Str("field_two"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );

    // Test deserializing with kebab-case aliases
    assert_de_tokens(
        &TestEnum::VariantTwo {
            field_two: "hello".to_string(),
        },
        &[
            Token::Enum { name: "TestEnum" },
            Token::Str("variant-two"),
            Token::Struct {
                name: "VariantTwo",
                len: 1,
            },
            Token::Str("field-two"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_multiple_alias_all() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(alias_all = "kebab-case")]
    #[serde(alias_all = "UPPERCASE")]
    struct TestStruct {
        field_one: i32,
        field_two: String,
    }

    // Test deserializing with original snake_case names
    assert_de_tokens(
        &TestStruct {
            field_one: 42,
            field_two: "hello".to_string(),
        },
        &[
            Token::Struct {
                name: "TestStruct",
                len: 2,
            },
            Token::Str("field_one"),
            Token::I32(42),
            Token::Str("field_two"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );

    // Test deserializing with kebab-case aliases
    assert_de_tokens(
        &TestStruct {
            field_one: 42,
            field_two: "hello".to_string(),
        },
        &[
            Token::Struct {
                name: "TestStruct",
                len: 2,
            },
            Token::Str("field-one"),
            Token::I32(42),
            Token::Str("field-two"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );

    // Test deserializing with UPPERCASE aliases
    assert_de_tokens(
        &TestStruct {
            field_one: 42,
            field_two: "hello".to_string(),
        },
        &[
            Token::Struct {
                name: "TestStruct",
                len: 2,
            },
            Token::Str("FIELD_ONE"),
            Token::I32(42),
            Token::Str("FIELD_TWO"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );
}
