use serde::{ser, Serialize};

use error::Error;
use token::Token;

/// A `Serializer` that ensures that a value serializes to a given list of tokens.
pub struct Serializer<'a> {
    tokens: &'a [Token],
}

impl<'a> Serializer<'a> {
    /// Creates the serializer.
    pub fn new(tokens: &'a [Token]) -> Self {
        Serializer {
            tokens: tokens,
        }
    }

    /// Pulls the next token off of the serializer, ignoring it.
    pub fn next_token(&mut self) -> Option<&Token> {
        if let Some((first, rest)) = self.tokens.split_first() {
            self.tokens = rest;
            Some(first)
        } else {
            None
        }
    }
}

macro_rules! assert_next_token {
    ($self:ident, $expected:ident($arg:expr)) => {
        match $self.next_token() {
            Some(&Token::$expected(v)) if v == $arg => {}
            Some(other) => {
                panic!("expected {}({:?}) but serialized as {:?}", stringify!($expected), $arg, other);
            }
            None => {
                panic!("expected {}({:?}) after end of serialized tokens", stringify!($expected), $arg);
            }
        }
    }
}

impl<'s, 'a> ser::Serializer for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::Bool(v)));
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::I8(v)));
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::I16(v)));
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::I32(v)));
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::I64(v)));
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::U8(v)));
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::U16(v)));
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::U32(v)));
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::U64(v)));
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::F32(v)));
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::F64(v)));
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::Char(v)));
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<(), Error> {
        assert_next_token!(self, Str(v));
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<(), Self::Error> {
        assert_next_token!(self, Bytes(value));
        Ok(())
    }

    fn serialize_unit(self) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::Unit));
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::UnitStruct(name)));
        Ok(())
    }

    fn serialize_unit_variant(self,
                              name: &'static str,
                              _variant_index: usize,
                              variant: &'static str)
                              -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::EnumUnit(name, variant)));
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.next_token(), Some(&Token::StructNewType(name)));
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self,
                                            name: &'static str,
                                            _variant_index: usize,
                                            variant: &'static str,
                                            value: &T)
                                            -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.next_token(), Some(&Token::EnumNewType(name, variant)));
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::Option(false)));
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.next_token(), Some(&Token::Option(true)));
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self, Error> {
        assert_eq!(self.next_token(), Some(&Token::SeqStart(len)));
        Ok(self)
    }

    fn serialize_seq_fixed_size(self, len: usize) -> Result<Self, Error> {
        assert_eq!(self.next_token(), Some(&Token::SeqArrayStart(len)));
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self, Error> {
        assert_eq!(self.next_token(), Some(&Token::TupleStart(len)));
        Ok(self)
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self, Error> {
        assert_eq!(self.next_token(),
                   Some(&Token::TupleStructStart(name, len)));
        Ok(self)
    }

    fn serialize_tuple_variant(self,
                               name: &'static str,
                               _variant_index: usize,
                               variant: &'static str,
                               len: usize)
                               -> Result<Self, Error> {
        assert_eq!(self.next_token(),
                   Some(&Token::EnumSeqStart(name, variant, len)));
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self, Error> {
        assert_eq!(self.next_token(), Some(&Token::MapStart(len)));
        Ok(self)
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self, Error> {
        assert_eq!(self.next_token(), Some(&Token::StructStart(name, len)));
        Ok(self)
    }

    fn serialize_struct_variant(self,
                                name: &'static str,
                                _variant_index: usize,
                                variant: &'static str,
                                len: usize)
                                -> Result<Self, Error> {
        assert_eq!(self.next_token(),
                   Some(&Token::EnumMapStart(name, variant, len)));
        Ok(self)
    }
}

impl<'s, 'a> ser::SerializeSeq for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.next_token(), Some(&Token::SeqSep));
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::SeqEnd));
        Ok(())
    }
}

impl<'s, 'a> ser::SerializeTuple for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.next_token(), Some(&Token::TupleSep));
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::TupleEnd));
        Ok(())
    }
}

impl<'s, 'a> ser::SerializeTupleStruct for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.next_token(), Some(&Token::TupleStructSep));
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::TupleStructEnd));
        Ok(())
    }
}

impl<'s, 'a> ser::SerializeTupleVariant for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.next_token(), Some(&Token::EnumSeqSep));
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        assert_eq!(self.next_token(), Some(&Token::EnumSeqEnd));
        Ok(())
    }
}

impl<'s, 'a> ser::SerializeMap for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
        where T: Serialize
    {
        assert_eq!(self.next_token(), Some(&Token::MapSep));
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where T: Serialize
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        assert_eq!(self.next_token(), Some(&Token::MapEnd));
        Ok(())
    }
}

impl<'s, 'a> ser::SerializeStruct for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self,
                                  key: &'static str,
                                  value: &T)
                                  -> Result<(), Self::Error>
        where T: Serialize
    {
        assert_eq!(self.next_token(), Some(&Token::StructSep));
        try!(key.serialize(&mut **self));
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        assert_eq!(self.next_token(), Some(&Token::StructEnd));
        Ok(())
    }
}

impl<'s, 'a> ser::SerializeStructVariant for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self,
                                  key: &'static str,
                                  value: &T)
                                  -> Result<(), Self::Error>
        where T: Serialize
    {
        assert_eq!(self.next_token(), Some(&Token::EnumMapSep));
        try!(key.serialize(&mut **self));
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        assert_eq!(self.next_token(), Some(&Token::EnumMapEnd));
        Ok(())
    }
}
