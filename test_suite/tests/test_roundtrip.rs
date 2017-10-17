extern crate serde_test;
use self::serde_test::{Token, assert_tokens_readable};

use std::net;

#[macro_use]
#[allow(unused_macros)]
mod macros;

#[test]
fn ip_addr_roundtrip() {

    assert_tokens_readable(
        &net::IpAddr::from(*b"1234"),
        &seq![
            Token::NewtypeVariant { name: "IpAddr", variant: "V4" },

            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,
        ],
        Some(false),
    );
}

#[test]
fn socked_addr_roundtrip() {

    assert_tokens_readable(
        &net::SocketAddr::from((*b"1234567890123456", 1234)),
        &seq![
            Token::NewtypeVariant { name: "SocketAddr", variant: "V6" },

            Token::Tuple { len: 2 },

            Token::Tuple { len: 16 },
            seq b"1234567890123456".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd,
        ],
        Some(false),
    );
}
