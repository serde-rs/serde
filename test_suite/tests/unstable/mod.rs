use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_tokens, Token};

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
