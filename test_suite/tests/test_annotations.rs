#![allow(
    clippy::cast_lossless,
    clippy::derive_partial_eq_without_eq,
    clippy::from_over_into,
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
    clippy::nonstandard_macro_braces,
    clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::type_repetition_in_bounds,
    clippy::uninlined_format_args,
)]

use serde::de::{self, Deserialize, Deserializer, IgnoredAny, MapAccess, Unexpected, Visitor};
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use serde_test::{
    assert_de_tokens, assert_de_tokens_error, assert_ser_tokens, assert_ser_tokens_error,
    assert_tokens, Token,
};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::fmt;
use std::marker::PhantomData;

trait MyDefault: Sized {
    fn my_default() -> Self;
}

trait ShouldSkip: Sized {
    fn should_skip(&self) -> bool;
}

trait SerializeWith: Sized {
    fn serialize_with<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

trait DeserializeWith: Sized {
    fn deserialize_with<'de, D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl MyDefault for i32 {
    fn my_default() -> Self {
        123
    }
}

impl ShouldSkip for i32 {
    fn should_skip(&self) -> bool {
        *self == 123
    }
}

impl SerializeWith for i32 {
    fn serialize_with<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if *self == 123 {
            true.serialize(ser)
        } else {
            false.serialize(ser)
        }
    }
}

impl DeserializeWith for i32 {
    fn deserialize_with<'de, D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if Deserialize::deserialize(de)? {
            Ok(123)
        } else {
            Ok(2)
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DefaultStruct<A, B, C, D, E>
where
    C: MyDefault,
    E: MyDefault,
{
    a1: A,
    #[serde(default)]
    a2: B,
    #[serde(default = "MyDefault::my_default")]
    a3: C,
    #[serde(skip_deserializing)]
    a4: D,
    #[serde(skip_deserializing, default = "MyDefault::my_default")]
    a5: E,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DefaultTupleStruct<A, B, C>(
    A,
    #[serde(default)] B,
    #[serde(default = "MyDefault::my_default")] C,
)
where
    C: MyDefault;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct CollectOther {
    a: u32,
    b: u32,
    #[serde(flatten)]
    extra: HashMap<String, u32>,
}

#[test]
fn test_default_struct() {
    assert_de_tokens(
        &DefaultStruct {
            a1: 1,
            a2: 2,
            a3: 3,
            a4: 0,
            a5: 123,
        },
        &[
            Token::Struct {
                name: "DefaultStruct",
                len: 3,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::Str("a2"),
            Token::I32(2),
            Token::Str("a3"),
            Token::I32(3),
            Token::Str("a4"),
            Token::I32(4),
            Token::Str("a5"),
            Token::I32(5),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &DefaultStruct {
            a1: 1,
            a2: 0,
            a3: 123,
            a4: 0,
            a5: 123,
        },
        &[
            Token::Struct {
                name: "DefaultStruct",
                len: 3,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_default_tuple() {
    assert_de_tokens(
        &DefaultTupleStruct(1, 2, 3),
        &[
            Token::TupleStruct {
                name: "DefaultTupleStruct",
                len: 3,
            },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::TupleStructEnd,
        ],
    );

    assert_de_tokens(
        &DefaultTupleStruct(1, 0, 123),
        &[
            Token::TupleStruct {
                name: "DefaultTupleStruct",
                len: 3,
            },
            Token::I32(1),
            Token::TupleStructEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum DefaultStructVariant<A, B, C, D, E>
where
    C: MyDefault,
    E: MyDefault,
{
    Struct {
        a1: A,
        #[serde(default)]
        a2: B,
        #[serde(default = "MyDefault::my_default")]
        a3: C,
        #[serde(skip_deserializing)]
        a4: D,
        #[serde(skip_deserializing, default = "MyDefault::my_default")]
        a5: E,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum DefaultTupleVariant<A, B, C>
where
    C: MyDefault,
{
    Tuple(
        A,
        #[serde(default)] B,
        #[serde(default = "MyDefault::my_default")] C,
    ),
}

#[test]
fn test_default_struct_variant() {
    assert_de_tokens(
        &DefaultStructVariant::Struct {
            a1: 1,
            a2: 2,
            a3: 3,
            a4: 0,
            a5: 123,
        },
        &[
            Token::StructVariant {
                name: "DefaultStructVariant",
                variant: "Struct",
                len: 3,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::Str("a2"),
            Token::I32(2),
            Token::Str("a3"),
            Token::I32(3),
            Token::Str("a4"),
            Token::I32(4),
            Token::Str("a5"),
            Token::I32(5),
            Token::StructVariantEnd,
        ],
    );

    assert_de_tokens(
        &DefaultStructVariant::Struct {
            a1: 1,
            a2: 0,
            a3: 123,
            a4: 0,
            a5: 123,
        },
        &[
            Token::StructVariant {
                name: "DefaultStructVariant",
                variant: "Struct",
                len: 3,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_default_tuple_variant() {
    assert_de_tokens(
        &DefaultTupleVariant::Tuple(1, 2, 3),
        &[
            Token::TupleVariant {
                name: "DefaultTupleVariant",
                variant: "Tuple",
                len: 3,
            },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::TupleVariantEnd,
        ],
    );

    assert_de_tokens(
        &DefaultTupleVariant::Tuple(1, 0, 123),
        &[
            Token::TupleVariant {
                name: "DefaultTupleVariant",
                variant: "Tuple",
                len: 3,
            },
            Token::I32(1),
            Token::TupleVariantEnd,
        ],
    );
}

// Does not implement std::default::Default.
#[derive(Debug, PartialEq, Deserialize)]
struct NoStdDefault(i8);

impl MyDefault for NoStdDefault {
    fn my_default() -> Self {
        NoStdDefault(123)
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct ContainsNoStdDefault<A: MyDefault> {
    #[serde(default = "MyDefault::my_default")]
    a: A,
}

// Tests that a struct field does not need to implement std::default::Default if
// it is annotated with `default=...`.
#[test]
fn test_no_std_default() {
    assert_de_tokens(
        &ContainsNoStdDefault {
            a: NoStdDefault(123),
        },
        &[
            Token::Struct {
                name: "ContainsNoStdDefault",
                len: 1,
            },
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &ContainsNoStdDefault { a: NoStdDefault(8) },
        &[
            Token::Struct {
                name: "ContainsNoStdDefault",
                len: 1,
            },
            Token::Str("a"),
            Token::NewtypeStruct {
                name: "NoStdDefault",
            },
            Token::I8(8),
            Token::StructEnd,
        ],
    );
}

// Does not implement Deserialize.
#[derive(Debug, PartialEq)]
struct NotDeserializeStruct(i8);

impl Default for NotDeserializeStruct {
    fn default() -> Self {
        NotDeserializeStruct(123)
    }
}

impl DeserializeWith for NotDeserializeStruct {
    fn deserialize_with<'de, D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        panic!()
    }
}

// Does not implement Deserialize.
#[derive(Debug, PartialEq)]
enum NotDeserializeEnum {
    Trouble,
}

impl MyDefault for NotDeserializeEnum {
    fn my_default() -> Self {
        NotDeserializeEnum::Trouble
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct ContainsNotDeserialize<A, B, C: DeserializeWith, E: MyDefault> {
    #[serde(skip_deserializing)]
    a: A,
    #[serde(skip_deserializing, default)]
    b: B,
    #[serde(deserialize_with = "DeserializeWith::deserialize_with", default)]
    c: C,
    #[serde(skip_deserializing, default = "MyDefault::my_default")]
    e: E,
}

// Tests that a struct field does not need to implement Deserialize if it is
// annotated with skip_deserializing, whether using the std Default or a
// custom default.
#[test]
fn test_elt_not_deserialize() {
    assert_de_tokens(
        &ContainsNotDeserialize {
            a: NotDeserializeStruct(123),
            b: NotDeserializeStruct(123),
            c: NotDeserializeStruct(123),
            e: NotDeserializeEnum::Trouble,
        },
        &[
            Token::Struct {
                name: "ContainsNotDeserialize",
                len: 1,
            },
            Token::StructEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DenyUnknown {
    a1: i32,
}

#[test]
fn test_ignore_unknown() {
    // 'Default' allows unknown. Basic smoke test of ignore...
    assert_de_tokens(
        &DefaultStruct {
            a1: 1,
            a2: 2,
            a3: 3,
            a4: 0,
            a5: 123,
        },
        &[
            Token::Struct {
                name: "DefaultStruct",
                len: 3,
            },
            Token::Str("whoops1"),
            Token::I32(2),
            Token::Str("a1"),
            Token::I32(1),
            Token::Str("whoops2"),
            Token::Seq { len: Some(1) },
            Token::I32(2),
            Token::SeqEnd,
            Token::Str("a2"),
            Token::I32(2),
            Token::Str("whoops3"),
            Token::I32(2),
            Token::Str("a3"),
            Token::I32(3),
            Token::StructEnd,
        ],
    );

    assert_de_tokens_error::<DenyUnknown>(
        &[
            Token::Struct {
                name: "DenyUnknown",
                len: 1,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::Str("whoops"),
        ],
        "unknown field `whoops`, expected `a1`",
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Superhero")]
struct RenameStruct {
    a1: i32,
    #[serde(rename = "a3")]
    a2: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename(serialize = "SuperheroSer", deserialize = "SuperheroDe"))]
struct RenameStructSerializeDeserialize {
    a1: i32,
    #[serde(rename(serialize = "a4", deserialize = "a5"))]
    a2: i32,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct AliasStruct {
    a1: i32,
    #[serde(alias = "a3")]
    a2: i32,
    #[serde(alias = "a5", rename = "a6")]
    a4: i32,
}

#[test]
fn test_rename_struct() {
    assert_tokens(
        &RenameStruct { a1: 1, a2: 2 },
        &[
            Token::Struct {
                name: "Superhero",
                len: 2,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::Str("a3"),
            Token::I32(2),
            Token::StructEnd,
        ],
    );

    assert_ser_tokens(
        &RenameStructSerializeDeserialize { a1: 1, a2: 2 },
        &[
            Token::Struct {
                name: "SuperheroSer",
                len: 2,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::Str("a4"),
            Token::I32(2),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &RenameStructSerializeDeserialize { a1: 1, a2: 2 },
        &[
            Token::Struct {
                name: "SuperheroDe",
                len: 2,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::Str("a5"),
            Token::I32(2),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &AliasStruct {
            a1: 1,
            a2: 2,
            a4: 3,
        },
        &[
            Token::Struct {
                name: "AliasStruct",
                len: 3,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::Str("a2"),
            Token::I32(2),
            Token::Str("a5"),
            Token::I32(3),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &AliasStruct {
            a1: 1,
            a2: 2,
            a4: 3,
        },
        &[
            Token::Struct {
                name: "AliasStruct",
                len: 3,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::Str("a3"),
            Token::I32(2),
            Token::Str("a6"),
            Token::I32(3),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_unknown_field_rename_struct() {
    assert_de_tokens_error::<AliasStruct>(
        &[
            Token::Struct {
                name: "AliasStruct",
                len: 3,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::Str("a3"),
            Token::I32(2),
            Token::Str("a4"),
            Token::I32(3),
        ],
        "unknown field `a4`, expected one of `a1`, `a2`, `a3`, `a5`, `a6`",
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Superhero")]
enum RenameEnum {
    #[serde(rename = "bruce_wayne")]
    Batman,
    #[serde(rename = "clark_kent")]
    Superman(i8),
    #[serde(rename = "diana_prince")]
    WonderWoman(i8, i8),
    #[serde(rename = "barry_allan")]
    Flash {
        #[serde(rename = "b")]
        a: i32,
    },
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename(serialize = "SuperheroSer", deserialize = "SuperheroDe"))]
enum RenameEnumSerializeDeserialize<A> {
    #[serde(rename(serialize = "dick_grayson", deserialize = "jason_todd"))]
    Robin {
        a: i8,
        #[serde(rename(serialize = "c"))]
        #[serde(rename(deserialize = "d"))]
        b: A,
    },
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
enum AliasEnum {
    #[serde(rename = "sailor_moon", alias = "usagi_tsukino")]
    SailorMoon {
        a: i8,
        #[serde(alias = "c")]
        b: i8,
        #[serde(alias = "e", rename = "f")]
        d: i8,
    },
}

#[test]
fn test_rename_enum() {
    assert_tokens(
        &RenameEnum::Batman,
        &[Token::UnitVariant {
            name: "Superhero",
            variant: "bruce_wayne",
        }],
    );

    assert_tokens(
        &RenameEnum::Superman(0),
        &[
            Token::NewtypeVariant {
                name: "Superhero",
                variant: "clark_kent",
            },
            Token::I8(0),
        ],
    );

    assert_tokens(
        &RenameEnum::WonderWoman(0, 1),
        &[
            Token::TupleVariant {
                name: "Superhero",
                variant: "diana_prince",
                len: 2,
            },
            Token::I8(0),
            Token::I8(1),
            Token::TupleVariantEnd,
        ],
    );

    assert_tokens(
        &RenameEnum::Flash { a: 1 },
        &[
            Token::StructVariant {
                name: "Superhero",
                variant: "barry_allan",
                len: 1,
            },
            Token::Str("b"),
            Token::I32(1),
            Token::StructVariantEnd,
        ],
    );

    assert_ser_tokens(
        &RenameEnumSerializeDeserialize::Robin {
            a: 0,
            b: String::new(),
        },
        &[
            Token::StructVariant {
                name: "SuperheroSer",
                variant: "dick_grayson",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(0),
            Token::Str("c"),
            Token::Str(""),
            Token::StructVariantEnd,
        ],
    );

    assert_de_tokens(
        &RenameEnumSerializeDeserialize::Robin {
            a: 0,
            b: String::new(),
        },
        &[
            Token::StructVariant {
                name: "SuperheroDe",
                variant: "jason_todd",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(0),
            Token::Str("d"),
            Token::Str(""),
            Token::StructVariantEnd,
        ],
    );

    assert_de_tokens(
        &AliasEnum::SailorMoon { a: 0, b: 1, d: 2 },
        &[
            Token::StructVariant {
                name: "AliasEnum",
                variant: "sailor_moon",
                len: 5,
            },
            Token::Str("a"),
            Token::I8(0),
            Token::Str("b"),
            Token::I8(1),
            Token::Str("e"),
            Token::I8(2),
            Token::StructVariantEnd,
        ],
    );

    assert_de_tokens(
        &AliasEnum::SailorMoon { a: 0, b: 1, d: 2 },
        &[
            Token::StructVariant {
                name: "AliasEnum",
                variant: "usagi_tsukino",
                len: 5,
            },
            Token::Str("a"),
            Token::I8(0),
            Token::Str("c"),
            Token::I8(1),
            Token::Str("f"),
            Token::I8(2),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_unknown_field_rename_enum() {
    assert_de_tokens_error::<AliasEnum>(
        &[Token::StructVariant {
            name: "AliasEnum",
            variant: "SailorMoon",
            len: 3,
        }],
        "unknown variant `SailorMoon`, expected `sailor_moon`",
    );

    assert_de_tokens_error::<AliasEnum>(
        &[
            Token::StructVariant {
                name: "AliasEnum",
                variant: "usagi_tsukino",
                len: 5,
            },
            Token::Str("a"),
            Token::I8(0),
            Token::Str("c"),
            Token::I8(1),
            Token::Str("d"),
            Token::I8(2),
        ],
        "unknown field `d`, expected one of `a`, `b`, `c`, `e`, `f`",
    );
}

#[derive(Debug, PartialEq, Serialize)]
struct SkipSerializingStruct<'a, B, C>
where
    C: ShouldSkip,
{
    a: &'a i8,
    #[serde(skip_serializing)]
    b: B,
    #[serde(skip_serializing_if = "ShouldSkip::should_skip")]
    c: C,
}

#[test]
fn test_skip_serializing_struct() {
    let a = 1;
    assert_ser_tokens(
        &SkipSerializingStruct { a: &a, b: 2, c: 3 },
        &[
            Token::Struct {
                name: "SkipSerializingStruct",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("c"),
            Token::I32(3),
            Token::StructEnd,
        ],
    );

    assert_ser_tokens(
        &SkipSerializingStruct {
            a: &a,
            b: 2,
            c: 123,
        },
        &[
            Token::Struct {
                name: "SkipSerializingStruct",
                len: 1,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::StructEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Serialize)]
struct SkipSerializingTupleStruct<'a, B, C>(
    &'a i8,
    #[serde(skip_serializing)] B,
    #[serde(skip_serializing_if = "ShouldSkip::should_skip")] C,
)
where
    C: ShouldSkip;

#[test]
fn test_skip_serializing_tuple_struct() {
    let a = 1;
    assert_ser_tokens(
        &SkipSerializingTupleStruct(&a, 2, 3),
        &[
            Token::TupleStruct {
                name: "SkipSerializingTupleStruct",
                len: 2,
            },
            Token::I8(1),
            Token::I32(3),
            Token::TupleStructEnd,
        ],
    );

    assert_ser_tokens(
        &SkipSerializingTupleStruct(&a, 2, 123),
        &[
            Token::TupleStruct {
                name: "SkipSerializingTupleStruct",
                len: 1,
            },
            Token::I8(1),
            Token::TupleStructEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SkipStruct<B> {
    a: i8,
    #[serde(skip)]
    b: B,
}

#[test]
fn test_skip_struct() {
    assert_ser_tokens(
        &SkipStruct { a: 1, b: 2 },
        &[
            Token::Struct {
                name: "SkipStruct",
                len: 1,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &SkipStruct { a: 1, b: 0 },
        &[
            Token::Struct {
                name: "SkipStruct",
                len: 1,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::StructEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Serialize)]
enum SkipSerializingEnum<'a, B, C>
where
    C: ShouldSkip,
{
    Struct {
        a: &'a i8,
        #[serde(skip_serializing)]
        _b: B,
        #[serde(skip_serializing_if = "ShouldSkip::should_skip")]
        c: C,
    },
    Tuple(
        &'a i8,
        #[serde(skip_serializing)] B,
        #[serde(skip_serializing_if = "ShouldSkip::should_skip")] C,
    ),
}

#[test]
fn test_skip_serializing_enum() {
    let a = 1;
    assert_ser_tokens(
        &SkipSerializingEnum::Struct { a: &a, _b: 2, c: 3 },
        &[
            Token::StructVariant {
                name: "SkipSerializingEnum",
                variant: "Struct",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("c"),
            Token::I32(3),
            Token::StructVariantEnd,
        ],
    );

    assert_ser_tokens(
        &SkipSerializingEnum::Struct {
            a: &a,
            _b: 2,
            c: 123,
        },
        &[
            Token::StructVariant {
                name: "SkipSerializingEnum",
                variant: "Struct",
                len: 1,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::StructVariantEnd,
        ],
    );

    assert_ser_tokens(
        &SkipSerializingEnum::Tuple(&a, 2, 3),
        &[
            Token::TupleVariant {
                name: "SkipSerializingEnum",
                variant: "Tuple",
                len: 2,
            },
            Token::I8(1),
            Token::I32(3),
            Token::TupleVariantEnd,
        ],
    );

    assert_ser_tokens(
        &SkipSerializingEnum::Tuple(&a, 2, 123),
        &[
            Token::TupleVariant {
                name: "SkipSerializingEnum",
                variant: "Tuple",
                len: 1,
            },
            Token::I8(1),
            Token::TupleVariantEnd,
        ],
    );
}

#[derive(Debug, PartialEq)]
struct NotSerializeStruct(i8);

#[derive(Debug, PartialEq)]
enum NotSerializeEnum {
    Trouble,
}

impl SerializeWith for NotSerializeEnum {
    fn serialize_with<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        "trouble".serialize(ser)
    }
}

#[derive(Debug, PartialEq, Serialize)]
struct ContainsNotSerialize<'a, B, C, D>
where
    B: 'a,
    D: SerializeWith,
{
    a: &'a Option<i8>,
    #[serde(skip_serializing)]
    b: &'a B,
    #[serde(skip_serializing)]
    c: Option<C>,
    #[serde(serialize_with = "SerializeWith::serialize_with")]
    d: D,
}

#[test]
fn test_elt_not_serialize() {
    let a = 1;
    assert_ser_tokens(
        &ContainsNotSerialize {
            a: &Some(a),
            b: &NotSerializeStruct(2),
            c: Some(NotSerializeEnum::Trouble),
            d: NotSerializeEnum::Trouble,
        },
        &[
            Token::Struct {
                name: "ContainsNotSerialize",
                len: 2,
            },
            Token::Str("a"),
            Token::Some,
            Token::I8(1),
            Token::Str("d"),
            Token::Str("trouble"),
            Token::StructEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Serialize)]
struct SerializeWithStruct<'a, B>
where
    B: SerializeWith,
{
    a: &'a i8,
    #[serde(serialize_with = "SerializeWith::serialize_with")]
    b: B,
}

#[test]
fn test_serialize_with_struct() {
    let a = 1;
    assert_ser_tokens(
        &SerializeWithStruct { a: &a, b: 2 },
        &[
            Token::Struct {
                name: "SerializeWithStruct",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("b"),
            Token::Bool(false),
            Token::StructEnd,
        ],
    );

    assert_ser_tokens(
        &SerializeWithStruct { a: &a, b: 123 },
        &[
            Token::Struct {
                name: "SerializeWithStruct",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("b"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Serialize)]
enum SerializeWithEnum<'a, B>
where
    B: SerializeWith,
{
    Struct {
        a: &'a i8,
        #[serde(serialize_with = "SerializeWith::serialize_with")]
        b: B,
    },
}

#[test]
fn test_serialize_with_enum() {
    let a = 1;
    assert_ser_tokens(
        &SerializeWithEnum::Struct { a: &a, b: 2 },
        &[
            Token::StructVariant {
                name: "SerializeWithEnum",
                variant: "Struct",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("b"),
            Token::Bool(false),
            Token::StructVariantEnd,
        ],
    );

    assert_ser_tokens(
        &SerializeWithEnum::Struct { a: &a, b: 123 },
        &[
            Token::StructVariant {
                name: "SerializeWithEnum",
                variant: "Struct",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("b"),
            Token::Bool(true),
            Token::StructVariantEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum WithVariant {
    #[serde(serialize_with = "serialize_unit_variant_as_i8")]
    #[serde(deserialize_with = "deserialize_i8_as_unit_variant")]
    Unit,

    #[serde(serialize_with = "SerializeWith::serialize_with")]
    #[serde(deserialize_with = "DeserializeWith::deserialize_with")]
    Newtype(i32),

    #[serde(serialize_with = "serialize_variant_as_string")]
    #[serde(deserialize_with = "deserialize_string_as_variant")]
    Tuple(String, u8),

    #[serde(serialize_with = "serialize_variant_as_string")]
    #[serde(deserialize_with = "deserialize_string_as_variant")]
    Struct { f1: String, f2: u8 },
}

fn serialize_unit_variant_as_i8<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i8(0)
}

fn deserialize_i8_as_unit_variant<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    let n = i8::deserialize(deserializer)?;
    match n {
        0 => Ok(()),
        _ => Err(de::Error::invalid_value(Unexpected::Signed(n as i64), &"0")),
    }
}

fn serialize_variant_as_string<S>(f1: &str, f2: &u8, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_str(&format_args!("{};{:?}", f1, f2))
}

fn deserialize_string_as_variant<'de, D>(deserializer: D) -> Result<(String, u8), D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mut pieces = s.split(';');
    let Some(f1) = pieces.next() else {
        return Err(de::Error::invalid_length(0, &"2"));
    };
    let Some(f2) = pieces.next() else {
        return Err(de::Error::invalid_length(1, &"2"));
    };
    let Ok(f2) = f2.parse() else {
        return Err(de::Error::invalid_value(
            Unexpected::Str(f2),
            &"an 8-bit signed integer",
        ));
    };
    Ok((f1.into(), f2))
}

#[test]
fn test_serialize_with_variant() {
    assert_ser_tokens(
        &WithVariant::Unit,
        &[
            Token::NewtypeVariant {
                name: "WithVariant",
                variant: "Unit",
            },
            Token::I8(0),
        ],
    );

    assert_ser_tokens(
        &WithVariant::Newtype(123),
        &[
            Token::NewtypeVariant {
                name: "WithVariant",
                variant: "Newtype",
            },
            Token::Bool(true),
        ],
    );

    assert_ser_tokens(
        &WithVariant::Tuple("hello".into(), 0),
        &[
            Token::NewtypeVariant {
                name: "WithVariant",
                variant: "Tuple",
            },
            Token::Str("hello;0"),
        ],
    );

    assert_ser_tokens(
        &WithVariant::Struct {
            f1: "world".into(),
            f2: 1,
        },
        &[
            Token::NewtypeVariant {
                name: "WithVariant",
                variant: "Struct",
            },
            Token::Str("world;1"),
        ],
    );
}

#[test]
fn test_deserialize_with_variant() {
    assert_de_tokens(
        &WithVariant::Unit,
        &[
            Token::NewtypeVariant {
                name: "WithVariant",
                variant: "Unit",
            },
            Token::I8(0),
        ],
    );

    assert_de_tokens(
        &WithVariant::Newtype(123),
        &[
            Token::NewtypeVariant {
                name: "WithVariant",
                variant: "Newtype",
            },
            Token::Bool(true),
        ],
    );

    assert_de_tokens(
        &WithVariant::Tuple("hello".into(), 0),
        &[
            Token::NewtypeVariant {
                name: "WithVariant",
                variant: "Tuple",
            },
            Token::Str("hello;0"),
        ],
    );

    assert_de_tokens(
        &WithVariant::Struct {
            f1: "world".into(),
            f2: 1,
        },
        &[
            Token::NewtypeVariant {
                name: "WithVariant",
                variant: "Struct",
            },
            Token::Str("world;1"),
        ],
    );
}

#[derive(Debug, PartialEq, Deserialize)]
struct DeserializeWithStruct<B>
where
    B: DeserializeWith,
{
    a: i8,
    #[serde(deserialize_with = "DeserializeWith::deserialize_with")]
    b: B,
}

#[test]
fn test_deserialize_with_struct() {
    assert_de_tokens(
        &DeserializeWithStruct { a: 1, b: 2 },
        &[
            Token::Struct {
                name: "DeserializeWithStruct",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("b"),
            Token::Bool(false),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &DeserializeWithStruct { a: 1, b: 123 },
        &[
            Token::Struct {
                name: "DeserializeWithStruct",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("b"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Deserialize)]
enum DeserializeWithEnum<B>
where
    B: DeserializeWith,
{
    Struct {
        a: i8,
        #[serde(deserialize_with = "DeserializeWith::deserialize_with")]
        b: B,
    },
}

#[test]
fn test_deserialize_with_enum() {
    assert_de_tokens(
        &DeserializeWithEnum::Struct { a: 1, b: 2 },
        &[
            Token::StructVariant {
                name: "DeserializeWithEnum",
                variant: "Struct",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("b"),
            Token::Bool(false),
            Token::StructVariantEnd,
        ],
    );

    assert_de_tokens(
        &DeserializeWithEnum::Struct { a: 1, b: 123 },
        &[
            Token::StructVariant {
                name: "DeserializeWithEnum",
                variant: "Struct",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(1),
            Token::Str("b"),
            Token::Bool(true),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_missing_renamed_field_struct() {
    assert_de_tokens_error::<RenameStruct>(
        &[
            Token::Struct {
                name: "Superhero",
                len: 2,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::StructEnd,
        ],
        "missing field `a3`",
    );

    assert_de_tokens_error::<RenameStructSerializeDeserialize>(
        &[
            Token::Struct {
                name: "SuperheroDe",
                len: 2,
            },
            Token::Str("a1"),
            Token::I32(1),
            Token::StructEnd,
        ],
        "missing field `a5`",
    );
}

#[test]
fn test_missing_renamed_field_enum() {
    assert_de_tokens_error::<RenameEnum>(
        &[
            Token::StructVariant {
                name: "Superhero",
                variant: "barry_allan",
                len: 1,
            },
            Token::StructVariantEnd,
        ],
        "missing field `b`",
    );

    assert_de_tokens_error::<RenameEnumSerializeDeserialize<i8>>(
        &[
            Token::StructVariant {
                name: "SuperheroDe",
                variant: "jason_todd",
                len: 2,
            },
            Token::Str("a"),
            Token::I8(0),
            Token::StructVariantEnd,
        ],
        "missing field `d`",
    );
}

#[derive(Debug, PartialEq, Deserialize)]
enum InvalidLengthEnum {
    A(i32, i32, i32),
    B(#[serde(skip_deserializing)] i32, i32, i32),
}

#[test]
fn test_invalid_length_enum() {
    assert_de_tokens_error::<InvalidLengthEnum>(
        &[
            Token::TupleVariant {
                name: "InvalidLengthEnum",
                variant: "A",
                len: 3,
            },
            Token::I32(1),
            Token::TupleVariantEnd,
        ],
        "invalid length 1, expected tuple variant InvalidLengthEnum::A with 3 elements",
    );
    assert_de_tokens_error::<InvalidLengthEnum>(
        &[
            Token::TupleVariant {
                name: "InvalidLengthEnum",
                variant: "B",
                len: 2,
            },
            Token::I32(1),
            Token::TupleVariantEnd,
        ],
        "invalid length 1, expected tuple variant InvalidLengthEnum::B with 2 elements",
    );
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(into = "EnumToU32", from = "EnumToU32")]
struct StructFromEnum(Option<u32>);

impl Into<EnumToU32> for StructFromEnum {
    fn into(self) -> EnumToU32 {
        match self {
            StructFromEnum(v) => v.into(),
        }
    }
}

impl From<EnumToU32> for StructFromEnum {
    fn from(v: EnumToU32) -> Self {
        StructFromEnum(v.into())
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(into = "Option<u32>", from = "Option<u32>")]
enum EnumToU32 {
    One,
    Two,
    Three,
    Four,
    Nothing,
}

impl Into<Option<u32>> for EnumToU32 {
    fn into(self) -> Option<u32> {
        match self {
            EnumToU32::One => Some(1),
            EnumToU32::Two => Some(2),
            EnumToU32::Three => Some(3),
            EnumToU32::Four => Some(4),
            EnumToU32::Nothing => None,
        }
    }
}

impl From<Option<u32>> for EnumToU32 {
    fn from(v: Option<u32>) -> Self {
        match v {
            Some(1) => EnumToU32::One,
            Some(2) => EnumToU32::Two,
            Some(3) => EnumToU32::Three,
            Some(4) => EnumToU32::Four,
            _ => EnumToU32::Nothing,
        }
    }
}

#[derive(Clone, Deserialize, PartialEq, Debug)]
#[serde(try_from = "u32")]
enum TryFromU32 {
    One,
    Two,
}

impl TryFrom<u32> for TryFromU32 {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(TryFromU32::One),
            2 => Ok(TryFromU32::Two),
            _ => Err("out of range".to_owned()),
        }
    }
}

#[test]
fn test_from_into_traits() {
    assert_ser_tokens(&EnumToU32::One, &[Token::Some, Token::U32(1)]);
    assert_ser_tokens(&EnumToU32::Nothing, &[Token::None]);
    assert_de_tokens(&EnumToU32::Two, &[Token::Some, Token::U32(2)]);
    assert_ser_tokens(&StructFromEnum(Some(5)), &[Token::None]);
    assert_ser_tokens(&StructFromEnum(None), &[Token::None]);
    assert_de_tokens(&StructFromEnum(Some(2)), &[Token::Some, Token::U32(2)]);
    assert_de_tokens(&TryFromU32::Two, &[Token::U32(2)]);
    assert_de_tokens_error::<TryFromU32>(&[Token::U32(5)], "out of range");
}

#[test]
fn test_collect_other() {
    let mut extra = HashMap::new();
    extra.insert("c".into(), 3);
    assert_tokens(
        &CollectOther { a: 1, b: 2, extra },
        &[
            Token::Map { len: None },
            Token::Str("a"),
            Token::U32(1),
            Token::Str("b"),
            Token::U32(2),
            Token::Str("c"),
            Token::U32(3),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_partially_untagged_enum() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum Exp {
        Lambda(u32, Box<Exp>),
        #[serde(untagged)]
        App(Box<Exp>, Box<Exp>),
        #[serde(untagged)]
        Var(u32),
    }
    use Exp::*;

    let data = Lambda(0, Box::new(App(Box::new(Var(0)), Box::new(Var(0)))));
    assert_tokens(
        &data,
        &[
            Token::TupleVariant {
                name: "Exp",
                variant: "Lambda",
                len: 2,
            },
            Token::U32(0),
            Token::Tuple { len: 2 },
            Token::U32(0),
            Token::U32(0),
            Token::TupleEnd,
            Token::TupleVariantEnd,
        ],
    );
}

#[test]
fn test_partially_untagged_enum_generic() {
    trait Trait<T> {
        type Assoc;
        type Assoc2;
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum E<A, B, C>
    where
        A: Trait<C, Assoc2 = B>,
    {
        A(A::Assoc),
        #[serde(untagged)]
        B(A::Assoc2),
    }

    impl<T> Trait<T> for () {
        type Assoc = T;
        type Assoc2 = bool;
    }

    type MyE = E<(), bool, u32>;
    use E::*;

    assert_tokens::<MyE>(&B(true), &[Token::Bool(true)]);

    assert_tokens::<MyE>(
        &A(5),
        &[
            Token::NewtypeVariant {
                name: "E",
                variant: "A",
            },
            Token::U32(5),
        ],
    );
}

#[test]
fn test_partially_untagged_enum_desugared() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum Test {
        A(u32, u32),
        B(u32),
        #[serde(untagged)]
        C(u32),
        #[serde(untagged)]
        D(u32, u32),
    }
    use Test::*;

    mod desugared {
        use super::*;
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        pub(super) enum Test {
            A(u32, u32),
            B(u32),
        }
    }
    use desugared::Test as TestTagged;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum TestUntagged {
        Tagged(TestTagged),
        C(u32),
        D(u32, u32),
    }

    impl From<Test> for TestUntagged {
        fn from(test: Test) -> Self {
            match test {
                A(x, y) => TestUntagged::Tagged(TestTagged::A(x, y)),
                B(x) => TestUntagged::Tagged(TestTagged::B(x)),
                C(x) => TestUntagged::C(x),
                D(x, y) => TestUntagged::D(x, y),
            }
        }
    }

    fn assert_tokens_desugared(value: Test, tokens: &[Token]) {
        assert_tokens(&value, tokens);
        let desugared: TestUntagged = value.into();
        assert_tokens(&desugared, tokens);
    }

    assert_tokens_desugared(
        A(0, 1),
        &[
            Token::TupleVariant {
                name: "Test",
                variant: "A",
                len: 2,
            },
            Token::U32(0),
            Token::U32(1),
            Token::TupleVariantEnd,
        ],
    );

    assert_tokens_desugared(
        B(1),
        &[
            Token::NewtypeVariant {
                name: "Test",
                variant: "B",
            },
            Token::U32(1),
        ],
    );

    assert_tokens_desugared(C(2), &[Token::U32(2)]);

    assert_tokens_desugared(
        D(3, 5),
        &[
            Token::Tuple { len: 2 },
            Token::U32(3),
            Token::U32(5),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn test_partially_untagged_internally_tagged_enum() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(tag = "t")]
    enum Data {
        A,
        B,
        #[serde(untagged)]
        Var(u32),
    }

    let data = Data::A;

    assert_de_tokens(
        &data,
        &[
            Token::Map { len: None },
            Token::Str("t"),
            Token::Str("A"),
            Token::MapEnd,
        ],
    );

    let data = Data::Var(42);

    assert_de_tokens(&data, &[Token::U32(42)]);

    // TODO test error output
}

#[test]
fn test_transparent_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(transparent)]
    struct Transparent {
        #[serde(skip)]
        a: bool,
        b: u32,
        #[serde(skip)]
        c: bool,
        d: PhantomData<()>,
    }

    assert_tokens(
        &Transparent {
            a: false,
            b: 1,
            c: false,
            d: PhantomData,
        },
        &[Token::U32(1)],
    );
}

#[test]
fn test_transparent_tuple_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(transparent)]
    struct Transparent(
        #[serde(skip)] bool,
        u32,
        #[serde(skip)] bool,
        PhantomData<()>,
    );

    assert_tokens(&Transparent(false, 1, false, PhantomData), &[Token::U32(1)]);
}

#[test]
fn test_expecting_message() {
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(expecting = "something strange...")]
    struct Unit;

    #[derive(Deserialize)]
    #[serde(expecting = "something strange...")]
    struct Newtype(bool);

    #[derive(Deserialize)]
    #[serde(expecting = "something strange...")]
    struct Tuple(u32, bool);

    #[derive(Deserialize)]
    #[serde(expecting = "something strange...")]
    struct Struct {
        #[allow(dead_code)]
        question: String,
        #[allow(dead_code)]
        answer: u32,
    }

    assert_de_tokens_error::<Unit>(
        &[Token::Str("Unit")],
        r#"invalid type: string "Unit", expected something strange..."#,
    );

    assert_de_tokens_error::<Newtype>(
        &[Token::Str("Newtype")],
        r#"invalid type: string "Newtype", expected something strange..."#,
    );

    assert_de_tokens_error::<Tuple>(
        &[Token::Str("Tuple")],
        r#"invalid type: string "Tuple", expected something strange..."#,
    );

    assert_de_tokens_error::<Struct>(
        &[Token::Str("Struct")],
        r#"invalid type: string "Struct", expected something strange..."#,
    );
}

#[test]
fn test_expecting_message_externally_tagged_enum() {
    #[derive(Deserialize)]
    #[serde(expecting = "something strange...")]
    enum Enum {
        ExternallyTagged,
    }

    assert_de_tokens_error::<Enum>(
        &[Token::Str("ExternallyTagged")],
        r#"invalid type: string "ExternallyTagged", expected something strange..."#,
    );

    // Check that #[serde(expecting = "...")] doesn't affect variant identifier error message
    assert_de_tokens_error::<Enum>(
        &[Token::Enum { name: "Enum" }, Token::Unit],
        "invalid type: unit value, expected variant identifier",
    );
}

#[test]
fn test_expecting_message_identifier_enum() {
    #[derive(Deserialize)]
    #[serde(field_identifier)]
    #[serde(expecting = "something strange...")]
    enum FieldEnum {
        Field,
    }

    #[derive(Deserialize)]
    #[serde(variant_identifier)]
    #[serde(expecting = "something strange...")]
    enum VariantEnum {
        Variant,
    }

    assert_de_tokens_error::<FieldEnum>(
        &[Token::Unit],
        "invalid type: unit value, expected something strange...",
    );

    assert_de_tokens_error::<FieldEnum>(
        &[
            Token::Enum { name: "FieldEnum" },
            Token::Str("Unknown"),
            Token::None,
        ],
        "invalid type: map, expected something strange...",
    );

    assert_de_tokens_error::<VariantEnum>(
        &[Token::Unit],
        "invalid type: unit value, expected something strange...",
    );

    assert_de_tokens_error::<VariantEnum>(
        &[
            Token::Enum {
                name: "VariantEnum",
            },
            Token::Str("Unknown"),
            Token::None,
        ],
        "invalid type: map, expected something strange...",
    );
}

#[test]
fn test_non_string_renames() {
    #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
    #[serde(tag = "op")]
    enum SpecialEnum {
        #[serde(rename = -1)]
        A,
        #[serde(rename = true)]
        B,
    }

    assert_de_tokens(
        &SpecialEnum::A,
        &[
            Token::Map { len: None },
            Token::Str("op"),
            Token::I64(-1),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &SpecialEnum::B,
        &[
            Token::Map { len: None },
            Token::Str("op"),
            Token::Bool(true),
            Token::MapEnd,
        ],
    );

    assert_ser_tokens(
        &SpecialEnum::A,
        &[
            Token::Struct {
                name: "SpecialEnum",
                len: 1,
            },
            Token::Str("op"),
            Token::I64(-1),
            Token::StructEnd,
        ],
    );

    assert_ser_tokens(
        &SpecialEnum::B,
        &[
            Token::Struct {
                name: "SpecialEnum",
                len: 1,
            },
            Token::Str("op"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    );

    #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
    #[serde(tag = "op", content = "d")]
    enum AdjacentEnum {
        #[serde(rename = -1i64)]
        A { a: u64 },
        #[serde(rename = true)]
        B,
    }

    assert_de_tokens(
        &AdjacentEnum::A { a: 1 },
        &[
            Token::Map { len: None },
            Token::Str("op"),
            Token::I64(-1),
            Token::Str("d"),
            Token::Map { len: Some(1) },
            Token::Str("a"),
            Token::U64(1),
            Token::MapEnd,
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &AdjacentEnum::B,
        &[
            Token::Map { len: None },
            Token::Str("op"),
            Token::Bool(true),
            Token::MapEnd,
        ],
    );

    assert_ser_tokens(
        &AdjacentEnum::A { a: 1 },
        &[
            Token::Struct {
                name: "AdjacentEnum",
                len: 2,
            },
            Token::Str("op"),
            Token::I64(-1),
            Token::Str("d"),
            Token::Struct {
                name: "-1i64",
                len: 1,
            },
            Token::Str("a"),
            Token::U64(1),
            Token::StructEnd,
            Token::StructEnd,
        ],
    );

    assert_ser_tokens(
        &AdjacentEnum::B,
        &[
            Token::Struct {
                name: "AdjacentEnum",
                len: 1,
            },
            Token::Str("op"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    )
}

#[test]
fn test_non_string_aliases() {
    #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
    #[serde(tag = "op")]
    enum AliasedEnum {
        #[serde(rename = 1, alias = 2, alias = "foo")]
        A,
        #[serde(rename = 3, alias = "bar", alias = false)]
        B,
    }

    assert_de_tokens(
        &AliasedEnum::A,
        &[
            Token::Map { len: None },
            Token::Str("op"),
            Token::I64(1),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &AliasedEnum::A,
        &[
            Token::Map { len: None },
            Token::Str("op"),
            Token::I64(2),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &AliasedEnum::A,
        &[
            Token::Map { len: None },
            Token::Str("op"),
            Token::Str("foo"),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &AliasedEnum::B,
        &[
            Token::Map { len: None },
            Token::Str("op"),
            Token::I64(3),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &AliasedEnum::B,
        &[
            Token::Map { len: None },
            Token::Str("op"),
            Token::Str("bar"),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &AliasedEnum::B,
        &[
            Token::Map { len: None },
            Token::Str("op"),
            Token::Bool(false),
            Token::MapEnd,
        ],
    );
}

mod flatten {
    use super::*;

    #[test]
    fn complex() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Outer {
            y: u32,
            #[serde(flatten)]
            first: First,
            #[serde(flatten)]
            second: Second,
            z: u32,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct First {
            a: u32,
            b: bool,
            c: Vec<String>,
            d: String,
            e: Option<u64>,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Second {
            f: u32,
        }

        assert_de_tokens(
            &Outer {
                y: 0,
                first: First {
                    a: 1,
                    b: true,
                    c: vec!["a".into(), "b".into()],
                    d: "c".into(),
                    e: Some(2),
                },
                second: Second { f: 3 },
                z: 4,
            },
            &[
                Token::Map { len: None },
                Token::Str("y"),
                Token::U32(0),
                Token::Str("a"),
                Token::U32(1),
                Token::Str("b"),
                Token::Bool(true),
                Token::Str("c"),
                Token::Seq { len: Some(2) },
                Token::Str("a"),
                Token::Str("b"),
                Token::SeqEnd,
                Token::Str("d"),
                Token::Str("c"),
                Token::Str("e"),
                Token::U64(2),
                Token::Str("f"),
                Token::U32(3),
                Token::Str("z"),
                Token::U32(4),
                Token::MapEnd,
            ],
        );

        assert_ser_tokens(
            &Outer {
                y: 0,
                first: First {
                    a: 1,
                    b: true,
                    c: vec!["a".into(), "b".into()],
                    d: "c".into(),
                    e: Some(2),
                },
                second: Second { f: 3 },
                z: 4,
            },
            &[
                Token::Map { len: None },
                Token::Str("y"),
                Token::U32(0),
                Token::Str("a"),
                Token::U32(1),
                Token::Str("b"),
                Token::Bool(true),
                Token::Str("c"),
                Token::Seq { len: Some(2) },
                Token::Str("a"),
                Token::Str("b"),
                Token::SeqEnd,
                Token::Str("d"),
                Token::Str("c"),
                Token::Str("e"),
                Token::Some,
                Token::U64(2),
                Token::Str("f"),
                Token::U32(3),
                Token::Str("z"),
                Token::U32(4),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn map_twice() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Outer {
            #[serde(flatten)]
            first: BTreeMap<String, String>,
            #[serde(flatten)]
            between: Inner,
            #[serde(flatten)]
            second: BTreeMap<String, String>,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct Inner {
            y: String,
        }

        assert_de_tokens(
            &Outer {
                first: {
                    let mut first = BTreeMap::new();
                    first.insert("x".to_owned(), "X".to_owned());
                    first.insert("y".to_owned(), "Y".to_owned());
                    first
                },
                between: Inner { y: "Y".to_owned() },
                second: {
                    let mut second = BTreeMap::new();
                    second.insert("x".to_owned(), "X".to_owned());
                    second
                },
            },
            &[
                Token::Map { len: None },
                Token::Str("x"),
                Token::Str("X"),
                Token::Str("y"),
                Token::Str("Y"),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn unsupported_type() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Outer {
            outer: String,
            #[serde(flatten)]
            inner: String,
        }

        assert_ser_tokens_error(
            &Outer {
                outer: "foo".into(),
                inner: "bar".into(),
            },
            &[
                Token::Map { len: None },
                Token::Str("outer"),
                Token::Str("foo"),
            ],
            "can only flatten structs and maps (got a string)",
        );
        assert_de_tokens_error::<Outer>(
            &[
                Token::Map { len: None },
                Token::Str("outer"),
                Token::Str("foo"),
                Token::Str("a"),
                Token::Str("b"),
                Token::MapEnd,
            ],
            "can only flatten structs and maps",
        );
    }

    #[test]
    fn unknown_field() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Outer {
            dummy: String,
            #[serde(flatten)]
            inner: Inner,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Inner {
            foo: HashMap<String, u32>,
        }

        assert_de_tokens_error::<Outer>(
            &[
                Token::Struct {
                    name: "Outer",
                    len: 1,
                },
                Token::Str("dummy"),
                Token::Str("23"),
                Token::Str("foo"),
                Token::Map { len: None },
                Token::Str("a"),
                Token::U32(1),
                Token::Str("b"),
                Token::U32(2),
                Token::MapEnd,
                Token::Str("bar"),
                Token::U32(23),
                Token::StructEnd,
            ],
            "unknown field `bar`",
        );
    }

    #[test]
    fn non_string_keys() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct TestStruct {
            name: String,
            age: u32,
            #[serde(flatten)]
            mapping: HashMap<u32, u32>,
        }

        let mut mapping = HashMap::new();
        mapping.insert(0, 42);
        assert_tokens(
            &TestStruct {
                name: "peter".into(),
                age: 3,
                mapping,
            },
            &[
                Token::Map { len: None },
                Token::Str("name"),
                Token::Str("peter"),
                Token::Str("age"),
                Token::U32(3),
                Token::U32(0),
                Token::U32(42),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn lifetime_propagation() {
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct A<T> {
            #[serde(flatten)]
            t: T,
        }

        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct B<'a> {
            #[serde(flatten, borrow)]
            t: HashMap<&'a str, u32>,
        }

        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct C<'a> {
            #[serde(flatten, borrow)]
            t: HashMap<&'a [u8], u32>,
        }

        let mut owned_map = HashMap::new();
        owned_map.insert("x".to_string(), 42u32);
        assert_tokens(
            &A { t: owned_map },
            &[
                Token::Map { len: None },
                Token::Str("x"),
                Token::U32(42),
                Token::MapEnd,
            ],
        );

        let mut borrowed_map = HashMap::new();
        borrowed_map.insert("x", 42u32);
        assert_ser_tokens(
            &B {
                t: borrowed_map.clone(),
            },
            &[
                Token::Map { len: None },
                Token::BorrowedStr("x"),
                Token::U32(42),
                Token::MapEnd,
            ],
        );

        assert_de_tokens(
            &B { t: borrowed_map },
            &[
                Token::Map { len: None },
                Token::BorrowedStr("x"),
                Token::U32(42),
                Token::MapEnd,
            ],
        );

        let mut borrowed_map = HashMap::new();
        borrowed_map.insert(&b"x"[..], 42u32);
        assert_ser_tokens(
            &C {
                t: borrowed_map.clone(),
            },
            &[
                Token::Map { len: None },
                Token::Seq { len: Some(1) },
                Token::U8(120),
                Token::SeqEnd,
                Token::U32(42),
                Token::MapEnd,
            ],
        );

        assert_de_tokens(
            &C { t: borrowed_map },
            &[
                Token::Map { len: None },
                Token::BorrowedBytes(b"x"),
                Token::U32(42),
                Token::MapEnd,
            ],
        );
    }

    // Regression test for https://github.com/serde-rs/serde/issues/1904
    #[test]
    fn enum_tuple_and_struct() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        enum Outer {
            Tuple(f64, i32),
            Flatten {
                #[serde(flatten)]
                nested: Nested,
            },
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Nested {
            a: i32,
            b: i32,
        }

        assert_tokens(
            &Outer::Tuple(1.2, 3),
            &[
                Token::TupleVariant {
                    name: "Outer",
                    variant: "Tuple",
                    len: 2,
                },
                Token::F64(1.2),
                Token::I32(3),
                Token::TupleVariantEnd,
            ],
        );
        assert_tokens(
            &Outer::Flatten {
                nested: Nested { a: 1, b: 2 },
            },
            &[
                Token::NewtypeVariant {
                    name: "Outer",
                    variant: "Flatten",
                },
                Token::Map { len: None },
                Token::Str("a"),
                Token::I32(1),
                Token::Str("b"),
                Token::I32(2),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn option() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Outer {
            #[serde(flatten)]
            inner1: Option<Inner1>,
            #[serde(flatten)]
            inner2: Option<Inner2>,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Inner1 {
            inner1: i32,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Inner2 {
            inner2: i32,
        }

        assert_tokens(
            &Outer {
                inner1: Some(Inner1 { inner1: 1 }),
                inner2: Some(Inner2 { inner2: 2 }),
            },
            &[
                Token::Map { len: None },
                Token::Str("inner1"),
                Token::I32(1),
                Token::Str("inner2"),
                Token::I32(2),
                Token::MapEnd,
            ],
        );

        assert_tokens(
            &Outer {
                inner1: Some(Inner1 { inner1: 1 }),
                inner2: None,
            },
            &[
                Token::Map { len: None },
                Token::Str("inner1"),
                Token::I32(1),
                Token::MapEnd,
            ],
        );

        assert_tokens(
            &Outer {
                inner1: None,
                inner2: Some(Inner2 { inner2: 2 }),
            },
            &[
                Token::Map { len: None },
                Token::Str("inner2"),
                Token::I32(2),
                Token::MapEnd,
            ],
        );

        assert_tokens(
            &Outer {
                inner1: None,
                inner2: None,
            },
            &[Token::Map { len: None }, Token::MapEnd],
        );
    }

    #[test]
    fn ignored_any() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Outer {
            #[serde(flatten)]
            inner: IgnoredAny,
        }

        assert_de_tokens(
            &Outer { inner: IgnoredAny },
            &[Token::Map { len: None }, Token::MapEnd],
        );

        assert_de_tokens(
            &Outer { inner: IgnoredAny },
            &[
                Token::Struct {
                    name: "DoNotMatter",
                    len: 0,
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn flatten_any_after_flatten_struct() {
        #[derive(PartialEq, Debug)]
        struct Any;

        impl<'de> Deserialize<'de> for Any {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct AnyVisitor;

                impl<'de> Visitor<'de> for AnyVisitor {
                    type Value = Any;

                    fn expecting(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
                    where
                        M: MapAccess<'de>,
                    {
                        while let Some((Any, Any)) = map.next_entry()? {}
                        Ok(Any)
                    }
                }

                deserializer.deserialize_any(AnyVisitor)
            }
        }

        #[derive(Deserialize, PartialEq, Debug)]
        struct Outer {
            #[serde(flatten)]
            inner: Inner,
            #[serde(flatten)]
            extra: Any,
        }

        #[derive(Deserialize, PartialEq, Debug)]
        struct Inner {
            inner: i32,
        }

        let s = Outer {
            inner: Inner { inner: 0 },
            extra: Any,
        };

        assert_de_tokens(
            &s,
            &[
                Token::Map { len: None },
                Token::Str("inner"),
                Token::I32(0),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn alias() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Outer {
            #[serde(flatten)]
            a: AliasStruct,
            b: i32,
        }

        assert_de_tokens(
            &Outer {
                a: AliasStruct {
                    a1: 1,
                    a2: 2,
                    a4: 4,
                },
                b: 7,
            },
            &[
                Token::Struct {
                    name: "Outer",
                    len: 4,
                },
                Token::Str("a1"),
                Token::I32(1),
                Token::Str("a2"),
                Token::I32(2),
                Token::Str("a5"),
                Token::I32(4),
                Token::Str("b"),
                Token::I32(7),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &Outer {
                a: AliasStruct {
                    a1: 1,
                    a2: 2,
                    a4: 4,
                },
                b: 7,
            },
            &[
                Token::Struct {
                    name: "Outer",
                    len: 4,
                },
                Token::Str("a1"),
                Token::I32(1),
                Token::Str("a2"),
                Token::I32(2),
                Token::Str("a6"),
                Token::I32(4),
                Token::Str("b"),
                Token::I32(7),
                Token::StructEnd,
            ],
        );
    }

    mod unit {
        use super::*;

        #[test]
        fn unit() {
            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            struct Response<T> {
                #[serde(flatten)]
                data: T,
                status: usize,
            }

            assert_tokens(
                &Response {
                    data: (),
                    status: 0,
                },
                &[
                    Token::Map { len: None },
                    Token::Str("status"),
                    Token::U64(0),
                    Token::MapEnd,
                ],
            );
        }

        #[test]
        fn unit_struct() {
            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            struct Response<T> {
                #[serde(flatten)]
                data: T,
                status: usize,
            }

            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            struct Unit;

            assert_tokens(
                &Response {
                    data: Unit,
                    status: 0,
                },
                &[
                    Token::Map { len: None },
                    Token::Str("status"),
                    Token::U64(0),
                    Token::MapEnd,
                ],
            );
        }
    }

    mod enum_ {
        use super::*;

        mod externally_tagged {
            use super::*;
            use std::iter::FromIterator;

            #[test]
            fn straightforward() {
                #[derive(Serialize, Deserialize, PartialEq, Debug)]
                enum Data {
                    A {
                        a: i32,
                        #[serde(flatten)]
                        flat: Flat,
                    },
                }

                #[derive(Serialize, Deserialize, PartialEq, Debug)]
                struct Flat {
                    b: i32,
                }

                let data = Data::A {
                    a: 0,
                    flat: Flat { b: 0 },
                };

                assert_tokens(
                    &data,
                    &[
                        Token::NewtypeVariant {
                            name: "Data",
                            variant: "A",
                        },
                        Token::Map { len: None },
                        Token::Str("a"),
                        Token::I32(0),
                        Token::Str("b"),
                        Token::I32(0),
                        Token::MapEnd,
                    ],
                );
            }

            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            struct Flatten {
                #[serde(flatten)]
                data: Enum,

                #[serde(flatten)]
                extra: HashMap<String, String>,
            }

            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            enum Enum {
                Newtype(HashMap<String, String>),
                Tuple(u32, u32),
                Struct { index: u32, value: u32 },
            }

            #[test]
            fn newtype() {
                assert_tokens(
                    &Flatten {
                        data: Enum::Newtype(HashMap::from_iter([("key".into(), "value".into())])),
                        extra: HashMap::from_iter([("extra_key".into(), "extra value".into())]),
                    },
                    &[
                        Token::Map { len: None },
                        Token::Str("Newtype"), // variant
                        Token::Map { len: Some(1) },
                        Token::Str("key"),
                        Token::Str("value"),
                        Token::MapEnd,
                        Token::Str("extra_key"),
                        Token::Str("extra value"),
                        Token::MapEnd,
                    ],
                );
            }

            // Reaches crate::private::de::content::VariantDeserializer::tuple_variant
            // Content::Seq case
            // via FlatMapDeserializer::deserialize_enum
            #[test]
            fn tuple() {
                assert_tokens(
                    &Flatten {
                        data: Enum::Tuple(0, 42),
                        extra: HashMap::from_iter([("extra_key".into(), "extra value".into())]),
                    },
                    &[
                        Token::Map { len: None },
                        Token::Str("Tuple"), // variant
                        Token::Seq { len: Some(2) },
                        Token::U32(0),
                        Token::U32(42),
                        Token::SeqEnd,
                        Token::Str("extra_key"),
                        Token::Str("extra value"),
                        Token::MapEnd,
                    ],
                );
            }

            // Reaches crate::private::de::content::VariantDeserializer::struct_variant
            // Content::Seq case
            // via FlatMapDeserializer::deserialize_enum
            #[test]
            fn struct_from_seq() {
                assert_de_tokens(
                    &Flatten {
                        data: Enum::Struct {
                            index: 0,
                            value: 42,
                        },
                        extra: HashMap::from_iter([("extra_key".into(), "extra value".into())]),
                    },
                    &[
                        Token::Map { len: None },
                        Token::Str("Struct"), // variant
                        Token::Seq { len: Some(2) },
                        Token::U32(0),  // index
                        Token::U32(42), // value
                        Token::SeqEnd,
                        Token::Str("extra_key"),
                        Token::Str("extra value"),
                        Token::MapEnd,
                    ],
                );
            }

            // Reaches crate::private::de::content::VariantDeserializer::struct_variant
            // Content::Map case
            // via FlatMapDeserializer::deserialize_enum
            #[test]
            fn struct_from_map() {
                assert_tokens(
                    &Flatten {
                        data: Enum::Struct {
                            index: 0,
                            value: 42,
                        },
                        extra: HashMap::from_iter([("extra_key".into(), "extra value".into())]),
                    },
                    &[
                        Token::Map { len: None },
                        Token::Str("Struct"), // variant
                        Token::Struct {
                            len: 2,
                            name: "Struct",
                        },
                        Token::Str("index"),
                        Token::U32(0),
                        Token::Str("value"),
                        Token::U32(42),
                        Token::StructEnd,
                        Token::Str("extra_key"),
                        Token::Str("extra value"),
                        Token::MapEnd,
                    ],
                );
            }
        }

        mod adjacently_tagged {
            use super::*;

            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            struct Flatten {
                outer: u32,

                #[serde(flatten)]
                data: NewtypeWrapper,
            }

            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            struct NewtypeWrapper(pub Enum);

            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            #[serde(tag = "tag", content = "content")]
            enum Enum {
                Newtype(NewtypeVariant),
                Struct { index: u32, value: u32 },
            }

            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            struct NewtypeVariant {
                value: u32,
            }

            #[test]
            fn struct_() {
                assert_tokens(
                    &Flatten {
                        outer: 42,
                        data: NewtypeWrapper(Enum::Struct {
                            index: 0,
                            value: 42,
                        }),
                    },
                    &[
                        Token::Map { len: None },
                        Token::Str("outer"),
                        Token::U32(42),
                        Token::Str("tag"),
                        Token::UnitVariant {
                            name: "Enum",
                            variant: "Struct",
                        },
                        Token::Str("content"),
                        Token::Struct {
                            len: 2,
                            name: "Struct",
                        },
                        Token::Str("index"),
                        Token::U32(0),
                        Token::Str("value"),
                        Token::U32(42),
                        Token::StructEnd,
                        Token::MapEnd,
                    ],
                );
            }

            #[test]
            fn newtype() {
                assert_tokens(
                    &Flatten {
                        outer: 42,
                        data: NewtypeWrapper(Enum::Newtype(NewtypeVariant { value: 23 })),
                    },
                    &[
                        Token::Map { len: None },
                        Token::Str("outer"),
                        Token::U32(42),
                        Token::Str("tag"),
                        Token::UnitVariant {
                            name: "Enum",
                            variant: "Newtype",
                        },
                        Token::Str("content"),
                        Token::Struct {
                            len: 1,
                            name: "NewtypeVariant",
                        },
                        Token::Str("value"),
                        Token::U32(23),
                        Token::StructEnd,
                        Token::MapEnd,
                    ],
                );
            }
        }

        mod internally_tagged {
            use super::*;

            #[test]
            fn structs() {
                #[derive(Debug, PartialEq, Serialize, Deserialize)]
                struct Flatten {
                    #[serde(flatten)]
                    x: X,
                    #[serde(flatten)]
                    y: Y,
                }

                #[derive(Debug, PartialEq, Serialize, Deserialize)]
                #[serde(tag = "typeX")]
                enum X {
                    A { a: i32 },
                    B { b: i32 },
                }

                #[derive(Debug, PartialEq, Serialize, Deserialize)]
                #[serde(tag = "typeY")]
                enum Y {
                    C { c: i32 },
                    D { d: i32 },
                }

                assert_tokens(
                    &Flatten {
                        x: X::B { b: 1 },
                        y: Y::D { d: 2 },
                    },
                    &[
                        Token::Map { len: None },
                        Token::Str("typeX"),
                        Token::Str("B"),
                        Token::Str("b"),
                        Token::I32(1),
                        Token::Str("typeY"),
                        Token::Str("D"),
                        Token::Str("d"),
                        Token::I32(2),
                        Token::MapEnd,
                    ],
                );
            }

            #[test]
            fn unit_enum_with_unknown_fields() {
                #[derive(Debug, PartialEq, Deserialize)]
                struct Flatten {
                    #[serde(flatten)]
                    x: X,
                    #[serde(flatten)]
                    y: Y,
                }

                #[derive(Debug, PartialEq, Deserialize)]
                #[serde(tag = "typeX")]
                enum X {
                    A,
                }

                #[derive(Debug, PartialEq, Deserialize)]
                #[serde(tag = "typeY")]
                enum Y {
                    B { c: u32 },
                }

                assert_de_tokens(
                    &Flatten {
                        x: X::A,
                        y: Y::B { c: 0 },
                    },
                    &[
                        Token::Map { len: None },
                        Token::Str("typeX"),
                        Token::Str("A"),
                        Token::Str("typeY"),
                        Token::Str("B"),
                        Token::Str("c"),
                        Token::I32(0),
                        Token::MapEnd,
                    ],
                );
            }
        }

        mod untagged {
            use super::*;

            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            struct Flatten {
                #[serde(flatten)]
                data: Enum,
            }

            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            #[serde(untagged)]
            enum Enum {
                Struct { a: i32 },
            }

            #[test]
            fn struct_() {
                assert_tokens(
                    &Flatten {
                        data: Enum::Struct { a: 0 },
                    },
                    &[
                        Token::Map { len: None },
                        Token::Str("a"),
                        Token::I32(0),
                        Token::MapEnd,
                    ],
                );
            }
        }
    }
}
