use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_tokens, Token};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Enum {
    Simple {
        a: i32,
    },
    Flatten {
        #[serde(flatten)]
        flatten: (),
        a: i32,
    },
}

#[test]
fn simple_variant() {
    assert_tokens(
        &Enum::Simple { a: 42 },
        &[
            Token::StructVariant {
                name: "Enum",
                variant: "Simple",
                len: 1,
            },
            Token::Str("a"),
            Token::I32(42),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn flatten_variant() {
    assert_tokens(
        &Enum::Flatten { flatten: (), a: 42 },
        &[
            Token::NewtypeVariant {
                name: "Enum",
                variant: "Flatten",
            },
            Token::Map { len: None },
            Token::Str("a"),
            Token::I32(42),
            Token::MapEnd,
        ],
    );
}
