use std::fmt;
use std::error;

use serde::{Serialize, Serializer};
use serde::bytes::{ByteBuf, Bytes};
use serde::ser;
use serde::de;

use serde_test::{assert_de_tokens, Token};

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq)]
struct Error;

impl ser::Error for Error {
    fn custom<T: Into<String>>(_: T) -> Error { Error }
}

impl de::Error for Error {
    fn custom<T: Into<String>>(_: T) -> Error { Error }

    fn end_of_stream() -> Error { Error }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str(format!("{:?}", self).as_ref())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Serde Deserialization Error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

///////////////////////////////////////////////////////////////////////////////

struct BytesSerializer {
    bytes: Vec<u8>,
}

impl BytesSerializer {
    fn new(bytes: Vec<u8>) -> Self {
        BytesSerializer {
            bytes: bytes,
        }
    }
}

impl Serializer for BytesSerializer {
    type Error = Error;
    type SeqState = ();
    type MapState = ();
    type TupleState = ();
    type TupleStructState = ();
    type TupleVariantState = ();
    type StructState = ();
    type StructVariantState = ();

    fn serialize_unit(&mut self) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_unit_struct(&mut self, _name: &'static str) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_unit_variant(&mut self, _: &'static str, _: usize, _: &'static str) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_bool(&mut self, _v: bool) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_isize(&mut self, _v: isize) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_usize(&mut self, _v: usize) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_i8(&mut self, _v: i8) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_u8(&mut self, _v: u8) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_i16(&mut self, _v: i16) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_u16(&mut self, _v: u16) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_i32(&mut self, _v: i32) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_u32(&mut self, _v: u32) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_i64(&mut self, _v: i64) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_u64(&mut self, _v: u64) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_f32(&mut self, _v: f32) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_f64(&mut self, _v: f64) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_char(&mut self, _v: char) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_str(&mut self, _v: &str) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_none(&mut self) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_some<V>(&mut self, _value: V) -> Result<(), Error>
        where V: Serialize,
    {
        Err(Error)
    }

    fn serialize_newtype_struct<V>(&mut self, _: &'static str, _value: V) -> Result<(), Error>
        where V: Serialize,
    {
        Err(Error)
    }

    fn serialize_newtype_variant<V>(&mut self, _: &'static str, _: usize, _: &'static str, _value: V) -> Result<(), Error>
        where V: Serialize,
    {
        Err(Error)
    }

    fn serialize_seq(&mut self, _len: Option<usize>) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_seq_fixed_size(&mut self, _len: usize) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_seq_elt<T>(&mut self, _: &mut (), _value: T) -> Result<(), Error>
        where T: Serialize
    {
        Err(Error)
    }

    fn serialize_seq_end(&mut self, _: ()) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_tuple(&mut self, _len: usize) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_tuple_elt<T>(&mut self, _: &mut (), _value: T) -> Result<(), Error>
        where T: Serialize
    {
        Err(Error)
    }

    fn serialize_tuple_end(&mut self, _: ()) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_tuple_struct(&mut self, _: &'static str, _len: usize) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_tuple_struct_elt<T>(&mut self, _: &mut (), _value: T) -> Result<(), Error>
        where T: Serialize
    {
        Err(Error)
    }

    fn serialize_tuple_struct_end(&mut self, _: ()) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_tuple_variant(&mut self, _: &'static str, _: usize, _: &'static str, _len: usize) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_tuple_variant_elt<T>(&mut self, _: &mut (), _value: T) -> Result<(), Error>
        where T: Serialize
    {
        Err(Error)
    }

    fn serialize_tuple_variant_end(&mut self, _: ()) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_map(&mut self, _: Option<usize>) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_map_key<T>(&mut self, _: &mut (), _key: T) -> Result<(), Error>
        where T: Serialize
    {
        Err(Error)
    }

    fn serialize_map_value<T>(&mut self, _: &mut (), _value: T) -> Result<(), Error>
        where T: Serialize
    {
        Err(Error)
    }

    fn serialize_map_end(&mut self, _: ()) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_struct(&mut self, _: &'static str, _: usize) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_struct_elt<V>(&mut self, _: &mut (), _key: &'static str, _value: V) -> Result<(), Error>
        where V: Serialize,
    {
        Err(Error)
    }

    fn serialize_struct_end(&mut self, _: ()) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_struct_variant(&mut self, _: &'static str, _: usize, _: &'static str, _: usize) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_struct_variant_elt<V>(&mut self, _: &mut (), _key: &'static str, _value: V) -> Result<(), Error>
        where V: Serialize,
    {
        Err(Error)
    }

    fn serialize_struct_variant_end(&mut self, _: ()) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        assert_eq!(self.bytes, bytes);
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_bytes_ser_bytes() {
    let buf = vec![];
    let bytes = Bytes::from(&buf);
    let mut ser = BytesSerializer::new(vec![]);
    bytes.serialize(&mut ser).unwrap();

    let buf = vec![1, 2, 3];
    let bytes = Bytes::from(&buf);
    let mut ser = BytesSerializer::new(vec![1, 2, 3]);
    bytes.serialize(&mut ser).unwrap();
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_byte_buf_de() {
    let empty = ByteBuf::new();
    assert_de_tokens(&empty, &[Token::Bytes(b""),]);
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
    assert_de_tokens(&buf, &[Token::Bytes(b"ABC")]);
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
