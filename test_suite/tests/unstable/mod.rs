// Copyright 2018 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
