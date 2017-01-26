use serde::bytes::{ByteBuf, Bytes};
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
    assert_de_tokens(&empty, &[Token::ByteBuf(Vec::new())]);
    assert_de_tokens(&empty, &[Token::Str("")]);
    assert_de_tokens(&empty, &[Token::String(String::new())]);
    assert_de_tokens(&empty, &[
        Token::SeqStart(None),
        Token::SeqEnd,
    ]);
    assert_de_tokens(&empty, &[
        Token::SeqStart(Some(0)),
        Token::SeqEnd,
    ]);

    let buf = ByteBuf::from(vec![65, 66, 67]);
    assert_tokens(&buf, &[Token::Bytes(b"ABC")]);
    assert_de_tokens(&buf, &[Token::ByteBuf(vec![65, 66, 67])]);
    assert_de_tokens(&buf, &[Token::Str("ABC")]);
    assert_de_tokens(&buf, &[Token::String("ABC".to_owned())]);
    assert_de_tokens(&buf, &[
        Token::SeqStart(None),
        Token::SeqSep,
        Token::U8(65),
        Token::SeqSep,
        Token::U8(66),
        Token::SeqSep,
        Token::U8(67),
        Token::SeqEnd,
    ]);
    assert_de_tokens(&buf, &[
        Token::SeqStart(Some(3)),
        Token::SeqSep,
        Token::U8(65),
        Token::SeqSep,
        Token::U8(66),
        Token::SeqSep,
        Token::U8(67),
        Token::SeqEnd,
    ]);
}
