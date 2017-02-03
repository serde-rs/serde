use std::iter;

use serde::de::{
    self,
    Deserialize,
    DeserializeSeed,
    EnumVisitor,
    MapVisitor,
    SeqVisitor,
    VariantVisitor,
    Visitor,
};
use serde::de::value::ValueDeserializer;

use error::Error;
use token::Token;

pub struct Deserializer<I>
    where I: Iterator<Item=Token<'static>>,
{
    tokens: iter::Peekable<I>,
}

impl<I> Deserializer<I>
    where I: Iterator<Item=Token<'static>>,
{
    pub fn new(tokens: I) -> Deserializer<I> {
        Deserializer {
            tokens: tokens.peekable(),
        }
    }

    pub fn next_token(&mut self) -> Option<Token<'static>> {
        self.tokens.next()
    }

    pub fn expect_token(&mut self, expected: Token) -> Result<(), Error> {
        match self.tokens.next() {
            Some(token) => {
                if expected == token {
                    Ok(())
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            None => Err(Error::EndOfTokens),
        }
    }

    fn visit_seq<V>(&mut self, len: Option<usize>, sep: Token<'static>, end: Token<'static>, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        let value = try!(visitor.visit_seq(DeserializerSeqVisitor {
            de: self,
            len: len,
            sep: sep,
            end: end.clone(),
        }));
        try!(self.expect_token(end));
        Ok(value)
    }

    fn visit_map<V>(&mut self, len: Option<usize>, sep: Token<'static>, end: Token<'static>, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        let value = try!(visitor.visit_map(DeserializerMapVisitor {
            de: self,
            len: len,
            sep: sep,
            end: end.clone(),
        }));
        try!(self.expect_token(end));
        Ok(value)
    }
}

impl<'a, I> de::Deserializer for &'a mut Deserializer<I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit
        seq bytes byte_buf map struct_field ignored_any
    }

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        match self.tokens.next() {
            Some(Token::Bool(v)) => visitor.visit_bool(v),
            Some(Token::I8(v)) => visitor.visit_i8(v),
            Some(Token::I16(v)) => visitor.visit_i16(v),
            Some(Token::I32(v)) => visitor.visit_i32(v),
            Some(Token::I64(v)) => visitor.visit_i64(v),
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
            Some(Token::ByteBuf(v)) => visitor.visit_byte_buf(v),
            Some(Token::Option(false)) => visitor.visit_none(),
            Some(Token::Option(true)) => visitor.visit_some(self),
            Some(Token::Unit) => visitor.visit_unit(),
            Some(Token::UnitStruct(_name)) => visitor.visit_unit(),
            Some(Token::SeqStart(len)) => {
                self.visit_seq(len, Token::SeqSep, Token::SeqEnd, visitor)
            }
            Some(Token::SeqArrayStart(len)) => {
                self.visit_seq(Some(len), Token::SeqSep, Token::SeqEnd, visitor)
            }
            Some(Token::TupleStart(len)) => {
                self.visit_seq(Some(len), Token::TupleSep, Token::TupleEnd, visitor)
            }
            Some(Token::TupleStructStart(_, len)) => {
                self.visit_seq(Some(len), Token::TupleStructSep, Token::TupleStructEnd, visitor)
            }
            Some(Token::MapStart(len)) => {
                self.visit_map(len, Token::MapSep, Token::MapEnd, visitor)
            }
            Some(Token::StructStart(_, len)) => {
                self.visit_map(Some(len), Token::StructSep, Token::StructEnd, visitor)
            }
            Some(token) => Err(Error::UnexpectedToken(token)),
            None => Err(Error::EndOfTokens),
        }
    }

    /// Hook into `Option` deserializing so we can treat `Unit` as a
    /// `None`, or a regular value as `Some(value)`.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::Unit) | Some(&Token::Option(false)) => {
                self.tokens.next();
                visitor.visit_none()
            }
            Some(&Token::Option(true)) => {
                self.tokens.next();
                visitor.visit_some(self)
            }
            Some(_) => visitor.visit_some(self),
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_enum<V>(self,
                     name: &str,
                     _variants: &'static [&'static str],
                     visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::EnumStart(n)) if name == n => {
                self.tokens.next();

                visitor.visit_enum(DeserializerEnumVisitor {
                    de: self,
                })
            }
            Some(&Token::EnumUnit(n, _))
            | Some(&Token::EnumNewType(n, _))
            | Some(&Token::EnumSeqStart(n, _, _))
            | Some(&Token::EnumMapStart(n, _, _)) if name == n => {
                visitor.visit_enum(DeserializerEnumVisitor {
                    de: self,
                })
            }
            Some(_) => {
                let token = self.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_unit_struct<V>(self, name: &str, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
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
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_newtype_struct<V>(self,
                                     name: &str,
                                     visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
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
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_seq_fixed_size<V>(self,
                                       len: usize,
                                       visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::SeqArrayStart(_)) => {
                self.tokens.next();
                self.visit_seq(Some(len), Token::SeqSep, Token::SeqEnd, visitor)
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_tuple<V>(self,
                            len: usize,
                            visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::Unit) | Some(&Token::UnitStruct(_)) => {
                self.tokens.next();
                visitor.visit_unit()
            }
            Some(&Token::SeqStart(_)) => {
                self.tokens.next();
                self.visit_seq(Some(len), Token::SeqSep, Token::SeqEnd, visitor)
            }
            Some(&Token::SeqArrayStart(_)) => {
                self.tokens.next();
                self.visit_seq(Some(len), Token::SeqSep, Token::SeqEnd, visitor)
            }
            Some(&Token::TupleStart(_)) => {
                self.tokens.next();
                self.visit_seq(Some(len), Token::TupleSep, Token::TupleEnd, visitor)
            }
            Some(&Token::TupleStructStart(_, _)) => {
                self.tokens.next();
                self.visit_seq(Some(len), Token::TupleStructSep, Token::TupleStructEnd, visitor)
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_tuple_struct<V>(self,
                                   name: &str,
                                   len: usize,
                                   visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
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
                self.visit_seq(Some(len), Token::SeqSep, Token::SeqEnd, visitor)
            }
            Some(&Token::SeqArrayStart(_)) => {
                self.tokens.next();
                self.visit_seq(Some(len), Token::SeqSep, Token::SeqEnd, visitor)
            }
            Some(&Token::TupleStart(_)) => {
                self.tokens.next();
                self.visit_seq(Some(len), Token::TupleSep, Token::TupleEnd, visitor)
            }
            Some(&Token::TupleStructStart(n, _)) => {
                self.tokens.next();
                if name == n {
                    self.visit_seq(Some(len), Token::TupleStructSep, Token::TupleStructEnd, visitor)
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_struct<V>(self,
                             name: &str,
                             fields: &'static [&'static str],
                             visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::StructStart(n, _)) => {
                self.tokens.next();
                if name == n {
                    self.visit_map(Some(fields.len()), Token::StructSep, Token::StructEnd, visitor)
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(&Token::MapStart(_)) => {
                self.tokens.next();
                self.visit_map(Some(fields.len()), Token::MapSep, Token::MapEnd, visitor)
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfTokens),
        }
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerSeqVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: Option<usize>,
    sep: Token<'static>,
    end: Token<'static>,
}

impl<'a, I> SeqVisitor for DeserializerSeqVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed,
    {
        if self.de.tokens.peek() == Some(&self.end) {
            return Ok(None);
        }
        match self.de.tokens.next() {
            Some(ref token) if *token == self.sep => {
                self.len = self.len.map(|len| len.saturating_sub(1));
                seed.deserialize(&mut *self.de).map(Some)
            }
            Some(other) => Err(Error::UnexpectedToken(other)),
            None => Err(Error::EndOfTokens),
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
    sep: Token<'static>,
    end: Token<'static>,
}

impl<'a, I> MapVisitor for DeserializerMapVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: DeserializeSeed,
    {
        if self.de.tokens.peek() == Some(&self.end) {
            return Ok(None);
        }
        match self.de.tokens.next() {
            Some(ref token) if *token == self.sep => {
                self.len = self.len.map(|len| len.saturating_sub(1));
                seed.deserialize(&mut *self.de).map(Some)
            }
            Some(other) => Err(Error::UnexpectedToken(other)),
            None => Err(Error::EndOfTokens),
        }
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where V: DeserializeSeed,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len.unwrap_or(0);
        (len, self.len)
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerEnumVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
}

impl<'a, I> EnumVisitor for DeserializerEnumVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;
    type Variant = Self;

    fn visit_variant_seed<V>(self, seed: V) -> Result<(V::Value, Self), Error>
        where V: DeserializeSeed,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumUnit(_, v))
            | Some(&Token::EnumNewType(_, v))
            | Some(&Token::EnumSeqStart(_, v, _))
            | Some(&Token::EnumMapStart(_, v, _)) => {
                let de = v.into_deserializer();
                let value = try!(seed.deserialize(de));
                Ok((value, self))
            }
            Some(_) => {
                let value = try!(seed.deserialize(&mut *self.de));
                Ok((value, self))
            }
            None => Err(Error::EndOfTokens),
        }
    }
}

impl<'a, I> VariantVisitor for DeserializerEnumVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>
{
    type Error = Error;

    fn visit_unit(self) -> Result<(), Error> {
        match self.de.tokens.peek() {
            Some(&Token::EnumUnit(_, _)) => {
                self.de.tokens.next();
                Ok(())
            }
            Some(_) => {
                Deserialize::deserialize(self.de)
            }
            None => Err(Error::EndOfTokens),
        }
    }

    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
        where T: DeserializeSeed,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumNewType(_, _)) => {
                self.de.tokens.next();
                seed.deserialize(self.de)
            }
            Some(_) => {
                seed.deserialize(self.de)
            }
            None => Err(Error::EndOfTokens),
        }
    }

    fn visit_tuple<V>(self,
                      len: usize,
                      visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumSeqStart(_, _, enum_len)) => {
                let token = self.de.tokens.next().unwrap();

                if len == enum_len {
                    self.de.visit_seq(Some(len), Token::EnumSeqSep, Token::EnumSeqEnd, visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(&Token::SeqStart(Some(enum_len))) => {
                let token = self.de.tokens.next().unwrap();

                if len == enum_len {
                    self.de.visit_seq(Some(len), Token::SeqSep, Token::SeqEnd, visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(_) => {
                de::Deserializer::deserialize(self.de, visitor)
            }
            None => Err(Error::EndOfTokens),
        }
    }

    fn visit_struct<V>(self,
                       fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumMapStart(_, _, enum_len)) => {
                let token = self.de.tokens.next().unwrap();

                if fields.len() == enum_len {
                    self.de.visit_map(Some(fields.len()), Token::EnumMapSep, Token::EnumMapEnd, visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(&Token::MapStart(Some(enum_len))) => {
                let token = self.de.tokens.next().unwrap();

                if fields.len() == enum_len {
                    self.de.visit_map(Some(fields.len()), Token::MapSep, Token::MapEnd, visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(_) => {
                de::Deserializer::deserialize(self.de, visitor)
            }
            None => Err(Error::EndOfTokens),
        }
    }
}
