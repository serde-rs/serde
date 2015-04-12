#![feature(custom_derive, plugin, test)]
#![plugin(serde_macros)]

extern crate test;
extern crate serde;

use serde::Serialize;
use serde::bytes::{ByteBuf, Bytes};
use serde::json;

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq)]
struct Error;

impl serde::de::Error for Error {
    fn syntax_error() -> Error { Error }

    fn end_of_stream_error() -> Error { Error }

    fn unknown_field_error(_field: &str) -> Error { Error }

    fn missing_field_error(_field: &'static str) -> Error { Error }
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

impl serde::Serializer for BytesSerializer {
    type Error = Error;

    fn visit_unit(&mut self) -> Result<(), Error> {
        Err(Error)
    }

    fn visit_bool(&mut self, _v: bool) -> Result<(), Error> {
        Err(Error)
    }

    fn visit_i64(&mut self, _v: i64) -> Result<(), Error> {
        Err(Error)
    }

    fn visit_u64(&mut self, _v: u64) -> Result<(), Error> {
        Err(Error)
    }

    fn visit_f32(&mut self, _v: f32) -> Result<(), Error> {
        Err(Error)
    }

    fn visit_f64(&mut self, _v: f64) -> Result<(), Error> {
        Err(Error)
    }

    fn visit_char(&mut self, _v: char) -> Result<(), Error> {
        Err(Error)
    }

    fn visit_str(&mut self, _v: &str) -> Result<(), Error> {
        Err(Error)
    }

    fn visit_none(&mut self) -> Result<(), Error> {
        Err(Error)
    }

    fn visit_some<V>(&mut self, _value: V) -> Result<(), Error>
        where V: serde::Serialize,
    {
        Err(Error)
    }

    fn visit_seq<V>(&mut self, _visitor: V) -> Result<(), Error>
        where V: serde::ser::SeqVisitor,
    {
        Err(Error)
    }

    fn visit_seq_elt<T>(&mut self, _value: T) -> Result<(), Error>
        where T: serde::Serialize
    {
        Err(Error)
    }

    fn visit_map<V>(&mut self, _visitor: V) -> Result<(), Error>
        where V: serde::ser::MapVisitor,
    {
        Err(Error)
    }

    fn visit_map_elt<K, V>(&mut self, _key: K, _value: V) -> Result<(), Error>
        where K: serde::Serialize,
              V: serde::Serialize,
    {
        Err(Error)
    }

    fn visit_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        assert_eq!(self.bytes, bytes);
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////

struct BytesDeserializer {
    bytes: Option<Vec<u8>>,
}

impl BytesDeserializer {
    fn new(bytes: Vec<u8>) -> Self {
        BytesDeserializer {
            bytes: Some(bytes),
        }
    }
}

impl serde::Deserializer for BytesDeserializer {
    type Error = Error;

    fn visit<V>(&mut self, _visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor,
    {
        Err(Error)
    }

    fn visit_bytes<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor,
    {
        visitor.visit_byte_buf(self.bytes.take().unwrap())
    }
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_bytes_ser_json() {
    let buf = vec![];
    let bytes = Bytes::from(&buf);
    assert_eq!(json::to_string(&bytes).unwrap(), "[]".to_string());

    let buf = vec![1, 2, 3];
    let bytes = Bytes::from(&buf);
    assert_eq!(json::to_string(&bytes).unwrap(), "[1,2,3]".to_string());
}

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

#[test]
fn test_byte_buf_ser_json() {
    let bytes = ByteBuf::new();
    assert_eq!(json::to_string(&bytes).unwrap(), "[]".to_string());

    let bytes = ByteBuf::from(vec![1, 2, 3]);
    assert_eq!(json::to_string(&bytes).unwrap(), "[1,2,3]".to_string());
}

#[test]
fn test_byte_buf_ser_bytes() {
    let bytes = ByteBuf::new();
    let mut ser = BytesSerializer::new(vec![]);
    bytes.serialize(&mut ser).unwrap();

    let bytes = ByteBuf::from(vec![1, 2, 3]);
    let mut ser = BytesSerializer::new(vec![1, 2, 3]);
    bytes.serialize(&mut ser).unwrap();
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_byte_buf_de_json() {
    let bytes = ByteBuf::new();
    let v: ByteBuf = json::from_str("[]").unwrap();
    assert_eq!(v, bytes);

    let bytes = ByteBuf::from(vec![1, 2, 3]);
    let v: ByteBuf = json::from_str("[1, 2, 3]").unwrap();
    assert_eq!(v, bytes);
}

#[test]
fn test_byte_buf_de_bytes() {
    let mut de = BytesDeserializer::new(vec![]);
    let bytes = serde::Deserialize::deserialize(&mut de);
    assert_eq!(bytes, Ok(ByteBuf::new()));

    let mut de = BytesDeserializer::new(vec![1, 2, 3]);
    let bytes = serde::Deserialize::deserialize(&mut de);
    assert_eq!(bytes, Ok(ByteBuf::from(vec![1, 2, 3])));
}
