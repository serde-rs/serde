use serde_test::{assert_tokens, Configure, Token};
use std::net;

#[macro_use]
#[allow(unused_macros)]
mod macros;

#[test]
fn ip_addr_roundtrip() {
    assert_tokens(
        &net::IpAddr::from(*b"1234").compact(),
        &seq![
            Token::NewtypeVariant {
                name: "IpAddr",
                variant: "V4"
            },
            Token::Tuple { len: 4 },
            b"1234".iter().copied().map(Token::U8),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn socket_addr_roundtrip() {
    assert_tokens(
        &net::SocketAddr::from((*b"1234567890123456", 1234)).compact(),
        &seq![
            Token::NewtypeVariant {
                name: "SocketAddr",
                variant: "V6"
            },
            Token::Tuple { len: 2 },
            Token::Tuple { len: 16 },
            b"1234567890123456".iter().copied().map(Token::U8),
            Token::TupleEnd,
            Token::U16(1234),
            Token::TupleEnd,
        ],
    );
}
