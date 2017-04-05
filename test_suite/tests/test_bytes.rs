extern crate serde;
use serde::bytes::{ByteBuf, Bytes};

extern crate serde_test;
use serde_test::{assert_tokens, assert_ser_tokens, assert_de_tokens, Token};

#[test]
fn test_bytes() {
    let empty = Bytes::new(&[]);
    assert_ser_tokens(&empty, &[Token::Bytes(b"")]);

    let buf = vec![65, 66, 67];
    let bytes = Bytes::new(&buf);
    assert_ser_tokens(&bytes, &[Token::Bytes(b"ABC")]);
}

#[test]
fn test_byte_buf() {
    let empty = ByteBuf::new();
    assert_tokens(&empty, &[Token::Bytes(b"")]);
    assert_de_tokens(&empty, &[Token::ByteBuf(b"")]);
    assert_de_tokens(&empty, &[Token::Str("")]);
    assert_de_tokens(&empty, &[Token::String("")]);
    assert_de_tokens(&empty, &[
        Token::Seq(None),
        Token::SeqEnd,
    ]);
    assert_de_tokens(&empty, &[
        Token::Seq(Some(0)),
        Token::SeqEnd,
    ]);

    let buf = ByteBuf::from(vec![65, 66, 67]);
    assert_tokens(&buf, &[Token::Bytes(b"ABC")]);
    assert_de_tokens(&buf, &[Token::ByteBuf(b"ABC")]);
    assert_de_tokens(&buf, &[Token::Str("ABC")]);
    assert_de_tokens(&buf, &[Token::String("ABC")]);
    assert_de_tokens(&buf, &[
        Token::Seq(None),
        Token::U8(65),
        Token::U8(66),
        Token::U8(67),
        Token::SeqEnd,
    ]);
    assert_de_tokens(&buf, &[
        Token::Seq(Some(3)),
        Token::U8(65),
        Token::U8(66),
        Token::U8(67),
        Token::SeqEnd,
    ]);
}
