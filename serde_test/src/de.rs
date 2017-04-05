use serde::de::{self, Deserialize, DeserializeSeed, EnumVisitor, MapVisitor, SeqVisitor,
                VariantVisitor, Visitor};
use serde::de::value::{ValueDeserializer, MapVisitorDeserializer, SeqVisitorDeserializer};

use error::Error;
use token::Token;

/// A `Deserializer` that reads from a list of tokens.
pub struct Deserializer<'de> {
    tokens: &'de [Token],
}

impl<'de> Deserializer<'de> {
    /// Creates the deserializer.
    pub fn new(tokens: &'de [Token]) -> Self {
        Deserializer { tokens: tokens }
    }

    /// Pulls the next token off of the deserializer, ignoring it.
    pub fn next_token(&mut self) -> Option<Token> {
        if let Some((&first, rest)) = self.tokens.split_first() {
            self.tokens = rest;
            Some(first)
        } else {
            None
        }
    }

    /// Pulls the next token off of the deserializer and checks if it matches an expected token.
    pub fn expect_token(&mut self, expected: Token) -> Result<(), Error> {
        match self.next_token() {
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

    fn visit_seq<V>(&mut self,
                    len: Option<usize>,
                    end: Token,
                    visitor: V)
                    -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        let value = try!(visitor.visit_seq(DeserializerSeqVisitor {
            de: self,
            len: len,
            end: end.clone(),
        }));
        try!(self.expect_token(end));
        Ok(value)
    }

    fn visit_map<V>(&mut self,
                    len: Option<usize>,
                    end: Token,
                    visitor: V)
                    -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        let value = try!(visitor.visit_map(DeserializerMapVisitor {
            de: self,
            len: len,
            end: end.clone(),
        }));
        try!(self.expect_token(end));
        Ok(value)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit
        seq bytes byte_buf map struct_field ignored_any
    }

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        let token = self.next_token().ok_or(Error::EndOfTokens)?;
        match token {
            Token::Bool(v) => visitor.visit_bool(v),
            Token::I8(v) => visitor.visit_i8(v),
            Token::I16(v) => visitor.visit_i16(v),
            Token::I32(v) => visitor.visit_i32(v),
            Token::I64(v) => visitor.visit_i64(v),
            Token::U8(v) => visitor.visit_u8(v),
            Token::U16(v) => visitor.visit_u16(v),
            Token::U32(v) => visitor.visit_u32(v),
            Token::U64(v) => visitor.visit_u64(v),
            Token::F32(v) => visitor.visit_f32(v),
            Token::F64(v) => visitor.visit_f64(v),
            Token::Char(v) => visitor.visit_char(v),
            Token::Str(v) => visitor.visit_str(v),
            Token::BorrowedStr(v) => visitor.visit_borrowed_str(v),
            Token::String(v) => visitor.visit_string(v.to_owned()),
            Token::Bytes(v) => visitor.visit_bytes(v),
            Token::BorrowedBytes(v) => visitor.visit_borrowed_bytes(v),
            Token::ByteBuf(v) => visitor.visit_byte_buf(v.to_vec()),
            Token::Option(false) => visitor.visit_none(),
            Token::Option(true) => visitor.visit_some(self),
            Token::Unit => visitor.visit_unit(),
            Token::UnitStruct(_name) => visitor.visit_unit(),
            Token::StructNewType(_name) => visitor.visit_newtype_struct(self),
            Token::SeqStart(len) => {
                self.visit_seq(len, Token::SeqEnd, visitor)
            }
            Token::SeqArrayStart(len) => {
                self.visit_seq(Some(len), Token::SeqEnd, visitor)
            }
            Token::TupleStart(len) => {
                self.visit_seq(Some(len), Token::TupleEnd, visitor)
            }
            Token::TupleStructStart(_, len) => {
                self.visit_seq(Some(len),
                               Token::TupleStructEnd,
                               visitor)
            }
            Token::MapStart(len) => {
                self.visit_map(len, Token::MapEnd, visitor)
            }
            Token::StructStart(_, len) => {
                self.visit_map(Some(len), Token::StructEnd, visitor)
            }
            Token::EnumStart(_) => {
                let variant = self.next_token().ok_or(Error::EndOfTokens)?;
                let next = *self.tokens.first().ok_or(Error::EndOfTokens)?;
                match (variant, next) {
                    (Token::Str(variant), Token::Unit) => {
                        self.next_token();
                        visitor.visit_str(variant)
                    }
                    (Token::Bytes(variant), Token::Unit) => {
                        self.next_token();
                        visitor.visit_bytes(variant)
                    }
                    (Token::U32(variant), Token::Unit) => {
                        self.next_token();
                        visitor.visit_u32(variant)
                    }
                    (variant, Token::Unit) => {
                        Err(Error::UnexpectedToken(variant))
                    }
                    (variant, _) => {
                        visitor.visit_map(EnumMapVisitor::new(self, variant, EnumFormat::Any))
                    }
                }
            }
            Token::EnumUnit(_, variant) => visitor.visit_str(variant),
            Token::EnumNewType(_, variant) => {
                visitor.visit_map(EnumMapVisitor::new(self, Token::Str(variant), EnumFormat::Any))
            }
            Token::EnumSeqStart(_, variant, _) => {
                visitor.visit_map(EnumMapVisitor::new(self, Token::Str(variant), EnumFormat::Seq))
            }
            Token::EnumMapStart(_, variant, _) => {
                visitor.visit_map(EnumMapVisitor::new(self, Token::Str(variant), EnumFormat::Map))
            }
            Token::SeqEnd |
            Token::TupleEnd |
            Token::TupleStructEnd |
            Token::MapEnd |
            Token::StructEnd |
            Token::EnumSeqEnd |
            Token::EnumMapEnd => Err(Error::UnexpectedToken(token)),
        }
    }

    /// Hook into `Option` deserializing so we can treat `Unit` as a
    /// `None`, or a regular value as `Some(value)`.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.tokens.first() {
            Some(&Token::Unit) |
            Some(&Token::Option(false)) => {
                self.next_token();
                visitor.visit_none()
            }
            Some(&Token::Option(true)) => {
                self.next_token();
                visitor.visit_some(self)
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_enum<V>(self,
                           name: &str,
                           _variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.tokens.first() {
            Some(&Token::EnumStart(n)) if name == n => {
                self.next_token();

                visitor.visit_enum(DeserializerEnumVisitor { de: self })
            }
            Some(&Token::EnumUnit(n, _)) |
            Some(&Token::EnumNewType(n, _)) |
            Some(&Token::EnumSeqStart(n, _, _)) |
            Some(&Token::EnumMapStart(n, _, _)) if name == n => {
                visitor.visit_enum(DeserializerEnumVisitor { de: self })
            }
            Some(_) => {
                let token = self.next_token().unwrap();
                Err(Error::UnexpectedToken(token))
            }
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_unit_struct<V>(self, name: &str, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.tokens.first() {
            Some(&Token::UnitStruct(n)) => {
                self.next_token();
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

    fn deserialize_newtype_struct<V>(self, name: &str, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.tokens.first() {
            Some(&Token::StructNewType(n)) => {
                self.next_token();
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

    fn deserialize_seq_fixed_size<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.tokens.first() {
            Some(&Token::SeqArrayStart(_)) => {
                self.next_token();
                self.visit_seq(Some(len), Token::SeqEnd, visitor)
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.tokens.first() {
            Some(&Token::Unit) |
            Some(&Token::UnitStruct(_)) => {
                self.next_token();
                visitor.visit_unit()
            }
            Some(&Token::SeqStart(_)) => {
                self.next_token();
                self.visit_seq(Some(len), Token::SeqEnd, visitor)
            }
            Some(&Token::SeqArrayStart(_)) => {
                self.next_token();
                self.visit_seq(Some(len), Token::SeqEnd, visitor)
            }
            Some(&Token::TupleStart(_)) => {
                self.next_token();
                self.visit_seq(Some(len), Token::TupleEnd, visitor)
            }
            Some(&Token::TupleStructStart(_, _)) => {
                self.next_token();
                self.visit_seq(Some(len),
                               Token::TupleStructEnd,
                               visitor)
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfTokens),
        }
    }

    fn deserialize_tuple_struct<V>(self,
                                   name: &str,
                                   len: usize,
                                   visitor: V)
                                   -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.tokens.first() {
            Some(&Token::Unit) => {
                self.next_token();
                visitor.visit_unit()
            }
            Some(&Token::UnitStruct(n)) => {
                self.next_token();
                if name == n {
                    visitor.visit_unit()
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(&Token::SeqStart(_)) => {
                self.next_token();
                self.visit_seq(Some(len), Token::SeqEnd, visitor)
            }
            Some(&Token::SeqArrayStart(_)) => {
                self.next_token();
                self.visit_seq(Some(len), Token::SeqEnd, visitor)
            }
            Some(&Token::TupleStart(_)) => {
                self.next_token();
                self.visit_seq(Some(len), Token::TupleEnd, visitor)
            }
            Some(&Token::TupleStructStart(n, _)) => {
                self.next_token();
                if name == n {
                    self.visit_seq(Some(len),
                                   Token::TupleStructEnd,
                                   visitor)
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
                             visitor: V)
                             -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.tokens.first() {
            Some(&Token::StructStart(n, _)) => {
                self.next_token();
                if name == n {
                    self.visit_map(Some(fields.len()),
                                   Token::StructEnd,
                                   visitor)
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(&Token::MapStart(_)) => {
                self.next_token();
                self.visit_map(Some(fields.len()), Token::MapEnd, visitor)
            }
            Some(_) => self.deserialize(visitor),
            None => Err(Error::EndOfTokens),
        }
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerSeqVisitor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: Option<usize>,
    end: Token,
}

impl<'de, 'a> SeqVisitor<'de> for DeserializerSeqVisitor<'a, 'de> {
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed<'de>
    {
        if self.de.tokens.first() == Some(&self.end) {
            return Ok(None);
        }
        self.len = self.len.map(|len| len.saturating_sub(1));
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len.unwrap_or(0);
        (len, self.len)
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerMapVisitor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: Option<usize>,
    end: Token,
}

impl<'de, 'a> MapVisitor<'de> for DeserializerMapVisitor<'a, 'de> {
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: DeserializeSeed<'de>
    {
        if self.de.tokens.first() == Some(&self.end) {
            return Ok(None);
        }
        self.len = self.len.map(|len| len.saturating_sub(1));
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where V: DeserializeSeed<'de>
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len.unwrap_or(0);
        (len, self.len)
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerEnumVisitor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> EnumVisitor<'de> for DeserializerEnumVisitor<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn visit_variant_seed<V>(self, seed: V) -> Result<(V::Value, Self), Error>
        where V: DeserializeSeed<'de>
    {
        match self.de.tokens.first() {
            Some(&Token::EnumUnit(_, v)) |
            Some(&Token::EnumNewType(_, v)) |
            Some(&Token::EnumSeqStart(_, v, _)) |
            Some(&Token::EnumMapStart(_, v, _)) => {
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

impl<'de, 'a> VariantVisitor<'de> for DeserializerEnumVisitor<'a, 'de> {
    type Error = Error;

    fn visit_unit(self) -> Result<(), Error> {
        match self.de.tokens.first() {
            Some(&Token::EnumUnit(_, _)) => {
                self.de.next_token();
                Ok(())
            }
            Some(_) => Deserialize::deserialize(self.de),
            None => Err(Error::EndOfTokens),
        }
    }

    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
        where T: DeserializeSeed<'de>
    {
        match self.de.tokens.first() {
            Some(&Token::EnumNewType(_, _)) => {
                self.de.next_token();
                seed.deserialize(self.de)
            }
            Some(_) => seed.deserialize(self.de),
            None => Err(Error::EndOfTokens),
        }
    }

    fn visit_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.de.tokens.first() {
            Some(&Token::EnumSeqStart(_, _, enum_len)) => {
                let token = self.de.next_token().unwrap();

                if len == enum_len {
                    self.de.visit_seq(Some(len), Token::EnumSeqEnd, visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(&Token::SeqStart(Some(enum_len))) => {
                let token = self.de.next_token().unwrap();

                if len == enum_len {
                    self.de.visit_seq(Some(len), Token::SeqEnd, visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(_) => de::Deserializer::deserialize(self.de, visitor),
            None => Err(Error::EndOfTokens),
        }
    }

    fn visit_struct<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.de.tokens.first() {
            Some(&Token::EnumMapStart(_, _, enum_len)) => {
                let token = self.de.next_token().unwrap();

                if fields.len() == enum_len {
                    self.de.visit_map(Some(fields.len()),
                                      Token::EnumMapEnd,
                                      visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(&Token::MapStart(Some(enum_len))) => {
                let token = self.de.next_token().unwrap();

                if fields.len() == enum_len {
                    self.de.visit_map(Some(fields.len()), Token::MapEnd, visitor)
                } else {
                    Err(Error::UnexpectedToken(token))
                }
            }
            Some(_) => de::Deserializer::deserialize(self.de, visitor),
            None => Err(Error::EndOfTokens),
        }
    }
}

//////////////////////////////////////////////////////////////////////////

struct EnumMapVisitor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    variant: Option<Token>,
    format: EnumFormat,
}

enum EnumFormat {
    Seq,
    Map,
    Any,
}

impl<'a, 'de> EnumMapVisitor<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, variant: Token, format: EnumFormat) -> Self {
        EnumMapVisitor {
            de: de,
            variant: Some(variant),
            format: format,
        }
    }
}

impl<'de, 'a> MapVisitor<'de> for EnumMapVisitor<'a, 'de> {
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: DeserializeSeed<'de>
    {
        match self.variant.take() {
            Some(Token::Str(variant)) => {
                seed.deserialize(variant.into_deserializer()).map(Some)
            }
            Some(Token::Bytes(variant)) => {
                seed.deserialize(BytesDeserializer { value: variant }).map(Some)
            }
            Some(Token::U32(variant)) => {
                seed.deserialize(variant.into_deserializer()).map(Some)
            }
            Some(other) => {
                Err(Error::UnexpectedToken(other))
            }
            None => Ok(None),
        }
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where V: DeserializeSeed<'de>
    {
        match self.format {
            EnumFormat::Seq => {
                let value = {
                    let visitor = DeserializerSeqVisitor {
                        de: self.de,
                        len: None,
                        end: Token::EnumSeqEnd,
                    };
                    try!(seed.deserialize(SeqVisitorDeserializer::new(visitor)))
                };
                try!(self.de.expect_token(Token::EnumSeqEnd));
                Ok(value)
            }
            EnumFormat::Map => {
                let value = {
                    let visitor = DeserializerMapVisitor {
                        de: self.de,
                        len: None,
                        end: Token::EnumMapEnd,
                    };
                    try!(seed.deserialize(MapVisitorDeserializer::new(visitor)))
                };
                try!(self.de.expect_token(Token::EnumMapEnd));
                Ok(value)
            }
            EnumFormat::Any => seed.deserialize(&mut *self.de),
        }
    }
}

struct BytesDeserializer {
    value: &'static [u8],
}

impl<'de> de::Deserializer<'de> for BytesDeserializer {
    type Error = Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor<'de>
    {
        visitor.visit_bytes(self.value)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct struct_field tuple enum ignored_any byte_buf
    }
}
