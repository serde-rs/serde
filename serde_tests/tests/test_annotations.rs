use std::default;

use token::{Token, assert_tokens, assert_ser_tokens, assert_de_tokens};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Default {
    a1: i32,
    #[serde(default)]
    a2: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Rename {
    a1: i32,
    #[serde(rename="a3")]
    a2: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FormatRename {
    a1: i32,
    #[serde(rename(xml= "a4", token="a5"))]
    a2: i32,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum SerEnum<A> {
    Map {
        a: i8,
        #[serde(rename(xml= "c", token="d"))]
        b: A,
    },
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct SkipSerializingFields<A: default::Default> {
    a: i8,
    #[serde(skip_serializing, default)]
    b: A,
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
fn test_rename() {
    assert_tokens(
        &Rename { a1: 1, a2: 2 },
        vec![
            Token::StructStart("Rename", Some(2)),

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
fn test_format_rename() {
    assert_tokens(
        &FormatRename { a1: 1, a2: 2 },
        vec![
            Token::StructStart("FormatRename", Some(2)),

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
fn test_enum_format_rename() {
    assert_tokens(
        &SerEnum::Map {
            a: 0,
            b: String::new(),
        },
        vec![
            Token::EnumMapStart("SerEnum", "Map", Some(2)),

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
