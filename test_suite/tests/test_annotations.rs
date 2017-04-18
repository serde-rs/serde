// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

extern crate serde;
use self::serde::{Serialize, Serializer, Deserialize, Deserializer};

extern crate serde_test;
use self::serde_test::{Error, Token, assert_tokens, assert_ser_tokens, assert_de_tokens,
                       assert_de_tokens_error};

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
        if try!(Deserialize::deserialize(de)) {
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
    #[serde(default="MyDefault::my_default")]
    a3: C,
    #[serde(skip_deserializing)]
    a4: D,
    #[serde(skip_deserializing, default="MyDefault::my_default")]
    a5: E,
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
            Token::Struct("DefaultStruct", 3),

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
            Token::Struct("DefaultStruct", 1),

            Token::Str("a1"),
            Token::I32(1),

            Token::StructEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum DefaultEnum<A, B, C, D, E>
where
    C: MyDefault,
    E: MyDefault,
{
    Struct {
        a1: A,
        #[serde(default)]
        a2: B,
        #[serde(default="MyDefault::my_default")]
        a3: C,
        #[serde(skip_deserializing)]
        a4: D,
        #[serde(skip_deserializing, default="MyDefault::my_default")]
        a5: E,
    },
}

#[test]
fn test_default_enum() {
    assert_de_tokens(
        &DefaultEnum::Struct {
             a1: 1,
             a2: 2,
             a3: 3,
             a4: 0,
             a5: 123,
         },
        &[
            Token::StructVariant("DefaultEnum", "Struct", 3),

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
        &DefaultEnum::Struct {
             a1: 1,
             a2: 0,
             a3: 123,
             a4: 0,
             a5: 123,
         },
        &[
            Token::StructVariant("DefaultEnum", "Struct", 3),

            Token::Str("a1"),
            Token::I32(1),

            Token::StructVariantEnd,
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
    #[serde(default="MyDefault::my_default")]
    a: A,
}

// Tests that a struct field does not need to implement std::default::Default if
// it is annotated with `default=...`.
#[test]
fn test_no_std_default() {
    assert_de_tokens(
        &ContainsNoStdDefault { a: NoStdDefault(123) },
        &[Token::Struct("ContainsNoStdDefault", 1), Token::StructEnd],
    );

    assert_de_tokens(
        &ContainsNoStdDefault { a: NoStdDefault(8) },
        &[
            Token::Struct("ContainsNoStdDefault", 1),

            Token::Str("a"),
            Token::NewtypeStruct("NoStdDefault"),
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
    #[serde(deserialize_with="DeserializeWith::deserialize_with", default)]
    c: C,
    #[serde(skip_deserializing, default="MyDefault::my_default")]
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
        &[Token::Struct("ContainsNotDeserialize", 3), Token::StructEnd],
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
            Token::Struct("DefaultStruct", 5),

            Token::Str("whoops1"),
            Token::I32(2),

            Token::Str("a1"),
            Token::I32(1),

            Token::Str("whoops2"),
            Token::Seq(Some(1)),
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
            Token::Struct("DenyUnknown", 2),

            Token::Str("a1"),
            Token::I32(1),

            Token::Str("whoops"),
        ],
        Error::Message("unknown field `whoops`, expected `a1`".to_owned()),
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename="Superhero")]
struct RenameStruct {
    a1: i32,
    #[serde(rename="a3")]
    a2: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename(serialize="SuperheroSer", deserialize="SuperheroDe"))]
struct RenameStructSerializeDeserialize {
    a1: i32,
    #[serde(rename(serialize="a4", deserialize="a5"))]
    a2: i32,
}

#[test]
fn test_rename_struct() {
    assert_tokens(
        &RenameStruct { a1: 1, a2: 2 },
        &[
            Token::Struct("Superhero", 2),

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
            Token::Struct("SuperheroSer", 2),

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
            Token::Struct("SuperheroDe", 2),

            Token::Str("a1"),
            Token::I32(1),

            Token::Str("a5"),
            Token::I32(2),

            Token::StructEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename="Superhero")]
enum RenameEnum {
    #[serde(rename="bruce_wayne")]
    Batman,
    #[serde(rename="clark_kent")]
    Superman(i8),
    #[serde(rename="diana_prince")]
    WonderWoman(i8, i8),
    #[serde(rename="barry_allan")]
    Flash {
        #[serde(rename="b")]
        a: i32,
    },
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename(serialize="SuperheroSer", deserialize="SuperheroDe"))]
enum RenameEnumSerializeDeserialize<A> {
    #[serde(rename(serialize="dick_grayson", deserialize="jason_todd"))]
    Robin {
        a: i8,
        #[serde(rename(serialize="c"))]
        #[serde(rename(deserialize="d"))]
        b: A,
    },
}

#[test]
fn test_rename_enum() {
    assert_tokens(
        &RenameEnum::Batman,
        &[Token::UnitVariant("Superhero", "bruce_wayne")],
    );

    assert_tokens(
        &RenameEnum::Superman(0),
        &[
            Token::NewtypeVariant("Superhero", "clark_kent"),
            Token::I8(0),
        ],
    );

    assert_tokens(
        &RenameEnum::WonderWoman(0, 1),
        &[
            Token::TupleVariant("Superhero", "diana_prince", 2),
            Token::I8(0),
            Token::I8(1),
            Token::TupleVariantEnd,
        ],
    );

    assert_tokens(
        &RenameEnum::Flash { a: 1 },
        &[
            Token::StructVariant("Superhero", "barry_allan", 1),

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
            Token::StructVariant("SuperheroSer", "dick_grayson", 2),

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
            Token::StructVariant("SuperheroDe", "jason_todd", 2),

            Token::Str("a"),
            Token::I8(0),

            Token::Str("d"),
            Token::Str(""),

            Token::StructVariantEnd,
        ],
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
    #[serde(skip_serializing_if="ShouldSkip::should_skip")]
    c: C,
}

#[test]
fn test_skip_serializing_struct() {
    let a = 1;
    assert_ser_tokens(
        &SkipSerializingStruct { a: &a, b: 2, c: 3 },
        &[
            Token::Struct("SkipSerializingStruct", 2),

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
            Token::Struct("SkipSerializingStruct", 1),

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
        #[serde(skip_serializing_if="ShouldSkip::should_skip")]
        c: C,
    },
}

#[test]
fn test_skip_serializing_enum() {
    let a = 1;
    assert_ser_tokens(
        &SkipSerializingEnum::Struct { a: &a, _b: 2, c: 3 },
        &[
            Token::StructVariant("SkipSerializingEnum", "Struct", 2),

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
            Token::StructVariant("SkipSerializingEnum", "Struct", 1),

            Token::Str("a"),
            Token::I8(1),

            Token::StructVariantEnd,
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
    #[serde(serialize_with="SerializeWith::serialize_with")]
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
            Token::Struct("ContainsNotSerialize", 2),

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
    #[serde(serialize_with="SerializeWith::serialize_with")]
    b: B,
}

#[test]
fn test_serialize_with_struct() {
    let a = 1;
    assert_ser_tokens(
        &SerializeWithStruct { a: &a, b: 2 },
        &[
            Token::Struct("SerializeWithStruct", 2),

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
            Token::Struct("SerializeWithStruct", 2),

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
        #[serde(serialize_with="SerializeWith::serialize_with")]
        b: B,
    },
}

#[test]
fn test_serialize_with_enum() {
    let a = 1;
    assert_ser_tokens(
        &SerializeWithEnum::Struct { a: &a, b: 2 },
        &[
            Token::StructVariant("SerializeWithEnum", "Struct", 2),

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
            Token::StructVariant("SerializeWithEnum", "Struct", 2),

            Token::Str("a"),
            Token::I8(1),

            Token::Str("b"),
            Token::Bool(true),

            Token::StructVariantEnd,
        ],
    );
}

#[derive(Debug, PartialEq, Deserialize)]
struct DeserializeWithStruct<B>
where
    B: DeserializeWith,
{
    a: i8,
    #[serde(deserialize_with="DeserializeWith::deserialize_with")]
    b: B,
}

#[test]
fn test_deserialize_with_struct() {
    assert_de_tokens(
        &DeserializeWithStruct { a: 1, b: 2 },
        &[
            Token::Struct("DeserializeWithStruct", 2),

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
            Token::Struct("DeserializeWithStruct", 2),

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
        #[serde(deserialize_with="DeserializeWith::deserialize_with")]
        b: B,
    },
}

#[test]
fn test_deserialize_with_enum() {
    assert_de_tokens(
        &DeserializeWithEnum::Struct { a: 1, b: 2 },
        &[
            Token::StructVariant("DeserializeWithEnum", "Struct", 2),

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
            Token::StructVariant("DeserializeWithEnum", "Struct", 2),

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
            Token::Struct("Superhero", 2),

            Token::Str("a1"),
            Token::I32(1),

            Token::StructEnd,
        ],
        Error::Message("missing field `a3`".to_owned()),
    );

    assert_de_tokens_error::<RenameStructSerializeDeserialize>(
        &[
            Token::Struct("SuperheroDe", 2),

            Token::Str("a1"),
            Token::I32(1),

            Token::StructEnd,
        ],
        Error::Message("missing field `a5`".to_owned()),
    );
}

#[test]
fn test_missing_renamed_field_enum() {
    assert_de_tokens_error::<RenameEnum>(
        &[
            Token::StructVariant("Superhero", "barry_allan", 1),

            Token::StructVariantEnd,
        ],
        Error::Message("missing field `b`".to_owned()),
    );

    assert_de_tokens_error::<RenameEnumSerializeDeserialize<i8>>(
        &[
            Token::StructVariant("SuperheroDe", "jason_todd", 2),

            Token::Str("a"),
            Token::I8(0),

            Token::StructVariantEnd,
        ],
        Error::Message("missing field `d`".to_owned()),
    );
}

#[derive(Debug, PartialEq, Deserialize)]
enum InvalidLengthEnum {
    A(i32, i32, i32),
    B(
        #[serde(skip_deserializing)]
        i32,
        i32,
        i32
    ),
}

#[test]
fn test_invalid_length_enum() {
    assert_de_tokens_error::<InvalidLengthEnum>(
        &[
            Token::TupleVariant("InvalidLengthEnum", "A", 3),
            Token::I32(1),
            Token::TupleVariantEnd,
        ],
        Error::Message("invalid length 1, expected tuple of 3 elements".to_owned()),
    );
    assert_de_tokens_error::<InvalidLengthEnum>(
        &[
            Token::TupleVariant("InvalidLengthEnum", "B", 3),
            Token::I32(1),
            Token::TupleVariantEnd,
        ],
        Error::Message("invalid length 1, expected tuple of 2 elements".to_owned()),
    );
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(into="EnumToU32", from="EnumToU32")]
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
#[serde(into="Option<u32>", from="Option<u32>")]
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

#[test]
fn test_from_into_traits() {
    assert_ser_tokens::<EnumToU32>(&EnumToU32::One, &[Token::Some, Token::U32(1)]);
    assert_ser_tokens::<EnumToU32>(&EnumToU32::Nothing, &[Token::None]);
    assert_de_tokens::<EnumToU32>(&EnumToU32::Two, &[Token::Some, Token::U32(2)]);
    assert_ser_tokens::<StructFromEnum>(&StructFromEnum(Some(5)), &[Token::None]);
    assert_ser_tokens::<StructFromEnum>(&StructFromEnum(None), &[Token::None]);
    assert_de_tokens::<StructFromEnum>(&StructFromEnum(Some(2)), &[Token::Some, Token::U32(2)]);
}
