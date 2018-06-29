// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

extern crate serde_test;
use serde_test::{assert_de_tokens, Token};

#[test]
fn test_variant_identifier() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(variant_identifier)]
    enum V {
        Aaa,
        Bbb,
    }

    assert_de_tokens(&V::Aaa, &[Token::U8(0)]);
    assert_de_tokens(&V::Aaa, &[Token::U16(0)]);
    assert_de_tokens(&V::Aaa, &[Token::U32(0)]);
    assert_de_tokens(&V::Aaa, &[Token::U64(0)]);
    assert_de_tokens(&V::Aaa, &[Token::Str("Aaa")]);
    assert_de_tokens(&V::Aaa, &[Token::Bytes(b"Aaa")]);
}

#[test]
fn test_field_identifier() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(field_identifier, rename_all = "snake_case")]
    enum F {
        Aaa,
        Bbb,
    }

    assert_de_tokens(&F::Aaa, &[Token::Str("aaa")]);
    assert_de_tokens(&F::Aaa, &[Token::Bytes(b"aaa")]);
}

#[test]
fn test_unit_fallthrough() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(field_identifier, rename_all = "snake_case")]
    enum F {
        Aaa,
        Bbb,
        #[serde(other)]
        Other,
    }

    assert_de_tokens(&F::Other, &[Token::Str("x")]);
}

#[test]
fn test_newtype_fallthrough() {
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
fn test_newtype_fallthrough_generic() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(field_identifier, rename_all = "snake_case")]
    enum F<T> {
        Aaa,
        Bbb,
        Other(T),
    }

    assert_de_tokens(&F::Other("x".to_owned()), &[Token::Str("x")]);
}
