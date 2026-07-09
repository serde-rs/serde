use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_de_tokens, assert_ser_tokens, assert_tokens, Token};
use std::rc::UniqueRc;

#[test]
fn test_raw_identifiers() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[allow(non_camel_case_types)]
    enum r#type {
        r#type { r#type: () },
    }

    assert_tokens(
        &r#type::r#type { r#type: () },
        &[
            Token::StructVariant {
                name: "type",
                variant: "type",
                len: 1,
            },
            Token::Str("type"),
            Token::Unit,
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_unique_rc_ser() {
    assert_ser_tokens(&UniqueRc::new(true), &[Token::Bool(true)]);
}

#[test]
fn test_unique_rc_de() {
    assert_de_tokens(&UniqueRc::new(true), &[Token::Bool(true)]);
}
