use std::fmt;
use std::error;

extern crate serde;
use self::serde::Serialize;
use self::serde::bytes::{ByteBuf, Bytes};

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq)]
struct Error;

impl serde::ser::Error for Error {
    fn custom<T: Into<String>>(_: T) -> Error { Error }
}

impl serde::de::Error for Error {
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

struct SeqSerializer;

impl serde::ser::SeqSerializer for SeqSerializer {
    type Error = Error;

    fn serialize_elt<S: ?Sized, T>(&mut self, _serializer: &mut S, _value: T) -> Result<(), Self::Error>
        where T: Serialize, S: serde::ser::Serializer<Error = Error> {
        Err(Error)
    }

    fn drop<S: ?Sized>(self, _serializer: &mut S) -> Result<(), Self::Error> where S: serde::ser::Serializer<Error = Error> {
        Err(Error)
    }
}

struct MapSerializer;

impl serde::ser::MapSerializer for MapSerializer {
    type Error = Error;

    fn serialize_elt<S: ?Sized, K, V>(&mut self, _serializer: &mut S, _key: K, _value: V) -> Result<(), Self::Error>
        where K: Serialize,
              V: Serialize,
              S: serde::ser::Serializer<Error = Error> {
        Err(Error)
    }

    fn drop<S: ?Sized>(self, _serializer: &mut S) -> Result<(), Self::Error> where S: serde::ser::Serializer<Error = Error> {
        Err(Error)
    }
}

impl serde::Serializer for BytesSerializer {
    type Error = Error;
    type SeqSerializer = SeqSerializer;
    type MapSerializer = MapSerializer;

    fn serialize_unit(&mut self) -> Result<(), Error> {
        Err(Error)
    }

    fn serialize_bool(&mut self, _v: bool) -> Result<(), Error> {
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
        where V: serde::Serialize,
    {
        Err(Error)
    }

    fn serialize_seq<'a>(&'a mut self, _len: Option<usize>) -> Result<serde::ser::SeqHelper<'a, Self>, Error>
    {
        Err(Error)
    }

    fn serialize_seq_elt<T>(&mut self, _value: T) -> Result<(), Error>
        where T: serde::Serialize
    {
        Err(Error)
    }

    fn serialize_seq_end(&mut self) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_map<'a>(&mut self, _: Option<usize>) -> Result<serde::ser::MapHelper<'a, Self>, Error>
    {
        Err(Error)
    }

    fn serialize_map_elt<K, V>(&mut self, _key: K, _value: V) -> Result<(), Error>
        where K: serde::Serialize,
              V: serde::Serialize,
    {
        Err(Error)
    }

    fn serialize_map_end(&mut self) -> Result<(), Error>
    {
        Err(Error)
    }

    fn serialize_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
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

    fn deserialize<V>(&mut self, _visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor,
    {
        Err(Error)
    }

    fn deserialize_bytes<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor,
    {
        visitor.visit_byte_buf(self.bytes.take().unwrap())
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
fn test_byte_buf_de_bytes() {
    let mut de = BytesDeserializer::new(vec![]);
    let bytes = serde::Deserialize::deserialize(&mut de);
    assert_eq!(bytes, Ok(ByteBuf::new()));

    let mut de = BytesDeserializer::new(vec![1, 2, 3]);
    let bytes = serde::Deserialize::deserialize(&mut de);
    assert_eq!(bytes, Ok(ByteBuf::from(vec![1, 2, 3])));
}
