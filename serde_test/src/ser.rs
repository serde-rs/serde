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
    pub fn next_token(&mut self) -> Option<Token> {
        if let Some((&first, rest)) = self.tokens.split_first() {
            self.tokens = rest;
            Some(first)
        } else {
            None
        }
    }
}

macro_rules! assert_next_token {
    ($self:ident, $expected:ident($a:expr)) => {
        assert_next_token!($self, $expected { a: $a });
    };
    ($self:ident, $expected:ident($a:expr, $b:expr)) => {
        assert_next_token!($self, $expected { a: $a, b: $b });
    };
    ($self:ident, $expected:ident($a:expr, $b:expr, $c:expr)) => {
        assert_next_token!($self, $expected { a: $a, b: $b, c: $c });
    };
    ($self:ident, $expected:ident $({ $($n:ident: $v:expr),* })*) => {
        match $self.next_token() {
            Some(Token::$expected $(($($n),*))*) $(if $($n == $v)&&*)* => {}
            Some(other) => {
                panic!("expected Token::{} but serialized as {:?}",
                       stringify!($expected), other);
            }
            None => {
                panic!("expected Token::{} after end of serialized tokens",
                       stringify!($expected));
            }
        }
    };
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
        assert_next_token!(self, Bool(v));
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        assert_next_token!(self, I8(v));
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        assert_next_token!(self, I16(v));
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        assert_next_token!(self, I32(v));
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        assert_next_token!(self, I64(v));
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        assert_next_token!(self, U8(v));
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        assert_next_token!(self, U16(v));
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        assert_next_token!(self, U32(v));
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        assert_next_token!(self, U64(v));
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        assert_next_token!(self, F32(v));
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        assert_next_token!(self, F64(v));
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<(), Error> {
        assert_next_token!(self, Char(v));
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
        assert_next_token!(self, Unit);
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<(), Error> {
        assert_next_token!(self, UnitStruct(name));
        Ok(())
    }

    fn serialize_unit_variant(self,
                              name: &'static str,
                              _variant_index: usize,
                              variant: &'static str)
                              -> Result<(), Error> {
        if self.tokens.first() == Some(&Token::EnumStart(name)) {
            self.next_token();
            assert_next_token!(self, Str(variant));
            assert_next_token!(self, Unit);
        } else {
            assert_next_token!(self, EnumUnit(name, variant));
        }
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_next_token!(self, StructNewType(name));
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
        if self.tokens.first() == Some(&Token::EnumStart(name)) {
            self.next_token();
            assert_next_token!(self, Str(variant));
        } else {
            assert_next_token!(self, EnumNewType(name, variant));
        }
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<(), Error> {
        assert_next_token!(self, Option(false));
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_next_token!(self, Option(true));
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self, Error> {
        assert_next_token!(self, SeqStart(len));
        Ok(self)
    }

    fn serialize_seq_fixed_size(self, len: usize) -> Result<Self, Error> {
        assert_next_token!(self, SeqArrayStart(len));
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self, Error> {
        assert_next_token!(self, TupleStart(len));
        Ok(self)
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self, Error> {
        assert_next_token!(self, TupleStructStart(name, len));
        Ok(self)
    }

    fn serialize_tuple_variant(self,
                               name: &'static str,
                               _variant_index: usize,
                               variant: &'static str,
                               len: usize)
                               -> Result<Self, Error> {
        assert_next_token!(self, EnumSeqStart(name, variant, len));
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self, Error> {
        assert_next_token!(self, MapStart(len));
        Ok(self)
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self, Error> {
        assert_next_token!(self, StructStart(name, len));
        Ok(self)
    }

    fn serialize_struct_variant(self,
                                name: &'static str,
                                _variant_index: usize,
                                variant: &'static str,
                                len: usize)
                                -> Result<Self, Error> {
        assert_next_token!(self, EnumMapStart(name, variant, len));
        Ok(self)
    }
}

impl<'s, 'a> ser::SerializeSeq for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_next_token!(self, SeqSep);
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        assert_next_token!(self, SeqEnd);
        Ok(())
    }
}

impl<'s, 'a> ser::SerializeTuple for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_next_token!(self, TupleSep);
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        assert_next_token!(self, TupleEnd);
        Ok(())
    }
}

impl<'s, 'a> ser::SerializeTupleStruct for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_next_token!(self, TupleStructSep);
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        assert_next_token!(self, TupleStructEnd);
        Ok(())
    }
}

impl<'s, 'a> ser::SerializeTupleVariant for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where T: Serialize
    {
        assert_next_token!(self, EnumSeqSep);
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        assert_next_token!(self, EnumSeqEnd);
        Ok(())
    }
}

impl<'s, 'a> ser::SerializeMap for &'s mut Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
        where T: Serialize
    {
        assert_next_token!(self, MapSep);
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where T: Serialize
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        assert_next_token!(self, MapEnd);
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
        assert_next_token!(self, StructSep);
        try!(key.serialize(&mut **self));
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        assert_next_token!(self, StructEnd);
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
        assert_next_token!(self, EnumMapSep);
        try!(key.serialize(&mut **self));
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        assert_next_token!(self, EnumMapEnd);
        Ok(())
    }
}
