use std::default;

use token::{
    Error,
    Token,
    assert_tokens,
    assert_ser_tokens,
    assert_de_tokens,
    assert_de_tokens_error
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Default {
    a1: i32,
    #[serde(default)]
    a2: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DisallowUnknown {
    a1: i32,
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

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct SkipSerializingFields<A: default::Default> {
    a: i8,
    #[serde(skip_serializing, default)]
    b: A,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct SkipSerializingIfEmptyFields<A: default::Default> {
    a: i8,
    #[serde(skip_serializing_if_empty, default)]
    b: Vec<A>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct SkipSerializingIfNoneFields<A: default::Default> {
    a: i8,
    #[serde(skip_serializing_if_none, default)]
    b: Option<A>,
}

#[test]
fn test_default() {
    assert_de_tokens(
        &Default { a1: 1, a2: 2 },
        vec![
            Token::StructStart("Default", Some(2)),

            Token::MapSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::MapSep,
            Token::Str("a2"),
            Token::I32(2),

            Token::MapEnd,
        ]
    );

    assert_de_tokens(
        &Default { a1: 1, a2: 0 },
        vec![
            Token::StructStart("Default", Some(1)),

            Token::MapSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::MapEnd,
        ]
    );
}

#[test]
fn test_ignore_unknown() {
    // 'Default' allows unknown. Basic smoke test of ignore...
    assert_de_tokens(
        &Default { a1: 1, a2: 2},
        vec![
            Token::StructStart("Default", Some(5)),

            Token::MapSep,
            Token::Str("whoops1"),
            Token::I32(2),

            Token::MapSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::MapSep,
            Token::Str("whoops2"),
            Token::SeqStart(Some(1)),
            Token::SeqSep,
            Token::I32(2),
            Token::SeqEnd,

            Token::MapSep,
            Token::Str("a2"),
            Token::I32(2),

            Token::MapSep,
            Token::Str("whoops3"),
            Token::I32(2),

            Token::MapEnd,
        ]
    );

    assert_de_tokens_error::<DisallowUnknown>(
        vec![
            Token::StructStart("DisallowUnknown", Some(2)),

            Token::MapSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::MapSep,
            Token::Str("whoops"),
            Token::I32(2),

            Token::MapEnd,
        ],
        Error::UnknownFieldError("whoops".to_owned())
    );
}

#[test]
fn test_rename_struct() {
    assert_tokens(
        &RenameStruct { a1: 1, a2: 2 },
        vec![
            Token::StructStart("Superhero", Some(2)),

            Token::MapSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::MapSep,
            Token::Str("a3"),
            Token::I32(2),

            Token::MapEnd,
        ]
    );
}

#[test]
fn test_rename_struct_serialize_deserialize() {
    assert_ser_tokens(
        &RenameStructSerializeDeserialize { a1: 1, a2: 2 },
        &[
            Token::StructStart("SuperheroSer", Some(2)),

            Token::MapSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::MapSep,
            Token::Str("a4"),
            Token::I32(2),

            Token::MapEnd,
        ]
    );

    assert_de_tokens(
        &RenameStructSerializeDeserialize { a1: 1, a2: 2 },
        vec![
            Token::StructStart("SuperheroDe", Some(2)),

            Token::MapSep,
            Token::Str("a1"),
            Token::I32(1),

            Token::MapSep,
            Token::Str("a5"),
            Token::I32(2),

            Token::MapEnd,
        ]
    );
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
            Token::EnumNewtype("Superhero", "clark_kent"),
            Token::I8(0),
        ]
    );

    assert_tokens(
        &RenameEnum::WonderWoman(0, 1),
        vec![
            Token::EnumSeqStart("Superhero", "diana_prince", Some(2)),

            Token::SeqSep,
            Token::I8(0),

            Token::SeqSep,
            Token::I8(1),

            Token::SeqEnd,
        ]
    );

    assert_tokens(
        &RenameEnum::Flash { a: 1 },
        vec![
            Token::EnumMapStart("Superhero", "barry_allan", Some(1)),

            Token::MapSep,
            Token::Str("b"),
            Token::I32(1),

            Token::MapEnd,
        ]
    );
}

#[test]
fn test_enum_serialize_deserialize() {
    assert_ser_tokens(
        &RenameEnumSerializeDeserialize::Robin {
            a: 0,
            b: String::new(),
        },
        &[
            Token::EnumMapStart("SuperheroSer", "dick_grayson", Some(2)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(0),

            Token::MapSep,
            Token::Str("c"),
            Token::Str(""),

            Token::MapEnd,
        ]
    );

    assert_de_tokens(
        &RenameEnumSerializeDeserialize::Robin {
            a: 0,
            b: String::new(),
        },
        vec![
            Token::EnumMapStart("SuperheroDe", "jason_todd", Some(2)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(0),

            Token::MapSep,
            Token::Str("d"),
            Token::Str(""),

            Token::MapEnd,
        ]
    );
}

#[test]
fn test_skip_serializing_fields() {
    assert_ser_tokens(
        &SkipSerializingFields {
            a: 1,
            b: 2,
        },
        &[
            Token::StructStart("SkipSerializingFields", Some(1)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapEnd,
        ]
    );

    assert_de_tokens(
        &SkipSerializingFields {
            a: 1,
            b: 0,
        },
        vec![
            Token::StructStart("SkipSerializingFields", Some(1)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapEnd,
        ]
    );
}

#[test]
fn test_skip_serializing_fields_if_empty() {
    assert_ser_tokens(
        &SkipSerializingIfEmptyFields::<i32> {
            a: 1,
            b: vec![],
        },
        &[
            Token::StructStart("SkipSerializingIfEmptyFields", Some(1)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapEnd,
        ]
    );

    assert_de_tokens(
        &SkipSerializingIfEmptyFields::<i32> {
            a: 1,
            b: vec![],
        },
        vec![
            Token::StructStart("SkipSerializingIfEmptyFields", Some(1)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapEnd,
        ]
    );

    assert_ser_tokens(
        &SkipSerializingIfEmptyFields {
            a: 1,
            b: vec![2],
        },
        &[
            Token::StructStart("SkipSerializingIfEmptyFields", Some(2)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapSep,
            Token::Str("b"),
            Token::SeqStart(Some(1)),
            Token::SeqSep,
            Token::I32(2),
            Token::SeqEnd,

            Token::MapEnd,
        ]
    );

    assert_de_tokens(
        &SkipSerializingIfEmptyFields {
            a: 1,
            b: vec![2],
        },
        vec![
            Token::StructStart("SkipSerializingIfEmptyFields", Some(2)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapSep,
            Token::Str("b"),
            Token::SeqStart(Some(1)),
            Token::SeqSep,
            Token::I32(2),
            Token::SeqEnd,

            Token::MapEnd,
        ]
    );
}

#[test]
fn test_skip_serializing_fields_if_none() {
    assert_ser_tokens(
        &SkipSerializingIfNoneFields::<i32> {
            a: 1,
            b: None,
        },
        &[
            Token::StructStart("SkipSerializingIfNoneFields", Some(1)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapEnd,
        ]
    );

    assert_de_tokens(
        &SkipSerializingIfNoneFields::<i32> {
            a: 1,
            b: None,
        },
        vec![
            Token::StructStart("SkipSerializingIfNoneFields", Some(1)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapEnd,
        ]
    );

    assert_ser_tokens(
        &SkipSerializingIfNoneFields {
            a: 1,
            b: Some(2),
        },
        &[
            Token::StructStart("SkipSerializingIfNoneFields", Some(2)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapSep,
            Token::Str("b"),
            Token::Option(true),
            Token::I32(2),

            Token::MapEnd,
        ]
    );

    assert_de_tokens(
        &SkipSerializingIfNoneFields {
            a: 1,
            b: Some(2),
        },
        vec![
            Token::StructStart("SkipSerializingIfNoneFields", Some(2)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapSep,
            Token::Str("b"),
            Token::Option(true),
            Token::I32(2),

            Token::MapEnd,
        ]
    );
}
