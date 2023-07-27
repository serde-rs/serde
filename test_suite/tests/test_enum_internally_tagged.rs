#![deny(trivial_numeric_casts)]
#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::enum_variant_names,
    clippy::redundant_field_names,
    clippy::too_many_lines
)]

mod bytes;

use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Newtype(BTreeMap<String, String>);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Struct {
    f: u8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum InternallyTagged {
    Unit,
    NewtypeNewtype(Newtype),
    NewtypeMap(BTreeMap<String, String>),
    NewtypeStruct(Struct),
    Struct { a: u8 },
}

#[test]
fn unit() {
    assert_tokens(
        &InternallyTagged::Unit,
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 1,
            },
            Token::Str("type"),
            Token::Str("Unit"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::Unit,
        &[
            Token::Seq { len: Some(1) },
            Token::Str("Unit"),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn newtype_newtype() {
    assert_tokens(
        &InternallyTagged::NewtypeNewtype(Newtype(BTreeMap::new())),
        &[
            Token::Map { len: Some(1) },
            Token::Str("type"),
            Token::Str("NewtypeNewtype"),
            Token::MapEnd,
        ],
    );
}

#[test]
fn newtype_map() {
    assert_tokens(
        &InternallyTagged::NewtypeMap(BTreeMap::new()),
        &[
            Token::Map { len: Some(1) },
            Token::Str("type"),
            Token::Str("NewtypeMap"),
            Token::MapEnd,
        ],
    );

    assert_de_tokens_error::<InternallyTagged>(
        &[
            Token::Seq { len: Some(2) },
            Token::Str("NewtypeMap"),
            Token::Map { len: Some(0) },
            Token::MapEnd,
            Token::SeqEnd,
        ],
        "invalid type: sequence, expected a map",
    );
}

#[test]
fn newtype_struct() {
    assert_tokens(
        &InternallyTagged::NewtypeStruct(Struct { f: 6 }),
        &[
            Token::Struct {
                name: "Struct",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("NewtypeStruct"),
            Token::Str("f"),
            Token::U8(6),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::NewtypeStruct(Struct { f: 6 }),
        &[
            Token::Seq { len: Some(2) },
            Token::Str("NewtypeStruct"),
            Token::U8(6),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn struct_() {
    assert_tokens(
        &InternallyTagged::Struct { a: 1 },
        &[
            Token::Struct {
                name: "InternallyTagged",
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
        &InternallyTagged::Struct { a: 1 },
        &[
            Token::Seq { len: Some(2) },
            Token::Str("Struct"),
            Token::U8(1),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn wrong_tag() {
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
        "unknown variant `Z`, expected one of `Unit`, `NewtypeNewtype`, `NewtypeMap`, `NewtypeStruct`, `Struct`",
    );
}

#[test]
fn untagged_variant() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "kind")]
    enum InternallyTagged {
        Tagged {
            a: u8,
        },
        #[serde(untagged)]
        Untagged {
            kind: String,
            b: u8,
        },
    }

    assert_de_tokens(
        &InternallyTagged::Tagged { a: 1 },
        &[
            Token::Map { len: Some(2) },
            Token::Str("kind"),
            Token::Str("Tagged"),
            Token::Str("a"),
            Token::U8(1),
            Token::MapEnd,
        ],
    );

    assert_tokens(
        &InternallyTagged::Tagged { a: 1 },
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 2,
            },
            Token::Str("kind"),
            Token::Str("Tagged"),
            Token::Str("a"),
            Token::U8(1),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::Untagged {
            kind: "Foo".to_owned(),
            b: 2,
        },
        &[
            Token::Map { len: Some(2) },
            Token::Str("kind"),
            Token::Str("Foo"),
            Token::Str("b"),
            Token::U8(2),
            Token::MapEnd,
        ],
    );

    assert_tokens(
        &InternallyTagged::Untagged {
            kind: "Foo".to_owned(),
            b: 2,
        },
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 2,
            },
            Token::Str("kind"),
            Token::Str("Foo"),
            Token::Str("b"),
            Token::U8(2),
            Token::StructEnd,
        ],
    );

    assert_tokens(
        &InternallyTagged::Untagged {
            kind: "Tagged".to_owned(),
            b: 2,
        },
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 2,
            },
            Token::Str("kind"),
            Token::Str("Tagged"),
            Token::Str("b"),
            Token::U8(2),
            Token::StructEnd,
        ],
    );
}

mod string_and_bytes {
    use super::*;

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

    #[test]
    fn string_from_string() {
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
    }

    #[test]
    fn string_from_bytes() {
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
    }

    #[test]
    fn bytes_from_string() {
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
    }

    #[test]
    fn bytes_from_bytes() {
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
    }

    #[test]
    fn bytes_from_seq() {
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
}

#[test]
fn struct_variant_containing_unit_variant() {
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
        &Level::Info,
        &[
            Token::Enum { name: "Level" },
            Token::BorrowedStr("Info"),
            Token::Unit,
        ],
    );

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
            Token::Enum { name: "Level" },
            Token::BorrowedStr("Info"),
            Token::Unit,
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
            Token::Enum { name: "Level" },
            Token::BorrowedStr("Info"),
            Token::Unit,
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &Message::Log { level: Level::Info },
        &[
            Token::Seq { len: Some(2) },
            Token::Str("Log"),
            Token::Enum { name: "Level" },
            Token::BorrowedStr("Info"),
            Token::Unit,
            Token::SeqEnd,
        ],
    );
}

#[test]
fn borrow() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum Input<'a> {
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
fn newtype_variant_containing_externally_tagged_enum() {
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

    // Reaches crate::private::de::content::VariantDeserializer::tuple_variant
    // Content::Seq case
    // via ContentDeserializer::deserialize_enum
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

    // Reaches crate::private::de::content::VariantDeserializer::struct_variant
    // Content::Map case
    // via ContentDeserializer::deserialize_enum
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

    // Reaches crate::private::de::content::VariantDeserializer::struct_variant
    // Content::Seq case
    // via ContentDeserializer::deserialize_enum
    assert_de_tokens(
        &Outer::Inner(Inner::Struct { f: 1 }),
        &[
            Token::Map { len: Some(2) },
            Token::Str("type"),
            Token::Str("Inner"),
            Token::Str("Struct"),
            Token::Seq { len: Some(1) },
            Token::U8(1), // f
            Token::SeqEnd,
            Token::MapEnd,
        ],
    );
}

#[test]
fn newtype_variant_containing_unit_struct() {
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
fn with_skipped_conflict() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(tag = "t")]
    enum Data {
        A,
        #[serde(skip)]
        #[allow(dead_code)]
        B {
            t: String,
        },
        C {
            #[serde(default, skip)]
            t: String,
        },
    }

    let data = Data::C { t: String::new() };

    assert_tokens(
        &data,
        &[
            Token::Struct {
                name: "Data",
                len: 1,
            },
            Token::Str("t"),
            Token::Str("C"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn containing_flatten() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(tag = "t")]
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
            Token::Map { len: None },
            Token::Str("t"),
            Token::Str("A"),
            Token::Str("a"),
            Token::I32(0),
            Token::Str("b"),
            Token::I32(0),
            Token::MapEnd,
        ],
    );
}

#[test]
fn newtype_variant_containing_unit() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(tag = "t")]
    enum Data {
        A(()),
    }

    assert_tokens(
        &Data::A(()),
        &[
            Token::Map { len: Some(1) },
            Token::Str("t"),
            Token::Str("A"),
            Token::MapEnd,
        ],
    );
}

#[test]
fn unit_variant_with_unknown_fields() {
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(tag = "t")]
    enum Data {
        A,
    }

    let data = Data::A;

    assert_de_tokens(
        &data,
        &[
            Token::Map { len: None },
            Token::Str("t"),
            Token::Str("A"),
            Token::Str("b"),
            Token::I32(0),
            Token::MapEnd,
        ],
    );
}

#[test]
fn expecting_message() {
    #[derive(Deserialize)]
    #[serde(tag = "tag")]
    #[serde(expecting = "something strange...")]
    enum Enum {
        InternallyTagged,
    }

    assert_de_tokens_error::<Enum>(
        &[Token::Str("InternallyTagged")],
        r#"invalid type: string "InternallyTagged", expected something strange..."#,
    );

    // Check that #[serde(expecting = "...")] doesn't affect variant identifier error message
    assert_de_tokens_error::<Enum>(
        &[Token::Map { len: None }, Token::Str("tag"), Token::Unit],
        "invalid type: unit value, expected variant identifier",
    );
}
