use std::default::Default;
use serde::{Serialize, Serializer, Deserialize, Deserializer};

use token::{
    Error,
    Token,
    assert_tokens,
    assert_ser_tokens,
    assert_de_tokens,
    assert_de_tokens_error
};

trait Trait: Sized {
    fn my_default() -> Self;

    fn should_skip(&self) -> bool;

    fn serialize_with<S>(&self, ser: &mut S) -> Result<(), S::Error>
        where S: Serializer;

    fn deserialize_with<D>(de: &mut D) -> Result<Self, D::Error>
        where D: Deserializer;
}

impl Trait for i32 {
    fn my_default() -> Self { 123 }

    fn should_skip(&self) -> bool { *self == 123 }

    fn serialize_with<S>(&self, ser: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        if *self == 123 {
            true.serialize(ser)
        } else {
            false.serialize(ser)
        }
    }

    fn deserialize_with<D>(de: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        if try!(Deserialize::deserialize(de)) {
            Ok(123)
        } else {
            Ok(2)
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DefaultStruct<A, B: Default, C> where C: Trait {
    a1: A,
    #[serde(default)]
    a2: B,
    #[serde(default="Trait::my_default()")]
    a3: C,
}

#[test]
fn test_default_struct() {
    assert_de_tokens(
        &DefaultStruct { a1: 1, a2: 2, a3: 3 },
        vec![
            Token::StructStart("DefaultStruct", Some(3)),

            Token::StructSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::StructSep,
            Token::Str("a2"),
            Token::I32(2),

            Token::StructSep,
            Token::Str("a3"),
            Token::I32(3),

            Token::StructEnd,
        ]
    );

    assert_de_tokens(
        &DefaultStruct { a1: 1, a2: 0, a3: 123 },
        vec![
            Token::StructStart("DefaultStruct", Some(1)),

            Token::StructSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::StructEnd,
        ]
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum DefaultEnum<A, B: Default, C> where C: Trait {
    Struct {
        a1: A,
        #[serde(default)]
        a2: B,
        #[serde(default="Trait::my_default()")]
        a3: C,
    }
}

#[test]
fn test_default_enum() {
    assert_de_tokens(
        &DefaultEnum::Struct { a1: 1, a2: 2, a3: 3 },
        vec![
            Token::EnumMapStart("DefaultEnum", "Struct", Some(3)),

            Token::EnumMapSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::EnumMapSep,
            Token::Str("a2"),
            Token::I32(2),

            Token::EnumMapSep,
            Token::Str("a3"),
            Token::I32(3),

            Token::EnumMapEnd,
        ]
    );

    assert_de_tokens(
        &DefaultEnum::Struct { a1: 1, a2: 0, a3: 123 },
        vec![
            Token::EnumMapStart("DefaultEnum", "Struct", Some(3)),

            Token::EnumMapSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::EnumMapEnd,
        ]
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
        &DefaultStruct { a1: 1, a2: 2, a3: 3 },
        vec![
            Token::StructStart("DefaultStruct", Some(5)),

            Token::StructSep,
            Token::Str("whoops1"),
            Token::I32(2),

            Token::StructSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::StructSep,
            Token::Str("whoops2"),
            Token::SeqStart(Some(1)),
            Token::SeqSep,
            Token::I32(2),
            Token::SeqEnd,

            Token::StructSep,
            Token::Str("a2"),
            Token::I32(2),

            Token::StructSep,
            Token::Str("whoops3"),
            Token::I32(2),

            Token::StructSep,
            Token::Str("a3"),
            Token::I32(3),

            Token::StructEnd,
        ]
    );

    assert_de_tokens_error::<DenyUnknown>(
        vec![
            Token::StructStart("DenyUnknown", Some(2)),

            Token::StructSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::StructSep,
            Token::Str("whoops"),
            Token::I32(2),

            Token::StructEnd,
        ],
        Error::UnknownFieldError("whoops".to_owned())
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
        vec![
            Token::StructStart("Superhero", Some(2)),

            Token::StructSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::StructSep,
            Token::Str("a3"),
            Token::I32(2),

            Token::StructEnd,
        ]
    );

    assert_ser_tokens(
        &RenameStructSerializeDeserialize { a1: 1, a2: 2 },
        &[
            Token::StructStart("SuperheroSer", Some(2)),

            Token::StructSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::StructSep,
            Token::Str("a4"),
            Token::I32(2),

            Token::StructEnd,
        ]
    );

    assert_de_tokens(
        &RenameStructSerializeDeserialize { a1: 1, a2: 2 },
        vec![
            Token::StructStart("SuperheroDe", Some(2)),

            Token::StructSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::StructSep,
            Token::Str("a5"),
            Token::I32(2),

            Token::StructEnd,
        ]
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
        #[serde(rename(serialize="c", deserialize="d"))]
        b: A,
    },
}

#[test]
fn test_rename_enum() {
    assert_tokens(
        &RenameEnum::Batman,
        vec![
            Token::EnumUnit("Superhero", "bruce_wayne"),
        ]
    );

    assert_tokens(
        &RenameEnum::Superman(0),
        vec![
            Token::EnumNewType("Superhero", "clark_kent"),
            Token::I8(0),
        ]
    );

    assert_tokens(
        &RenameEnum::WonderWoman(0, 1),
        vec![
            Token::EnumSeqStart("Superhero", "diana_prince", Some(2)),

            Token::EnumSeqSep,
            Token::I8(0),

            Token::EnumSeqSep,
            Token::I8(1),

            Token::EnumSeqEnd,
        ]
    );

    assert_tokens(
        &RenameEnum::Flash { a: 1 },
        vec![
            Token::EnumMapStart("Superhero", "barry_allan", Some(1)),

            Token::EnumMapSep,
            Token::Str("b"),
            Token::I32(1),

            Token::EnumMapEnd,
        ]
    );

    assert_ser_tokens(
        &RenameEnumSerializeDeserialize::Robin {
            a: 0,
            b: String::new(),
        },
        &[
            Token::EnumMapStart("SuperheroSer", "dick_grayson", Some(2)),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I8(0),

            Token::EnumMapSep,
            Token::Str("c"),
            Token::Str(""),

            Token::EnumMapEnd,
        ]
    );

    assert_de_tokens(
        &RenameEnumSerializeDeserialize::Robin {
            a: 0,
            b: String::new(),
        },
        vec![
            Token::EnumMapStart("SuperheroDe", "jason_todd", Some(2)),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I8(0),

            Token::EnumMapSep,
            Token::Str("d"),
            Token::Str(""),

            Token::EnumMapEnd,
        ]
    );
}

#[derive(Debug, PartialEq, Serialize)]
struct SkipSerializingStruct<'a, B, C, D, E> where C: Trait {
    a: &'a i8,
    #[serde(skip_serializing)]
    b: B,
    #[serde(skip_serializing_if="self.c.should_skip()")]
    c: C,
    #[serde(skip_serializing_if_none)]
    d: Option<D>,
    #[serde(skip_serializing_if_empty)]
    e: Vec<E>,
}

#[test]
fn test_skip_serializing_struct() {
    let a = 1;
    assert_ser_tokens(
        &SkipSerializingStruct {
            a: &a,
            b: 2,
            c: 3,
            d: Some(4),
            e: vec![5],
        },
        &[
            Token::StructStart("SkipSerializingStruct", Some(4)),

            Token::StructSep,
            Token::Str("a"),
            Token::I8(1),

            Token::StructSep,
            Token::Str("c"),
            Token::I32(3),

            Token::StructSep,
            Token::Str("d"),
            Token::Option(true),
            Token::I32(4),

            Token::StructSep,
            Token::Str("e"),
            Token::SeqStart(Some(1)),
            Token::SeqSep,
            Token::I32(5),
            Token::SeqEnd,

            Token::StructEnd,
        ]
    );

    assert_ser_tokens(
        &SkipSerializingStruct {
            a: &a,
            b: 2,
            c: 123,
            d: None::<u8>,
            e: Vec::<u8>::new(),
        },
        &[
            Token::StructStart("SkipSerializingStruct", Some(1)),

            Token::StructSep,
            Token::Str("a"),
            Token::I8(1),

            Token::StructEnd,
        ]
    );
}

#[derive(Debug, PartialEq, Serialize)]
enum SkipSerializingEnum<'a, B, C, D, E> where C: Trait {
    Struct {
        a: &'a i8,
        #[serde(skip_serializing)]
        _b: B,
        #[serde(skip_serializing_if="self.c.should_skip()")]
        c: C,
        #[serde(skip_serializing_if_none)]
        d: Option<D>,
        #[serde(skip_serializing_if_empty)]
        e: Vec<E>,
    }
}

#[test]
fn test_skip_serializing_enum() {
    let a = 1;
    assert_ser_tokens(
        &SkipSerializingEnum::Struct {
            a: &a,
            _b: 2,
            c: 3,
            d: Some(4),
            e: vec![5],
        },
        &[
            Token::EnumMapStart("SkipSerializingEnum", "Struct", Some(4)),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::EnumMapSep,
            Token::Str("c"),
            Token::I32(3),

            Token::EnumMapSep,
            Token::Str("d"),
            Token::Option(true),
            Token::I32(4),

            Token::EnumMapSep,
            Token::Str("e"),
            Token::SeqStart(Some(1)),
            Token::SeqSep,
            Token::I32(5),
            Token::SeqEnd,

            Token::EnumMapEnd,
        ]
    );

    assert_ser_tokens(
        &SkipSerializingEnum::Struct {
            a: &a,
            _b: 2,
            c: 123,
            d: None::<u8>,
            e: Vec::<u8>::new(),
        },
        &[
            Token::EnumMapStart("SkipSerializingEnum", "Struct", Some(1)),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::EnumMapEnd,
        ]
    );
}

#[derive(Debug, PartialEq, Serialize)]
struct SerializeWithStruct<'a, B> where B: Trait {
    a: &'a i8,
    #[serde(serialize_with="self.b.serialize_with(serializer)")]
    b: B,
}

#[test]
fn test_serialize_with_struct() {
    let a = 1;
    assert_ser_tokens(
        &SerializeWithStruct {
            a: &a,
            b: 2,
        },
        &[
            Token::StructStart("SerializeWithStruct", Some(2)),

            Token::StructSep,
            Token::Str("a"),
            Token::I8(1),

            Token::StructSep,
            Token::Str("b"),
            Token::Bool(false),

            Token::StructEnd,
        ]
    );

    assert_ser_tokens(
        &SerializeWithStruct {
            a: &a,
            b: 123,
        },
        &[
            Token::StructStart("SerializeWithStruct", Some(2)),

            Token::StructSep,
            Token::Str("a"),
            Token::I8(1),

            Token::StructSep,
            Token::Str("b"),
            Token::Bool(true),

            Token::StructEnd,
        ]
    );
}

#[derive(Debug, PartialEq, Serialize)]
enum SerializeWithEnum<'a, B> where B: Trait {
    Struct {
        a: &'a i8,
        #[serde(serialize_with="self.b.serialize_with(serializer)")]
        b: B,
    }
}

#[test]
fn test_serialize_with_enum() {
    let a = 1;
    assert_ser_tokens(
        &SerializeWithEnum::Struct {
            a: &a,
            b: 2,
        },
        &[
            Token::EnumMapStart("SerializeWithEnum", "Struct", Some(2)),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::EnumMapSep,
            Token::Str("b"),
            Token::Bool(false),

            Token::EnumMapEnd,
        ]
    );

    assert_ser_tokens(
        &SerializeWithEnum::Struct {
            a: &a,
            b: 123,
        },
        &[
            Token::EnumMapStart("SerializeWithEnum", "Struct", Some(2)),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::EnumMapSep,
            Token::Str("b"),
            Token::Bool(true),

            Token::EnumMapEnd,
        ]
    );
}

#[derive(Debug, PartialEq, Deserialize)]
struct DeserializeWithStruct<B> where B: Trait {
    a: i8,
    #[serde(deserialize_with="Trait::deserialize_with(deserializer)")]
    b: B,
}

#[test]
fn test_deserialize_with_struct() {
    assert_de_tokens(
        &DeserializeWithStruct {
            a: 1,
            b: 2,
        },
        vec![
            Token::StructStart("DeserializeWithStruct", Some(2)),

            Token::StructSep,
            Token::Str("a"),
            Token::I8(1),

            Token::StructSep,
            Token::Str("b"),
            Token::Bool(false),

            Token::StructEnd,
        ]
    );

    assert_de_tokens(
        &DeserializeWithStruct {
            a: 1,
            b: 123,
        },
        vec![
            Token::StructStart("DeserializeWithStruct", Some(2)),

            Token::StructSep,
            Token::Str("a"),
            Token::I8(1),

            Token::StructSep,
            Token::Str("b"),
            Token::Bool(true),

            Token::StructEnd,
        ]
    );
}

#[derive(Debug, PartialEq, Deserialize)]
enum DeserializeWithEnum<B> where B: Trait {
    Struct {
        a: i8,
        #[serde(deserialize_with="Trait::deserialize_with(deserializer)")]
        b: B,
    }
}

#[test]
fn test_deserialize_with_enum() {
    assert_de_tokens(
        &DeserializeWithEnum::Struct {
            a: 1,
            b: 2,
        },
        vec![
            Token::EnumMapStart("DeserializeWithEnum", "Struct", Some(2)),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::EnumMapSep,
            Token::Str("b"),
            Token::Bool(false),

            Token::EnumMapEnd,
        ]
    );

    assert_de_tokens(
        &DeserializeWithEnum::Struct {
            a: 1,
            b: 123,
        },
        vec![
            Token::EnumMapStart("DeserializeWithEnum", "Struct", Some(2)),

            Token::EnumMapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::EnumMapSep,
            Token::Str("b"),
            Token::Bool(true),

            Token::EnumMapEnd,
        ]
    );
}
