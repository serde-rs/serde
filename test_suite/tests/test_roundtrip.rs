// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate serde_test;
use self::serde_test::{assert_tokens, Configure, Token};

use std::net;

#[macro_use]
#[allow(unused_macros)]
mod macros;

#[test]
fn ip_addr_roundtrip() {
    assert_tokens(
        &net::IpAddr::from(*b"1234").compact(),
        &seq![
            Token::NewtypeVariant { name: "IpAddr", variant: "V4" },

            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn socket_addr_roundtrip() {
    assert_tokens(
        &net::SocketAddr::from((*b"1234567890123456", 1234)).compact(),
        &seq![
            Token::NewtypeVariant { name: "SocketAddr", variant: "V6" },

            Token::Tuple { len: 2 },

            Token::Tuple { len: 16 },
            seq b"1234567890123456".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd,
        ],
    );
}
