//! This module supports serializing into primitive values through the `ValueSerializer` trait

use ::ser;
use ::core::fmt;

#[cfg(feature = "std")]
use std::{error, io};
#[cfg(not(feature = "std"))]
use error;

#[cfg(all(not(feature = "std"), feature = "collections"))]
use collections::string::{String, ToString};
#[cfg(all(not(feature = "std"), feature = "collections"))]
use collections::vec::Vec;

/// This trait converts `Serialize` types into a primitive type
pub trait ValueSerializer<E: ser::Error>: Sized {
    /// Convert this value into a `Serializer`
    fn serialize_from<S: ser::Serialize + ?Sized>(value: &S) -> Result<Self, E>;
}

#[derive(Debug)]
enum Error {
    NotUtf8,
    Aggregate,
    #[cfg(any(feature = "std", feature = "collections"))]
    Custom(String),
    #[cfg(all(not(feature = "std"), not(feature = "collections")))]
    Custom(&'static str),
    #[cfg(feature = "std")]
    Io(io::Error)
}

impl ser::Error for Error {
    #[cfg(any(feature = "std", feature = "collections"))]
    fn custom<T: Into<String>>(msg: T) -> Self { Error::Custom(msg.into()) }

    #[cfg(all(not(feature = "std"), not(feature = "collections")))]
    fn custom<T: Into<&'static str>>(msg: T) -> Self { Error::Custom(msg.into()) }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::NotUtf8 => write!(formatter, "expected utf8"),
            Error::Aggregate => write!(formatter, "cannot serialize aggregates"),
            Error::Custom(ref s) => write!(formatter, "{}", s),
            #[cfg(feature = "std")]
            Error::Io(ref e) => write!(formatter, "{}", e),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Serde Serialization Error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[cfg(any(feature = "collections", feature = "std"))]
impl ValueSerializer<Error> for String {
    fn serialize_from<S: ser::Serialize + ?Sized>(value: &S) -> Result<Self, Error> {
        S::serialize(value, StringSerializer)
    }
}

#[cfg(any(feature = "collections", feature = "std"))]
struct StringSerializer;

#[cfg(any(feature = "collections", feature = "std"))]
impl ser::Serializer for StringSerializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = Void;
    type SerializeTuple = Void;
    type SerializeTupleStruct = Void;
    type SerializeTupleVariant = Void;
    type SerializeMap = Void;
    type SerializeStruct = Void;
    type SerializeStructVariant = Void;

    fn serialize_bool(self, value: bool) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_isize(self, value: isize) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_usize(self, value: usize) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_i64(self, value: i64) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_i32(self, value: i32) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_i16(self, value: i16) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_i8(self, value: i8) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_u64(self, value: u64) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_u32(self, value: u32) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_u16(self, value: u16) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_u8(self, value: u8) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_f32(self, value: f32) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_f64(self, value: f64) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_char(self, value: char) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_str(self, value: &str) -> Result<String, Error> {
        Ok(value.to_string())
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<String, Error> {
        match ::core::str::from_utf8(value) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(Error::NotUtf8)
        }
    }
    fn serialize_unit(self) -> Result<String, Error> {
        Ok(String::new())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, Error> {
        Ok(String::new())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: usize,
        variant: &'static str
    ) -> Result<String, Error> {
        Ok(variant.to_string())
    }
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: T
    ) -> Result<String, Error>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: usize,
        _variant: &'static str,
        _value: T
    ) -> Result<String, Error> {
        Err(Error::Aggregate)
    }
    fn serialize_none(self) -> Result<String, Error> {
        self.serialize_unit()
    }
    fn serialize_some<V: ser::Serialize>(self, value: V) -> Result<String, Error> {
        value.serialize(self)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Void, Error> {
        Err(Error::Aggregate)
    }
    fn serialize_seq_fixed_size(self, _len: usize) -> Result<Void, Error> {
        Err(Error::Aggregate)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Void, Error> {
        Err(Error::Aggregate)
    }
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Void, Error> {
        Err(Error::Aggregate)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: usize,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Void, Error> {
        Err(Error::Aggregate)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Void, Error> {
        Err(Error::Aggregate)
    }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Void, Error> {
        Err(Error::Aggregate)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: usize,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Void, Error> {
        Err(Error::Aggregate)
    }
}

#[cfg(any(feature = "collections", feature = "std"))]
enum Void {}

#[cfg(any(feature = "collections", feature = "std"))]
impl ser::SerializeSeq for Void {
    type Ok = String;
    type Error = Error;
    fn serialize_element<T: ser::Serialize>(&mut self, _value: T) -> Result<(), Self::Error> {
        match *self {}
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

#[cfg(any(feature = "collections", feature = "std"))]
impl ser::SerializeTuple for Void {
    type Ok = String;
    type Error = Error;
    fn serialize_element<T: ser::Serialize>(&mut self, _value: T) -> Result<(), Self::Error> {
        match *self {}
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

#[cfg(any(feature = "collections", feature = "std"))]
impl ser::SerializeTupleStruct for Void {
    type Ok = String;
    type Error = Error;
    fn serialize_field<T: ser::Serialize>(&mut self, _value: T) -> Result<(), Self::Error> {
        match *self {}
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

#[cfg(any(feature = "collections", feature = "std"))]
impl ser::SerializeTupleVariant for Void {
    type Ok = String;
    type Error = Error;
    fn serialize_field<T: ser::Serialize>(&mut self, _value: T) -> Result<(), Self::Error> {
        match *self {}
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

#[cfg(any(feature = "collections", feature = "std"))]
impl ser::SerializeMap for Void {
    type Ok = String;
    type Error = Error;
    fn serialize_key<T: ser::Serialize>(&mut self, _value: T) -> Result<(), Self::Error> {
        match *self {}
    }
    fn serialize_value<T: ser::Serialize>(&mut self, _value: T) -> Result<(), Self::Error> {
        match *self {}
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

#[cfg(any(feature = "collections", feature = "std"))]
impl ser::SerializeStruct for Void {
    type Ok = String;
    type Error = Error;
    fn serialize_field<T: ser::Serialize>(&mut self, _key: &'static str, _value: T) -> Result<(), Self::Error> {
        match *self {}
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

#[cfg(any(feature = "collections", feature = "std"))]
impl ser::SerializeStructVariant for Void {
    type Ok = String;
    type Error = Error;
    fn serialize_field<T: ser::Serialize>(&mut self, _key: &'static str, _value: T) -> Result<(), Self::Error> {
        match *self {}
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {}
    }
}

#[cfg(any(feature = "collections", feature = "std"))]
impl ser::ByteSerializer for StringSerializer {
    #[cfg(feature = "std")]
    fn to_writer<W, T: ?Sized>(mut writer: W, value: &T) -> Result<(), Error>
        where W: io::Write, T: ser::Serialize {
        let s = try!(String::serialize_from(value));
        writer.write_all(s.as_bytes()).map_err(Error::Io)
    }

    #[cfg(any(feature = "std", feature = "collections"))]
    fn to_string<T: ?Sized>(value: &T) -> Result<String, Self::Error>
        where T: ser::Serialize
    {
        String::serialize_from(value)
    }

    #[cfg(any(feature = "std", feature = "collections"))]
    fn to_bytes<T: ?Sized>(value: &T) -> Result<Vec<u8>, Self::Error>
        where T: ser::Serialize
    {
        Self::to_string(value).map(String::into_bytes)
    }
}
