//! Tests for `#[serde(field_identifier)]` and `#[serde(variant_identifier)]`

#![allow(clippy::derive_partial_eq_without_eq)]

use serde_derive::Deserialize;
use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};

mod variant_identifier {
    use super::*;

    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(variant_identifier)]
    enum V {
        Aaa,
        #[serde(alias = "Ccc", alias = "Ddd")]
        Bbb,
    }

    #[test]
    fn variant1() {
        assert_de_tokens(&V::Aaa, &[Token::U8(0)]);
        assert_de_tokens(&V::Aaa, &[Token::U16(0)]);
        assert_de_tokens(&V::Aaa, &[Token::U32(0)]);
        assert_de_tokens(&V::Aaa, &[Token::U64(0)]);
        assert_de_tokens(&V::Aaa, &[Token::Str("Aaa")]);
        assert_de_tokens(&V::Aaa, &[Token::Bytes(b"Aaa")]);
    }

    #[test]
    fn aliases() {
        assert_de_tokens(&V::Bbb, &[Token::U8(1)]);
        assert_de_tokens(&V::Bbb, &[Token::U16(1)]);
        assert_de_tokens(&V::Bbb, &[Token::U32(1)]);
        assert_de_tokens(&V::Bbb, &[Token::U64(1)]);

        assert_de_tokens(&V::Bbb, &[Token::Str("Bbb")]);
        assert_de_tokens(&V::Bbb, &[Token::Bytes(b"Bbb")]);

        assert_de_tokens(&V::Bbb, &[Token::Str("Ccc")]);
        assert_de_tokens(&V::Bbb, &[Token::Bytes(b"Ccc")]);

        assert_de_tokens(&V::Bbb, &[Token::Str("Ddd")]);
        assert_de_tokens(&V::Bbb, &[Token::Bytes(b"Ddd")]);
    }

    #[test]
    fn unknown() {
        assert_de_tokens_error::<V>(
            &[Token::U8(42)],
            "invalid value: integer `42`, expected variant index 0 <= i < 2",
        );
        assert_de_tokens_error::<V>(
            &[Token::U16(42)],
            "invalid value: integer `42`, expected variant index 0 <= i < 2",
        );
        assert_de_tokens_error::<V>(
            &[Token::U32(42)],
            "invalid value: integer `42`, expected variant index 0 <= i < 2",
        );
        assert_de_tokens_error::<V>(
            &[Token::U64(42)],
            "invalid value: integer `42`, expected variant index 0 <= i < 2",
        );
        assert_de_tokens_error::<V>(
            &[Token::Str("Unknown")],
            "unknown variant `Unknown`, expected one of `Aaa`, `Bbb`, `Ccc`, `Ddd`",
        );
        assert_de_tokens_error::<V>(
            &[Token::Bytes(b"Unknown")],
            "unknown variant `Unknown`, expected one of `Aaa`, `Bbb`, `Ccc`, `Ddd`",
        );
    }
}

mod field_identifier {
    use super::*;

    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(field_identifier, rename_all = "snake_case")]
    enum F {
        Aaa,
        #[serde(alias = "ccc", alias = "ddd")]
        Bbb,
    }

    #[test]
    fn field1() {
        assert_de_tokens(&F::Aaa, &[Token::U8(0)]);
        assert_de_tokens(&F::Aaa, &[Token::U16(0)]);
        assert_de_tokens(&F::Aaa, &[Token::U32(0)]);
        assert_de_tokens(&F::Aaa, &[Token::U64(0)]);
        assert_de_tokens(&F::Aaa, &[Token::Str("aaa")]);
        assert_de_tokens(&F::Aaa, &[Token::Bytes(b"aaa")]);
    }

    #[test]
    fn aliases() {
        assert_de_tokens(&F::Bbb, &[Token::U8(1)]);
        assert_de_tokens(&F::Bbb, &[Token::U16(1)]);
        assert_de_tokens(&F::Bbb, &[Token::U32(1)]);
        assert_de_tokens(&F::Bbb, &[Token::U64(1)]);

        assert_de_tokens(&F::Bbb, &[Token::Str("bbb")]);
        assert_de_tokens(&F::Bbb, &[Token::Bytes(b"bbb")]);

        assert_de_tokens(&F::Bbb, &[Token::Str("ccc")]);
        assert_de_tokens(&F::Bbb, &[Token::Bytes(b"ccc")]);

        assert_de_tokens(&F::Bbb, &[Token::Str("ddd")]);
        assert_de_tokens(&F::Bbb, &[Token::Bytes(b"ddd")]);
    }

    #[test]
    fn unknown() {
        assert_de_tokens_error::<F>(
            &[Token::U8(42)],
            "invalid value: integer `42`, expected field index 0 <= i < 2",
        );
        assert_de_tokens_error::<F>(
            &[Token::U16(42)],
            "invalid value: integer `42`, expected field index 0 <= i < 2",
        );
        assert_de_tokens_error::<F>(
            &[Token::U32(42)],
            "invalid value: integer `42`, expected field index 0 <= i < 2",
        );
        assert_de_tokens_error::<F>(
            &[Token::U64(42)],
            "invalid value: integer `42`, expected field index 0 <= i < 2",
        );
        assert_de_tokens_error::<F>(
            &[Token::Str("unknown")],
            "unknown field `unknown`, expected one of `aaa`, `bbb`, `ccc`, `ddd`",
        );
        assert_de_tokens_error::<F>(
            &[Token::Bytes(b"unknown")],
            "unknown field `unknown`, expected one of `aaa`, `bbb`, `ccc`, `ddd`",
        );
    }

    #[test]
    fn unit_fallthrough() {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum F {
            Aaa,
            Bbb,
            #[serde(other)]
            Other,
        }

        assert_de_tokens(&F::Other, &[Token::U8(42)]);
        assert_de_tokens(&F::Other, &[Token::U16(42)]);
        assert_de_tokens(&F::Other, &[Token::U32(42)]);
        assert_de_tokens(&F::Other, &[Token::U64(42)]);
        assert_de_tokens(&F::Other, &[Token::Str("x")]);
    }

    #[test]
    fn newtype_fallthrough() {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum F {
            Aaa,
            Bbb,
            Other(String),
        }

        assert_de_tokens(&F::Other("x".to_owned()), &[Token::Str("x")]);
    }

    #[test]
    fn newtype_fallthrough_generic() {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum F<T> {
            Aaa,
            Bbb,
            Other(T),
        }

        assert_de_tokens(&F::Other(42u8), &[Token::U8(42)]);
        assert_de_tokens(&F::Other(42u16), &[Token::U16(42)]);
        assert_de_tokens(&F::Other(42u32), &[Token::U32(42)]);
        assert_de_tokens(&F::Other(42u64), &[Token::U64(42)]);
        assert_de_tokens(&F::Other("x".to_owned()), &[Token::Str("x")]);
    }
}
