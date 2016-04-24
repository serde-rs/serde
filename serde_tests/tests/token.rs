use std::fmt;
use std::iter;
use std::error;

extern crate serde;
use self::serde::ser::{self, Serialize};
use self::serde::de;
use self::serde::de::value::{self, ValueDeserializer};

#[derive(Clone, PartialEq, Debug)]
pub enum Token<'a> {
    Bool(bool),
    Isize(isize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Usize(usize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    Str(&'a str),
    String(String),
    Bytes(&'a [u8]),

    Option(bool),

    Unit,
    UnitStruct(&'a str),

    StructNewType(&'a str),

    EnumStart(&'a str),
    EnumUnit(&'a str, &'a str),
    EnumNewType(&'a str, &'a str),

    SeqStart(Option<usize>),
    SeqArrayStart(usize),
    SeqSep,
    SeqEnd,

    TupleStart(usize),
    TupleSep,
    TupleEnd,

    TupleStructStart(&'a str, Option<usize>),
    TupleStructSep,
    TupleStructEnd,

    MapStart(Option<usize>),
    MapSep,
    MapEnd,

    StructStart(&'a str, Option<usize>),
    StructSep,
    StructEnd,

    EnumSeqStart(&'a str, &'a str, Option<usize>),
    EnumSeqSep,
    EnumSeqEnd,

    EnumMapStart(&'a str, &'a str, Option<usize>),
    EnumMapSep,
    EnumMapEnd,
}

//////////////////////////////////////////////////////////////////////////////

pub struct Serializer<I> {
    tokens: I,
}

impl<'a, I> Serializer<I>
    where I: Iterator<Item=&'a Token<'a>>
{
    pub fn new(tokens: I) -> Serializer<I> {
        Serializer {
            tokens: tokens,
        }
    }

    fn visit_sequence<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: ser::SeqVisitor
    {
        while let Some(()) = try!(visitor.visit(self)) { }

        assert_eq!(self.tokens.next(), Some(&Token::SeqEnd));

        Ok(())
    }

    fn visit_mapping<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: ser::MapVisitor
    {
        while let Some(()) = try!(visitor.visit(self)) { }

        assert_eq!(self.tokens.next(), Some(&Token::MapEnd));

        Ok(())
    }
}

impl<'a, I> ser::Serializer for Serializer<I>
    where I: Iterator<Item=&'a Token<'a>>,
{
    type Error = Error;

    fn serialize_unit(&mut self) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::Unit));
        Ok(())
    }

    fn serialize_newtype_variant<T>(&mut self,
                                name: &str,
                                _variant_index: usize,
                                variant: &str,
                                value: T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        assert_eq!(self.tokens.next(), Some(&Token::EnumNewType(name, variant)));
        value.serialize(self)
    }

    fn serialize_unit_struct(&mut self, name: &str) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::UnitStruct(name)));
        Ok(())
    }

    fn serialize_unit_variant(&mut self,
                          name: &str,
                          _variant_index: usize,
                          variant: &str) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::EnumUnit(name, variant)));

        Ok(())
    }

    fn serialize_bool(&mut self, v: bool) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::Bool(v)));
        Ok(())
    }

    fn serialize_isize(&mut self, v: isize) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::Isize(v)));
        Ok(())
    }

    fn serialize_i8(&mut self, v: i8) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::I8(v)));
        Ok(())
    }

    fn serialize_i16(&mut self, v: i16) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::I16(v)));
        Ok(())
    }

    fn serialize_i32(&mut self, v: i32) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::I32(v)));
        Ok(())
    }

    fn serialize_i64(&mut self, v: i64) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::I64(v)));
        Ok(())
    }

    fn serialize_usize(&mut self, v: usize) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::Usize(v)));
        Ok(())
    }

    fn serialize_u8(&mut self, v: u8) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::U8(v)));
        Ok(())
    }

    fn serialize_u16(&mut self, v: u16) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::U16(v)));
        Ok(())
    }

    fn serialize_u32(&mut self, v: u32) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::U32(v)));
        Ok(())
    }

    fn serialize_u64(&mut self, v: u64) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::U64(v)));
        Ok(())
    }

    fn serialize_f32(&mut self, v: f32) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::F32(v)));
        Ok(())
    }

    fn serialize_f64(&mut self, v: f64) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::F64(v)));
        Ok(())
    }

    fn serialize_char(&mut self, v: char) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::Char(v)));
        Ok(())
    }

    fn serialize_str(&mut self, v: &str) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::Str(v)));
        Ok(())
    }

    fn serialize_none(&mut self) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::Option(false)));
        Ok(())
    }

    fn serialize_some<V>(&mut self, value: V) -> Result<(), Error>
        where V: ser::Serialize,
    {
        assert_eq!(self.tokens.next(), Some(&Token::Option(true)));
        value.serialize(self)
    }


    fn serialize_seq<V>(&mut self, visitor: V) -> Result<(), Error>
        where V: ser::SeqVisitor
    {
        let len = visitor.len();

        assert_eq!(self.tokens.next(), Some(&Token::SeqStart(len)));

        self.visit_sequence(visitor)
    }

    fn serialize_fixed_size_array<V>(&mut self, visitor: V) -> Result<(), Error>
        where V: ser::SeqVisitor
    {
        let len = visitor.len().expect("arrays must have a length");

        assert_eq!(self.tokens.next(), Some(&Token::SeqArrayStart(len)));

        self.visit_sequence(visitor)
    }

    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize
    {
        assert_eq!(self.tokens.next(), Some(&Token::SeqSep));
        value.serialize(self)
    }

    fn serialize_tuple<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: ser::SeqVisitor
    {
        let len = visitor.len().expect("arrays must have a length");

        assert_eq!(self.tokens.next(), Some(&Token::TupleStart(len)));

        while let Some(()) = try!(visitor.visit(self)) { }

        assert_eq!(self.tokens.next(), Some(&Token::TupleEnd));

        Ok(())
    }

    fn serialize_tuple_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize
    {
        assert_eq!(self.tokens.next(), Some(&Token::TupleSep));
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(&mut self,
                               name: &'static str,
                               value: T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        assert_eq!(self.tokens.next(), Some(&Token::StructNewType(name)));
        value.serialize(self)
    }

    fn serialize_tuple_struct<V>(&mut self, name: &str, mut visitor: V) -> Result<(), Error>
        where V: ser::SeqVisitor
    {
        let len = visitor.len();

        assert_eq!(self.tokens.next(), Some(&Token::TupleStructStart(name, len)));

        while let Some(()) = try!(visitor.visit(self)) { }

        assert_eq!(self.tokens.next(), Some(&Token::TupleStructEnd));

        Ok(())
    }

    fn serialize_tuple_struct_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        assert_eq!(self.tokens.next(), Some(&Token::TupleStructSep));
        value.serialize(self)
    }

    fn serialize_tuple_variant<V>(&mut self,
                                  name: &str,
                                  _variant_index: usize,
                                  variant: &str,
                                  mut visitor: V) -> Result<(), Error>
        where V: ser::SeqVisitor
    {
        let len = visitor.len();

        assert_eq!(self.tokens.next(), Some(&Token::EnumSeqStart(name, variant, len)));

        while let Some(()) = try!(visitor.visit(self)) { }

        assert_eq!(self.tokens.next(), Some(&Token::EnumSeqEnd));

        Ok(())
    }

    fn serialize_tuple_variant_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        assert_eq!(self.tokens.next(), Some(&Token::EnumSeqSep));
        value.serialize(self)
    }

    fn serialize_map<V>(&mut self, visitor: V) -> Result<(), Error>
        where V: ser::MapVisitor
    {
        let len = visitor.len();

        assert_eq!(self.tokens.next(), Some(&Token::MapStart(len)));

        self.visit_mapping(visitor)
    }

    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Error>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        assert_eq!(self.tokens.next(), Some(&Token::MapSep));

        try!(key.serialize(self));
        value.serialize(self)
    }

    fn serialize_struct<V>(&mut self, name: &str, mut visitor: V) -> Result<(), Error>
        where V: ser::MapVisitor
    {
        let len = visitor.len();

        assert_eq!(self.tokens.next(), Some(&Token::StructStart(name, len)));

        while let Some(()) = try!(visitor.visit(self)) { }

        assert_eq!(self.tokens.next(), Some(&Token::StructEnd));

        Ok(())
    }

    fn serialize_struct_elt<T>(&mut self, key: &'static str, value: T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        assert_eq!(self.tokens.next(), Some(&Token::StructSep));

        try!(key.serialize(self));
        value.serialize(self)
    }

    fn serialize_struct_variant<V>(&mut self,
                                   name: &str,
                                   _variant_index: usize,
                                   variant: &str,
                                   mut visitor: V) -> Result<(), Error>
        where V: ser::MapVisitor
    {
        let len = visitor.len();

        assert_eq!(self.tokens.next(), Some(&Token::EnumMapStart(name, variant, len)));

        while let Some(()) = try!(visitor.visit(self)) { }

        assert_eq!(self.tokens.next(), Some(&Token::EnumMapEnd));

        Ok(())
    }

    fn serialize_struct_variant_elt<T>(&mut self, key: &'static str, value: T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        assert_eq!(self.tokens.next(), Some(&Token::EnumMapSep));

        try!(key.serialize(self));
        value.serialize(self)
    }
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    SyntaxError,
    EndOfStreamError,
    UnknownFieldError(String),
    UnknownVariantError(String),
    MissingFieldError(&'static str),
    InvalidName(&'static str),
    InvalidValue(String),
    UnexpectedToken(Token<'static>),
    ValueError(value::Error),
}

impl ser::Error for Error {
    fn custom<T: Into<String>>(_: T) -> Error { Error::SyntaxError }

    fn invalid_value(msg: &str) -> Error {
        Error::InvalidValue(msg.to_owned())
    }
}

impl de::Error for Error {
    fn custom<T: Into<String>>(_: T) -> Error { Error::SyntaxError }

    fn end_of_stream() -> Error { Error::EndOfStreamError }

    fn unknown_field(field: &str) -> Error {
        Error::UnknownFieldError(field.to_owned())
    }

    fn unknown_variant(variant: &str) -> Error {
        Error::UnknownVariantError(variant.to_owned())
    }

    fn missing_field(field: &'static str) -> Error {
        Error::MissingFieldError(field)
    }
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

impl From<value::Error> for Error {
    fn from(error: value::Error) -> Error {
        Error::ValueError(error)
    }
}

struct Deserializer<I> where I: Iterator<Item=Token<'static>> {
    tokens: iter::Peekable<I>,
}

impl<I> Deserializer<I>
    where I: Iterator<Item=Token<'static>>
{
    fn new(tokens: I) -> Deserializer<I> {
        Deserializer {
            tokens: tokens.peekable(),
        }
    }

    fn visit_seq<V>(&mut self, len: Option<usize>, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(DeserializerSeqVisitor {
            de: self,
            len: len,
        })
    }

    fn visit_array<V>(&mut self, len: usize, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(DeserializerArrayVisitor {
            de: self,
            len: len,
        })
    }

    fn visit_tuple<V>(&mut self, len: usize, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(DeserializerTupleVisitor {
            de: self,
            len: len,
        })
    }

    fn visit_tuple_struct<V>(&mut self, len: usize, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(DeserializerTupleStructVisitor {
            de: self,
            len: len,
        })
    }

    fn visit_variant_seq<V>(&mut self, len: Option<usize>, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(DeserializerVariantSeqVisitor {
            de: self,
            len: len,
        })
    }

    fn visit_map<V>(&mut self, len: Option<usize>, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_map(DeserializerMapVisitor {
            de: self,
            len: len,
        })
    }

    fn visit_struct<V>(&mut self, len: Option<usize>, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_map(DeserializerStructVisitor {
            de: self,
            len: len,
        })
    }

    fn visit_variant_map<V>(&mut self, len: Option<usize>, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_map(DeserializerVariantMapVisitor {
            de: self,
            len: len,
        })
    }
}

impl<I> de::Deserializer for Deserializer<I>
    where I: Iterator<Item=Token<'static>>
{
    type Error = Error;

    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.next() {
            Some(Token::Bool(v)) => visitor.visit_bool(v),
            Some(Token::Isize(v)) => visitor.visit_isize(v),
            Some(Token::I8(v)) => visitor.visit_i8(v),
            Some(Token::I16(v)) => visitor.visit_i16(v),
            Some(Token::I32(v)) => visitor.visit_i32(v),
            Some(Token::I64(v)) => visitor.visit_i64(v),
            Some(Token::Usize(v)) => visitor.visit_usize(v),
            Some(Token::U8(v)) => visitor.visit_u8(v),
            Some(Token::U16(v)) => visitor.visit_u16(v),
            Some(Token::U32(v)) => visitor.visit_u32(v),
            Some(Token::U64(v)) => visitor.visit_u64(v),
            Some(Token::F32(v)) => visitor.visit_f32(v),
            Some(Token::F64(v)) => visitor.visit_f64(v),
            Some(Token::Char(v)) => visitor.visit_char(v),
            Some(Token::Str(v)) => visitor.visit_str(v),
            Some(Token::String(v)) => visitor.visit_string(v),
            Some(Token::Bytes(v)) => visitor.visit_bytes(v),
            Some(Token::Option(false)) => visitor.visit_none(),
            Some(Token::Option(true)) => visitor.visit_some(self),
            Some(Token::Unit) => visitor.visit_unit(),
            Some(Token::UnitStruct(name)) => visitor.visit_unit_struct(name),
            Some(Token::SeqStart(len)) | Some(Token::TupleStructStart(_, len)) => {
                self.visit_seq(len, visitor)
            }
            Some(Token::SeqArrayStart(len)) => {
                self.visit_seq(Some(len), visitor)
            }
            Some(Token::MapStart(len)) | Some(Token::StructStart(_, len)) => {
                self.visit_map(len, visitor)
            }
            Some(token) => Err(Error::UnexpectedToken(token)),
            None => Err(Error::EndOfStreamError),
        }
    }

    /// Hook into `Option` deserializing so we can treat `Unit` as a
    /// `None`, or a regular value as `Some(value)`.
    fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::Option(false)) => {
                self.tokens.next();
                visitor.visit_none()
            }
            Some(&Token::Option(true)) => {
                self.tokens.next();
                visitor.visit_some(self)
            }
            Some(&Token::Unit) => {
                self.tokens.next();
                visitor.visit_none()
            }
            Some(_) => visitor.visit_some(self),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn deserialize_enum<V>(&mut self,
                     name: &str,
                     _variants: &'static [&'static str],
                     mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        match self.tokens.peek() {
            Some(&Token::EnumStart(n)) if name == n => {
                self.tokens.next();

                visitor.visit(DeserializerVariantVisitor {
                    de: self,
                })
            }
            Some(&Token::EnumUnit(n, _))
            | Some(&Token::EnumNewType(n, _))
            | Some(&Token::EnumSeqStart(n, _, _))
            | Some(&Token::EnumMapStart(n, _, _)) if name == n => {
                visitor.visit(DeserializerVariantVisitor {
                    de: self,
                })
            }
            Some(_) => {
                let token = self.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => { return Err(Error::EndOfStreamError); }
        }
    }

    fn deserialize_unit_struct<V>(&mut self, name: &str, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::UnitStruct(n)) => {
                self.tokens.next();
                if name == n {
                    visitor.visit_unit()
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn deserialize_newtype_struct<V>(&mut self,
                                     name: &str,
                                     mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::StructNewType(n)) => {
                self.tokens.next();
                if name == n {
                    visitor.visit_newtype_struct(self)
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn deserialize_fixed_size_array<V>(&mut self,
                                       len: usize,
                                       visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::SeqArrayStart(_)) => {
                self.tokens.next();
                self.visit_array(len, visitor)
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn deserialize_tuple<V>(&mut self,
                            len: usize,
                            mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::Unit) => {
                self.tokens.next();
                visitor.visit_unit()
            }
            Some(&Token::UnitStruct(_)) => {
                self.tokens.next();
                visitor.visit_unit()
            }
            Some(&Token::SeqStart(_)) => {
                self.tokens.next();
                self.visit_seq(Some(len), visitor)
            }
            Some(&Token::SeqArrayStart(_)) => {
                self.tokens.next();
                self.visit_array(len, visitor)
            }
            Some(&Token::TupleStart(_)) => {
                self.tokens.next();
                self.visit_tuple(len, visitor)
            }
            Some(&Token::TupleStructStart(_, _)) => {
                self.tokens.next();
                self.visit_tuple_struct(len, visitor)
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn deserialize_tuple_struct<V>(&mut self,
                                   name: &str,
                                   len: usize,
                                   mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::Unit) => {
                self.tokens.next();
                visitor.visit_unit()
            }
            Some(&Token::UnitStruct(n)) => {
                self.tokens.next();
                if name == n {
                    visitor.visit_unit()
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(&Token::SeqStart(_)) => {
                self.tokens.next();
                self.visit_seq(Some(len), visitor)
            }
            Some(&Token::SeqArrayStart(_)) => {
                self.tokens.next();
                self.visit_array(len, visitor)
            }
            Some(&Token::TupleStart(_)) => {
                self.tokens.next();
                self.visit_tuple(len, visitor)
            }
            Some(&Token::TupleStructStart(n, _)) => {
                self.tokens.next();
                if name == n {
                    self.visit_tuple_struct(len, visitor)
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn deserialize_struct<V>(&mut self,
                             name: &str,
                             fields: &'static [&'static str],
                             visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::StructStart(n, _)) => {
                self.tokens.next();
                if name == n {
                    self.visit_struct(Some(fields.len()), visitor)
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(&Token::MapStart(_)) => {
                self.tokens.next();
                self.visit_map(Some(fields.len()), visitor)
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfStreamError),
        }
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerSeqVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: Option<usize>,
}

impl<'a, I> de::SeqVisitor for DeserializerSeqVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::SeqSep) => {
                self.de.tokens.next();
                self.len = self.len.map(|len| len - 1);
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            Some(&Token::SeqEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        //assert_eq!(self.len.unwrap_or(0), 0);
        match self.de.tokens.next() {
            Some(Token::SeqEnd) => Ok(()),
            Some(token) => Err(Error::UnexpectedToken(token)),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len.unwrap_or(0);
        (len, self.len)
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerArrayVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: usize,
}

impl<'a, I> de::SeqVisitor for DeserializerArrayVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::SeqSep) => {
                self.de.tokens.next();
                self.len -= 1;
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            Some(&Token::SeqEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        assert_eq!(self.len, 0);
        match self.de.tokens.next() {
            Some(Token::SeqEnd) => Ok(()),
            Some(token) => Err(Error::UnexpectedToken(token)),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerTupleVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: usize,
}

impl<'a, I> de::SeqVisitor for DeserializerTupleVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::TupleSep) => {
                self.de.tokens.next();
                self.len -= 1;
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            Some(&Token::TupleEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        assert_eq!(self.len, 0);
        match self.de.tokens.next() {
            Some(Token::TupleEnd) => Ok(()),
            Some(token) => Err(Error::UnexpectedToken(token)),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerTupleStructVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: usize,
}

impl<'a, I> de::SeqVisitor for DeserializerTupleStructVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::TupleStructSep) => {
                self.de.tokens.next();
                self.len -= 1;
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            Some(&Token::TupleStructEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        assert_eq!(self.len, 0);
        match self.de.tokens.next() {
            Some(Token::TupleStructEnd) => Ok(()),
            Some(token) => Err(Error::UnexpectedToken(token)),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerVariantSeqVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: Option<usize>,
}

impl<'a, I> de::SeqVisitor for DeserializerVariantSeqVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumSeqSep) => {
                self.de.tokens.next();
                self.len = self.len.map(|len| len - 1);
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            Some(&Token::EnumSeqEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        //assert_eq!(self.len.unwrap_or(0), 0);
        match self.de.tokens.next() {
            Some(Token::EnumSeqEnd) => Ok(()),
            Some(token) => Err(Error::UnexpectedToken(token)),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len.unwrap_or(0);
        (len, self.len)
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerMapVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: Option<usize>,
}

impl<'a, I> de::MapVisitor for DeserializerMapVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: de::Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::MapSep) => {
                self.de.tokens.next();
                self.len = self.len.map(|len| if len > 0 { len - 1} else { 0 });
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            Some(&Token::MapEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        Ok(try!(de::Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<(), Error> {
        //assert_eq!(self.len.unwrap_or(0), 0);
        match self.de.tokens.next() {
            Some(Token::MapEnd) => Ok(()),
            Some(token) => Err(Error::UnexpectedToken(token)),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len.unwrap_or(0);
        (len, self.len)
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerStructVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: Option<usize>,
}

impl<'a, I> de::MapVisitor for DeserializerStructVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: de::Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::StructSep) => {
                self.de.tokens.next();
                self.len = self.len.map(|len| if len > 0 { len - 1} else { 0 });
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            Some(&Token::StructEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        Ok(try!(de::Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<(), Error> {
        //assert_eq!(self.len.unwrap_or(0), 0);
        match self.de.tokens.next() {
            Some(Token::StructEnd) => Ok(()),
            Some(token) => Err(Error::UnexpectedToken(token)),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len.unwrap_or(0);
        (len, self.len)
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerVariantVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
}

impl<'a, I> de::VariantVisitor for DeserializerVariantVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_variant<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumUnit(_, v))
            | Some(&Token::EnumNewType(_, v))
            | Some(&Token::EnumSeqStart(_, v, _))
            | Some(&Token::EnumMapStart(_, v, _)) => {
                let mut de = ValueDeserializer::<Error>::into_deserializer(v);
                let value = try!(de::Deserialize::deserialize(&mut de));
                Ok(value)
            }
            Some(_) => {
                de::Deserialize::deserialize(self.de)
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_unit(&mut self) -> Result<(), Error> {
        match self.de.tokens.peek() {
            Some(&Token::EnumUnit(_, _)) => {
                self.de.tokens.next();
                Ok(())
            }
            Some(_) => {
                de::Deserialize::deserialize(self.de)
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_newtype<T>(&mut self) -> Result<T, Self::Error>
        where T: de::Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumNewType(_, _)) => {
                self.de.tokens.next();
                de::Deserialize::deserialize(self.de)
            }
            Some(_) => {
                de::Deserialize::deserialize(self.de)
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_tuple<V>(&mut self,
                      len: usize,
                      visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumSeqStart(_, _, Some(enum_len))) => {
                let token = self.de.tokens.next().unwrap();

                if len == enum_len {
                    self.de.visit_variant_seq(Some(len), visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(&Token::SeqStart(Some(enum_len))) => {
                let token = self.de.tokens.next().unwrap();

                if len == enum_len {
                    self.de.visit_seq(Some(len), visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(_) => {
                de::Deserialize::deserialize(self.de)
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_struct<V>(&mut self,
                       fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumMapStart(_, _, Some(enum_len))) => {
                let token = self.de.tokens.next().unwrap();

                if fields.len() == enum_len {
                    self.de.visit_variant_map(Some(fields.len()), visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(&Token::MapStart(Some(enum_len))) => {
                let token = self.de.tokens.next().unwrap();

                if fields.len() == enum_len {
                    self.de.visit_map(Some(fields.len()), visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(_) => {
                de::Deserialize::deserialize(self.de)
            }
            None => Err(Error::EndOfStreamError),
        }
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerVariantMapVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: Option<usize>,
}

impl<'a, I> de::MapVisitor for DeserializerVariantMapVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: de::Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumMapSep) => {
                self.de.tokens.next();
                self.len = self.len.map(|len| if len > 0 { len - 1} else { 0 });
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            Some(&Token::EnumMapEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        Ok(try!(de::Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<(), Error> {
        //assert_eq!(self.len.unwrap_or(0), 0);
        match self.de.tokens.next() {
            Some(Token::EnumMapEnd) => Ok(()),
            Some(token) => Err(Error::UnexpectedToken(token)),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len.unwrap_or(0);
        (len, self.len)
    }
}

//////////////////////////////////////////////////////////////////////////////

pub fn assert_ser_tokens<T>(value: &T, tokens: &[Token])
    where T: ser::Serialize,
{
    let mut ser = Serializer::new(tokens.iter());
    assert_eq!(ser::Serialize::serialize(value, &mut ser), Ok(()));
    assert_eq!(ser.tokens.next(), None);
}

// Expect an error deserializing tokens into a T
pub fn assert_ser_tokens_error<T>(value: &T, tokens: &[Token], error: Error)
    where T: ser::Serialize + PartialEq + fmt::Debug,
{
    let mut ser = Serializer::new(tokens.iter());
    let v: Result<(), Error> = ser::Serialize::serialize(value, &mut ser);
    assert_eq!(v.as_ref(), Err(&error));
}

pub fn assert_de_tokens<T>(value: &T, tokens: Vec<Token<'static>>)
    where T: de::Deserialize + PartialEq + fmt::Debug,
{
    let mut de = Deserializer::new(tokens.into_iter());
    let v: Result<T, Error> = de::Deserialize::deserialize(&mut de);
    assert_eq!(v.as_ref(), Ok(value));
    assert_eq!(de.tokens.next(), None);
}

// Expect an error deserializing tokens into a T
pub fn assert_de_tokens_error<T>(tokens: Vec<Token<'static>>, error: Error)
    where T: de::Deserialize + PartialEq + fmt::Debug,
{
    let mut de = Deserializer::new(tokens.into_iter());
    let v: Result<T, Error> = de::Deserialize::deserialize(&mut de);
    assert_eq!(v, Err(error));
}

// Tests that the given token stream is ignorable when embedded in
// an otherwise normal struct
pub fn assert_de_tokens_ignore(ignorable_tokens: Vec<Token<'static>>) {
    #[derive(PartialEq, Debug, Deserialize)]
    struct IgnoreBase {
        a: i32,
    }

    let expected = IgnoreBase{a: 1};

    // Embed the tokens to be ignored in the normal token
    // stream for an IgnoreBase type
    let concated_tokens : Vec<Token<'static>> = vec![
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("ignored")
        ]
        .into_iter()
        .chain(ignorable_tokens.into_iter())
        .chain(vec![
            Token::MapEnd,
        ].into_iter())
        .collect();

    let mut de = Deserializer::new(concated_tokens.into_iter());
    let v: Result<IgnoreBase, Error> = de::Deserialize::deserialize(&mut de);

    // We run this test on every token stream for convenience, but
    // some token streams don't make sense embedded as a map value,
    // so we ignore those. SyntaxError is the real sign of trouble.
    if let Err(Error::UnexpectedToken(_)) = v {
        return;
    }

    assert_eq!(v.as_ref(), Ok(&expected));
    assert_eq!(de.tokens.next(), None);
}

pub fn assert_tokens<T>(value: &T, tokens: Vec<Token<'static>>)
    where T: ser::Serialize + de::Deserialize + PartialEq + fmt::Debug,
{
    assert_ser_tokens(value, &tokens[..]);
    assert_de_tokens(value, tokens);
}
