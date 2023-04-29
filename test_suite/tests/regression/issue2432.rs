//! This tests tries to deserialize `Struct` from the following JSON representations:
//!
//! - From mapping
//!   ```json
//!   {
//!       "number": 1234,
//!       "text": "hello",
//!       "dont": 345.234,
//!       "really": [],
//!       "care": true
//!   }
//!   ```
//! - From sequence:
//!   ```json
//!   [
//!       1234,
//!       "hello",
//!       345.234,
//!       [],
//!       true
//!   ]
//!   ```
use serde::Deserialize;
use serde_test::{Token, assert_de_tokens, assert_de_tokens_error};

mod no_deny_unknown_fields {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Struct {
        number: i32,
        text: String,
    }

    #[test]
    fn struct_() {
        let value = Struct {
            number: 1234,
            text: "hello".to_string(),
        };

        assert_de_tokens(
            &value,
            &[
                Token::Struct { name: "Struct", len: 5 },
                Token::Str("number"),
                Token::I32(1234),
                Token::Str("text"),
                Token::Str("hello"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &value,
            &[
                Token::Struct { name: "Struct", len: 5 },
                Token::Str("number"),
                Token::I32(1234),
                Token::Str("text"),
                Token::Str("hello"),
                Token::Str("dont"),
                Token::F32(345.234),
                Token::Str("really"),
                Token::Seq { len: None },
                Token::SeqEnd,
                Token::Str("care"),
                Token::Bool(true),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn map() {
        let value = Struct {
            number: 1234,
            text: "hello".to_string(),
        };

        assert_de_tokens(
            &value,
            &[
                Token::Map { len: None },
                Token::Str("number"),
                Token::I32(1234),
                Token::Str("text"),
                Token::Str("hello"),
                Token::MapEnd,
            ],
        );

        assert_de_tokens(
            &value,
            &[
                Token::Map { len: None },
                Token::Str("number"),
                Token::I32(1234),
                Token::Str("text"),
                Token::Str("hello"),
                Token::Str("dont"),
                Token::F32(345.234),
                Token::Str("really"),
                Token::Seq { len: None },
                Token::SeqEnd,
                Token::Str("care"),
                Token::Bool(true),
                Token::MapEnd,
            ],
        );
    }

//------------------------------------------------------------------------------

    #[test]
    fn seq() {
        let value = Struct {
            number: 1234,
            text: "hello".to_string(),
        };

        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: None },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::SeqEnd,
            ],
        );

        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: None },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::F32(345.234),        // dont
                Token::Seq { len: None },   // really
                Token::SeqEnd,
                Token::Bool(true),          // care
                Token::SeqEnd,
            ],
        );
    }

    #[test]
    fn tuple() {
        let value = Struct {
            number: 1234,
            text: "hello".to_string(),
        };

        assert_de_tokens(
            &value,
            &[
                Token::Tuple { len: 2 },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::TupleEnd,
            ],
        );

        assert_de_tokens(
            &value,
            &[
                Token::Tuple { len: 5 },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::F32(345.234),        // dont
                Token::Seq { len: None },   // really
                Token::SeqEnd,
                Token::Bool(true),          // care
                Token::TupleEnd,
            ],
        );
    }

    #[test]
    fn tuple_struct() {
        let value = Struct {
            number: 1234,
            text: "hello".to_string(),
        };

        assert_de_tokens(
            &value,
            &[
                Token::TupleStruct { name: "DoNotMatter", len: 2 },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::TupleStructEnd,
            ],
        );

        assert_de_tokens(
            &value,
            &[
                Token::TupleStruct { name: "DoNotMatter", len: 5 },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::F32(345.234),        // dont
                Token::Seq { len: None },   // really
                Token::SeqEnd,
                Token::Bool(true),          // care
                Token::TupleStructEnd,
            ],
        );
    }
}

mod deny_unknown_fields {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Struct {
        number: i32,
        text: String,
    }

    #[test]
    fn struct_() {
        assert_de_tokens(
            &Struct {
                number: 1234,
                text: "hello".to_string(),
            },
            &[
                Token::Struct { name: "Struct", len: 2 },
                Token::Str("number"),
                Token::I32(1234),
                Token::Str("text"),
                Token::Str("hello"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens_error::<Struct>(
            &[
                Token::Struct { name: "Struct", len: 5 },
                Token::Str("number"),
                Token::I32(1234),
                Token::Str("text"),
                Token::Str("hello"),
                Token::Str("dont"),
                Token::F32(345.234),
                // Tokens that could follow, but assert_de_tokens_error do not want them
                // Token::Str("really"),
                // Token::Seq { len: None },
                // Token::SeqEnd,
                // Token::Str("care"),
                // Token::Bool(true),
                // Token::StructEnd,
            ],
            "unknown field `dont`, expected `number` or `text`",
        );
    }

    #[test]
    fn map() {
        assert_de_tokens(
            &Struct {
                number: 1234,
                text: "hello".to_string(),
            },
            &[
                Token::Map { len: None },
                Token::Str("number"),
                Token::I32(1234),
                Token::Str("text"),
                Token::Str("hello"),
                Token::MapEnd,
            ],
        );

        assert_de_tokens_error::<Struct>(
            &[
                Token::Map { len: None },
                Token::Str("number"),
                Token::I32(1234),
                Token::Str("text"),
                Token::Str("hello"),
                Token::Str("dont"),
                Token::F32(345.234),
                // Tokens that could follow, but assert_de_tokens_error do not want them
                // Token::Str("really"),
                // Token::Seq { len: None },
                // Token::SeqEnd,
                // Token::Str("care"),
                // Token::Bool(true),
                // Token::MapEnd,
            ],
            "unknown field `dont`, expected `number` or `text`",
        );
    }

//------------------------------------------------------------------------------

    #[test]
    fn seq() {
        assert_de_tokens(
            &Struct {
                number: 1234,
                text: "hello".to_string(),
            },
            &[
                Token::Seq { len: None },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::SeqEnd,
            ],
        );

        assert_de_tokens_error::<Struct>(
            &[
                Token::Seq { len: None },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::F32(345.234),        // dont
                // Tokens that could follow, but assert_de_tokens_error do not want them
                // Token::Seq { len: None },   // really
                // Token::SeqEnd,
                // Token::Bool(true),          // care
                // Token::SeqEnd,
            ],
            "invalid length 2, expected struct Struct with 2 elements",
        );
    }

    #[test]
    fn tuple() {
        assert_de_tokens(
            &Struct {
                number: 1234,
                text: "hello".to_string(),
            },
            &[
                Token::Tuple { len: 2 },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::TupleEnd,
            ],
        );

        assert_de_tokens_error::<Struct>(
            &[
                Token::Tuple { len: 5 },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::F32(345.234),        // dont
                // Tokens that could follow, but assert_de_tokens_error do not want them
                // Token::Seq { len: None },   // really
                // Token::SeqEnd,
                // Token::Bool(true),          // care
                // Token::TupleEnd,
            ],
            "invalid length 2, expected struct Struct with 2 elements",
        );
    }

    #[test]
    fn tuple_struct() {
        assert_de_tokens(
            &Struct {
                number: 1234,
                text: "hello".to_string(),
            },
            &[
                Token::TupleStruct { name: "DoNotMatter", len: 2 },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::TupleStructEnd,
            ],
        );

        assert_de_tokens_error::<Struct>(
            &[
                Token::TupleStruct { name: "DoNotMatter", len: 5 },
                Token::I32(1234),           // number
                Token::Str("hello"),        // text
                Token::F32(345.234),        // dont
                // Tokens that could follow, but assert_de_tokens_error do not want them
                // Token::Seq { len: None },   // really
                // Token::SeqEnd,
                // Token::Bool(true),          // care
                // Token::TupleStructEnd,
            ],
            "invalid length 2, expected struct Struct with 2 elements",
        );
    }
}
