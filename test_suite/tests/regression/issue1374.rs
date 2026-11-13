use serde_derive::Deserialize;
use serde_test::{assert_de_tokens, Token};

#[derive(Debug, Deserialize, PartialEq)]
struct Container {
    other: u16,
    #[serde(flatten)]
    inner: DeEnum,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum DeEnum {
    NonEmptyVariant(NonEmpty),
    EmptyVariant,
}

#[derive(Debug, Deserialize, PartialEq)]
struct NonEmpty {
    key: u16,
}

#[test]
fn flattened_untagged_unit_variant_matches_empty_map() {
    assert_de_tokens(
        &Container {
            other: 12,
            inner: DeEnum::EmptyVariant,
        },
        &[
            Token::Map { len: None },
            Token::Str("other"),
            Token::U16(12),
            Token::MapEnd,
        ],
    );
}
