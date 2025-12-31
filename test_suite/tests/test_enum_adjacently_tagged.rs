#![deny(trivial_numeric_casts)]
#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::enum_variant_names,
    clippy::redundant_field_names,
    clippy::too_many_lines
)]

use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum AdjacentlyTagged<T> {
    Unit,
    Newtype(T),
    Tuple(u8, u8),
    Struct { f: u8 },
}

mod unit {
    use super::*;

    #[test]
    fn map_str_tag_only() {
        // Map: tag only
        assert_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 1,
                },
                Token::Str("t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::StructEnd,
            ],
        );

        // Map: tag only and incorrect hint for number of elements
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 2,
                },
                Token::Str("t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn map_int_tag_only() {
        // Map: tag (as number) only
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 1,
                },
                Token::U16(0),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn map_bytes_tag_only() {
        // Map: tag only
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 1,
                },
                Token::Bytes(b"t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::StructEnd,
            ],
        );

        // Map: tag only
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 1,
                },
                Token::BorrowedBytes(b"t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn map_str_tag_content() {
        // Map: tag + content
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 2,
                },
                Token::Str("t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::Str("c"),
                Token::Unit,
                Token::StructEnd,
            ],
        );
        // Map: content + tag
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
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::StructEnd,
            ],
        );

        // Map: tag + content + excess fields (f, g, h)
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
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::Str("g"),
                Token::Unit,
                Token::Str("c"),
                Token::Unit,
                Token::Str("h"),
                Token::Unit,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn map_int_tag_content() {
        // Map: tag (as number) + content (as number)
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 2,
                },
                Token::U8(0),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::U8(1),
                Token::Unit,
                Token::StructEnd,
            ],
        );

        // Map: content (as number) + tag (as number)
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 2,
                },
                Token::U64(1),
                Token::Unit,
                Token::U64(0),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn map_bytes_tag_content() {
        // Map: tag + content
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 2,
                },
                Token::BorrowedBytes(b"t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::BorrowedBytes(b"c"),
                Token::Unit,
                Token::StructEnd,
            ],
        );

        // Map: content + tag
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 2,
                },
                Token::Bytes(b"c"),
                Token::Unit,
                Token::Bytes(b"t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn seq_tag_content() {
        // Seq: tag and content
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Seq { len: Some(2) },
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Unit",
                },
                Token::Unit,
                Token::SeqEnd,
            ],
        );

        // Seq: tag (as string) and content
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Seq { len: None },
                Token::Str("Unit"), // tag
                Token::Unit,        // content
                Token::SeqEnd,
            ],
        );

        // Seq: tag (as borrowed string) and content
        assert_de_tokens(
            &AdjacentlyTagged::Unit::<u8>,
            &[
                Token::Seq { len: None },
                Token::BorrowedStr("Unit"), // tag
                Token::Unit,                // content
                Token::SeqEnd,
            ],
        );
    }
}

mod newtype {
    use super::*;

    #[test]
    fn map_tag_only() {
        // optional newtype with no content field
        assert_de_tokens(
            &AdjacentlyTagged::Newtype::<Option<u8>>(None),
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 1,
                },
                Token::Str("t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Newtype",
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn map_tag_content() {
        let value = AdjacentlyTagged::Newtype::<u8>(1);

        // Map: tag + content
        assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 2,
                },
                Token::Str("t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Newtype",
                },
                Token::Str("c"),
                Token::U8(1),
                Token::StructEnd,
            ],
        );

        // Map: content + tag
        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 2,
                },
                Token::Str("c"),
                Token::U8(1),
                Token::Str("t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Newtype",
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn seq() {
        let value = AdjacentlyTagged::Newtype::<u8>(1);

        // Seq: tag and content
        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: Some(2) },
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Newtype",
                },
                Token::U8(1),
                Token::SeqEnd,
            ],
        );

        // Seq: tag (as string) and content
        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: None },
                Token::Str("Newtype"), // tag
                Token::U8(1),          // content
                Token::SeqEnd,
            ],
        );

        // Seq: tag (as borrowed string) and content
        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: None },
                Token::BorrowedStr("Newtype"), // tag
                Token::U8(1),                  // content
                Token::SeqEnd,
            ],
        );
    }
}

// Reaches crate::private::de::content::ContentDeserializer::deserialize_newtype_struct
// in Content::Newtype case
#[test]
fn newtype_with_newtype() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct NewtypeStruct(u32);

    assert_de_tokens(
        &AdjacentlyTagged::Newtype(NewtypeStruct(5)),
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::NewtypeStruct {
                name: "NewtypeStruct",
            },
            Token::U32(5),
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Newtype",
            },
            Token::StructEnd,
        ],
    );
}

mod tuple {
    use super::*;

    #[test]
    fn map() {
        let value = AdjacentlyTagged::Tuple::<u8>(1, 1);

        // Map: tag + content
        assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 2,
                },
                Token::Str("t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Tuple",
                },
                Token::Str("c"),
                Token::Tuple { len: 2 },
                Token::U8(1),
                Token::U8(1),
                Token::TupleEnd,
                Token::StructEnd,
            ],
        );

        // Map: content + tag
        assert_de_tokens(
            &value,
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
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Tuple",
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn seq() {
        let value = AdjacentlyTagged::Tuple::<u8>(1, 1);

        // Seq: tag + content
        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: Some(2) },
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Tuple",
                },
                Token::Tuple { len: 2 },
                Token::U8(1),
                Token::U8(1),
                Token::TupleEnd,
                Token::SeqEnd,
            ],
        );
    }
}

mod struct_ {
    use super::*;

    #[test]
    fn map() {
        let value = AdjacentlyTagged::Struct::<u8> { f: 1 };

        // Map: tag + content
        assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "AdjacentlyTagged",
                    len: 2,
                },
                Token::Str("t"),
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Struct",
                },
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

        // Map: content + tag
        assert_de_tokens(
            &value,
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
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Struct",
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn seq() {
        let value = AdjacentlyTagged::Struct::<u8> { f: 1 };

        // Seq: tag + content
        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: Some(2) },
                Token::UnitVariant {
                    name: "AdjacentlyTagged",
                    variant: "Struct",
                },
                Token::Struct {
                    name: "Struct",
                    len: 1,
                },
                Token::Str("f"),
                Token::U8(1),
                Token::StructEnd,
                Token::SeqEnd,
            ],
        );
    }
}

#[test]
fn struct_with_flatten() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(tag = "t", content = "c")]
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
            Token::Struct {
                name: "Data",
                len: 2,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "Data",
                variant: "A",
            },
            Token::Str("c"),
            Token::Map { len: None },
            Token::Str("a"),
            Token::I32(0),
            Token::Str("b"),
            Token::I32(0),
            Token::MapEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn expecting_message() {
    #[derive(Deserialize)]
    #[serde(tag = "tag", content = "content")]
    #[serde(expecting = "something strange...")]
    enum Enum {
        AdjacentlyTagged,
    }

    assert_de_tokens_error::<Enum>(
        &[Token::Str("AdjacentlyTagged")],
        r#"invalid type: string "AdjacentlyTagged", expected something strange..."#,
    );

    assert_de_tokens_error::<Enum>(
        &[Token::Map { len: None }, Token::Unit],
        r#"invalid type: unit value, expected "tag", "content", or other ignored fields"#,
    );

    // Check that #[serde(expecting = "...")] doesn't affect variant identifier error message
    assert_de_tokens_error::<Enum>(
        &[Token::Map { len: None }, Token::Str("tag"), Token::Unit],
        "invalid type: unit value, expected variant of enum Enum",
    );
}

#[test]
fn partially_untagged() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(tag = "t", content = "c")]
    enum Data {
        A(u32),
        B,
        #[serde(untagged)]
        Var(u32),
    }

    let data = Data::A(7);

    assert_de_tokens(
        &data,
        &[
            Token::Map { len: None },
            Token::Str("t"),
            Token::Str("A"),
            Token::Str("c"),
            Token::U32(7),
            Token::MapEnd,
        ],
    );

    let data = Data::Var(42);

    assert_de_tokens(&data, &[Token::U32(42)]);

    // TODO test error output
}

#[test]
fn deny_unknown_fields() {
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
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
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
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
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

    assert_de_tokens_error::<AdjacentlyTagged>(
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::U64(0), // tag field
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
            Token::U64(3),
        ],
        r#"invalid value: integer `3`, expected "t" or "c""#,
    );

    assert_de_tokens_error::<AdjacentlyTagged>(
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Bytes(b"c"),
            Token::Unit,
            Token::Bytes(b"h"),
        ],
        r#"invalid value: byte array, expected "t" or "c""#,
    );
}
