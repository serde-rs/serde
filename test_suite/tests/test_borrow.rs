#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::items_after_statements,
    clippy::used_underscore_binding
)]

use serde::de::value::{BorrowedStrDeserializer, MapDeserializer};
use serde::de::{Deserialize, Deserializer, IntoDeserializer};
use serde_derive::Deserialize;
use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};

use std::borrow::Cow;

#[test]
fn test_borrowed_str() {
    assert_de_tokens(&"borrowed", &[Token::BorrowedStr("borrowed")]);
}

#[test]
fn test_borrowed_str_from_string() {
    assert_de_tokens_error::<&str>(
        &[Token::String("borrowed")],
        "invalid type: string \"borrowed\", expected a borrowed string",
    );
}

#[test]
fn test_borrowed_str_from_str() {
    assert_de_tokens_error::<&str>(
        &[Token::Str("borrowed")],
        "invalid type: string \"borrowed\", expected a borrowed string",
    );
}

#[test]
fn test_string_from_borrowed_str() {
    assert_de_tokens(&"owned".to_owned(), &[Token::BorrowedStr("owned")]);
}

#[test]
fn test_borrowed_bytes() {
    assert_de_tokens(&&b"borrowed"[..], &[Token::BorrowedBytes(b"borrowed")]);
}

#[test]
fn test_borrowed_bytes_from_bytebuf() {
    assert_de_tokens_error::<&[u8]>(
        &[Token::ByteBuf(b"borrowed")],
        "invalid type: byte array, expected a borrowed byte array",
    );
}

#[test]
fn test_borrowed_bytes_from_bytes() {
    assert_de_tokens_error::<&[u8]>(
        &[Token::Bytes(b"borrowed")],
        "invalid type: byte array, expected a borrowed byte array",
    );
}

#[test]
fn test_tuple() {
    assert_de_tokens(
        &("str", &b"bytes"[..]),
        &[
            Token::Tuple { len: 2 },
            Token::BorrowedStr("str"),
            Token::BorrowedBytes(b"bytes"),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn test_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Borrowing<'a, 'b> {
        bs: &'a str,
        bb: &'b [u8],
    }

    assert_de_tokens(
        &Borrowing {
            bs: "str",
            bb: b"bytes",
        },
        &[
            Token::Struct {
                name: "Borrowing",
                len: 2,
            },
            Token::BorrowedStr("bs"),
            Token::BorrowedStr("str"),
            Token::BorrowedStr("bb"),
            Token::BorrowedBytes(b"bytes"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_field_identifier() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(field_identifier)]
    enum FieldStr<'a> {
        #[serde(borrow)]
        Str(&'a str),
    }

    assert_de_tokens(&FieldStr::Str("value"), &[Token::BorrowedStr("value")]);

    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(field_identifier)]
    enum FieldBytes<'a> {
        #[serde(borrow)]
        Bytes(&'a [u8]),
    }

    assert_de_tokens(
        &FieldBytes::Bytes(b"value"),
        &[Token::BorrowedBytes(b"value")],
    );
}

#[test]
fn test_cow() {
    #[derive(Deserialize)]
    struct Cows<'a, 'b> {
        copied: Cow<'a, str>,

        #[serde(borrow)]
        borrowed: Cow<'b, str>,
    }

    struct BorrowedStr(&'static str);

    impl<'de> IntoDeserializer<'de> for BorrowedStr {
        type Deserializer = BorrowedStrDeserializer<'de, serde::de::value::Error>;

        fn into_deserializer(self) -> Self::Deserializer {
            BorrowedStrDeserializer::new(self.0)
        }
    }

    let de = MapDeserializer::new(IntoIterator::into_iter([
        ("copied", BorrowedStr("copied")),
        ("borrowed", BorrowedStr("borrowed")),
    ]));

    let cows = Cows::deserialize(de).unwrap();

    match cows.copied {
        Cow::Owned(ref s) if s == "copied" => {}
        _ => panic!("expected a copied string"),
    }

    match cows.borrowed {
        Cow::Borrowed("borrowed") => {}
        _ => panic!("expected a borrowed string"),
    }
}

#[test]
fn test_lifetimes() {
    #[derive(Deserialize)]
    struct Cows<'a, 'b> {
        _copied: Cow<'a, str>,

        #[serde(borrow)]
        _borrowed: Cow<'b, str>,
    }

    // Tests that `'de: 'a` is not required by the Deserialize impl.
    fn _cows_lifetimes<'de: 'b, 'a, 'b, D>(deserializer: D) -> Cows<'a, 'b>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).unwrap()
    }

    #[derive(Deserialize)]
    struct Wrap<'a, 'b> {
        #[serde(borrow = "'b")]
        _cows: Cows<'a, 'b>,
    }

    // Tests that `'de: 'a` is not required by the Deserialize impl.
    fn _wrap_lifetimes<'de: 'b, 'a, 'b, D>(deserializer: D) -> Wrap<'a, 'b>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).unwrap()
    }
}
