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

    fn visit_seq<V>(&mut self, len: Option<usize>, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        let value = try!(visitor.visit_seq(DeserializerSeqVisitor {
            de: self,
            len: len,
        }));
        try!(self.expect_token(Token::SeqEnd));
        Ok(value)
    }

    fn visit_array<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        let value = try!(visitor.visit_seq(DeserializerArrayVisitor {
            de: self,
            len: len,
        }));
        try!(self.expect_token(Token::SeqEnd));
        Ok(value)
    }

    fn visit_tuple<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        let value = try!(visitor.visit_seq(DeserializerTupleVisitor {
            de: self,
            len: len,
        }));
        try!(self.expect_token(Token::TupleEnd));
        Ok(value)
    }

    fn visit_tuple_struct<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        let value = try!(visitor.visit_seq(DeserializerTupleStructVisitor {
            de: self,
            len: len,
        }));
        try!(self.expect_token(Token::TupleStructEnd));
        Ok(value)
    }

    fn visit_variant_seq<V>(&mut self, len: Option<usize>, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        let value = try!(visitor.visit_seq(DeserializerVariantSeqVisitor {
            de: self,
            len: len,
        }));
        try!(self.expect_token(Token::EnumSeqEnd));
        Ok(value)
    }

    fn visit_map<V>(&mut self, len: Option<usize>, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        let value = try!(visitor.visit_map(DeserializerMapVisitor {
            de: self,
            len: len,
        }));
        try!(self.expect_token(Token::MapEnd));
        Ok(value)
    }

    fn visit_struct<V>(&mut self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        let value = try!(visitor.visit_map(DeserializerStructVisitor {
            de: self,
            len: fields.len(),
        }));
        try!(self.expect_token(Token::StructEnd));
        Ok(value)
    }

    fn visit_variant_map<V>(&mut self, len: Option<usize>, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        let value = try!(visitor.visit_map(DeserializerVariantMapVisitor {
            de: self,
            len: len,
        }));
        try!(self.expect_token(Token::EnumMapEnd));
        Ok(value)
    }
}

impl<'a, I> de::Deserializer for &'a mut Deserializer<I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn deserialize_seq<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_struct_field<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_map<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_unit<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_bytes<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_byte_buf<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_ignored_any<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_string<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_str<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_char<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_i64<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_i32<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_i16<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_i8<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_u64<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_u32<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_u16<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_u8<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_f32<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_f64<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_bool<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_usize<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }
    fn deserialize_isize<__V>(self, visitor: __V) -> Result<__V::Value, Self::Error>
        where __V: de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
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
            Some(Token::ByteBuf(v)) => visitor.visit_byte_buf(v),
            Some(Token::Option(false)) => visitor.visit_none(),
            Some(Token::Option(true)) => visitor.visit_some(self),
            Some(Token::Unit) => visitor.visit_unit(),
            Some(Token::UnitStruct(_name)) => visitor.visit_unit(),
            Some(Token::SeqStart(len)) => {
                self.visit_seq(len, visitor)
            }
            Some(Token::SeqArrayStart(len))| Some(Token::TupleStructStart(_, len)) => {
                self.visit_seq(Some(len), visitor)
            }
            Some(Token::MapStart(len)) => {
                self.visit_map(len, visitor)
            }
            Some(Token::StructStart(_, len)) => {
                self.visit_map(Some(len), visitor)
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
            None => { return Err(Error::EndOfTokens); }
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
                self.visit_array(len, visitor)
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
                    self.visit_struct(fields, visitor)
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(&Token::MapStart(_)) => {
                self.tokens.next();
                self.visit_map(Some(fields.len()), visitor)
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
}

impl<'a, I> SeqVisitor for DeserializerSeqVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed,
    {
        match self.de.tokens.peek() {
            Some(&Token::SeqSep) => {
                self.de.tokens.next();
                self.len = self.len.map(|len| len - 1);
                seed.deserialize(&mut *self.de).map(Some)
            }
            Some(&Token::SeqEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfTokens),
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

impl<'a, I> SeqVisitor for DeserializerArrayVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed,
    {
        match self.de.tokens.peek() {
            Some(&Token::SeqSep) => {
                self.de.tokens.next();
                self.len -= 1;
                seed.deserialize(&mut *self.de).map(Some)
            }
            Some(&Token::SeqEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfTokens),
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

impl<'a, I> SeqVisitor for DeserializerTupleVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed,
    {
        match self.de.tokens.peek() {
            Some(&Token::TupleSep) => {
                self.de.tokens.next();
                self.len -= 1;
                seed.deserialize(&mut *self.de).map(Some)
            }
            Some(&Token::TupleEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfTokens),
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

impl<'a, I> SeqVisitor for DeserializerTupleStructVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed,
    {
        match self.de.tokens.peek() {
            Some(&Token::TupleStructSep) => {
                self.de.tokens.next();
                self.len -= 1;
                seed.deserialize(&mut *self.de).map(Some)
            }
            Some(&Token::TupleStructEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfTokens),
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

impl<'a, I> SeqVisitor for DeserializerVariantSeqVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumSeqSep) => {
                self.de.tokens.next();
                self.len = self.len.map(|len| len - 1);
                seed.deserialize(&mut *self.de).map(Some)
            }
            Some(&Token::EnumSeqEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
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
}

impl<'a, I> MapVisitor for DeserializerMapVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: DeserializeSeed,
    {
        match self.de.tokens.peek() {
            Some(&Token::MapSep) => {
                self.de.tokens.next();
                self.len = self.len.map(|len| if len > 0 { len - 1} else { 0 });
                seed.deserialize(&mut *self.de).map(Some)
            }
            Some(&Token::MapEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
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

struct DeserializerStructVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: usize,
}

impl<'a, I> MapVisitor for DeserializerStructVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: DeserializeSeed,
    {
        match self.de.tokens.peek() {
            Some(&Token::StructSep) => {
                self.de.tokens.next();
                self.len = self.len.saturating_sub(1);
                seed.deserialize(&mut *self.de).map(Some)
            }
            Some(&Token::StructEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfTokens),
        }
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where V: DeserializeSeed,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
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
                de::Deserializer::deserialize(self.de, visitor)
            }
            None => Err(Error::EndOfTokens),
        }
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerVariantMapVisitor<'a, I: 'a> where I: Iterator<Item=Token<'static>> {
    de: &'a mut Deserializer<I>,
    len: Option<usize>,
}

impl<'a, I> MapVisitor for DeserializerVariantMapVisitor<'a, I>
    where I: Iterator<Item=Token<'static>>,
{
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: DeserializeSeed,
    {
        match self.de.tokens.peek() {
            Some(&Token::EnumMapSep) => {
                self.de.tokens.next();
                self.len = self.len.map(|len| if len > 0 { len - 1} else { 0 });
                seed.deserialize(&mut *self.de).map(Some)
            }
            Some(&Token::EnumMapEnd) => Ok(None),
            Some(_) => {
                let token = self.de.tokens.next().unwrap();
                Err(Error::UnexpectedToken(token))
            }
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
