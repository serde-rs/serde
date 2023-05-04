use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};

use error::Error;
use token::Token;

#[derive(Debug)]
pub struct Deserializer<'de> {
    tokens: &'de [Token],
}

fn assert_next_token(de: &mut Deserializer, expected: Token) -> Result<(), Error> {
    match de.next_token_opt() {
        Some(token) if token == expected => Ok(()),
        Some(other) => Err(de::Error::custom(format!(
            "expected Token::{} but deserialization wants Token::{}",
            other, expected,
        ))),
        None => Err(de::Error::custom(format!(
            "end of tokens but deserialization wants Token::{}",
            expected,
        ))),
    }
}

fn unexpected(token: Token) -> Error {
    de::Error::custom(format!(
        "deserialization did not expect this token: {}",
        token,
    ))
}

fn end_of_tokens() -> Error {
    de::Error::custom("ran out of tokens to deserialize")
}

impl<'de> Deserializer<'de> {
    pub fn new(tokens: &'de [Token]) -> Self {
        Deserializer { tokens: tokens }
    }

    fn peek_token_opt(&self) -> Option<Token> {
        self.tokens.first().cloned()
    }

    fn peek_token(&self) -> Result<Token, Error> {
        self.peek_token_opt().ok_or_else(end_of_tokens)
    }

    pub fn next_token_opt(&mut self) -> Option<Token> {
        match self.tokens.split_first() {
            Some((&first, rest)) => {
                self.tokens = rest;
                Some(first)
            }
            None => None,
        }
    }

    fn next_token(&mut self) -> Result<Token, Error> {
        let (&first, rest) = self.tokens.split_first().ok_or_else(end_of_tokens)?;
        self.tokens = rest;
        Ok(first)
    }

    pub fn remaining(&self) -> usize {
        self.tokens.len()
    }

    fn visit_seq<V>(
        &mut self,
        len: Option<usize>,
        end: Token,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_seq(DeserializerSeqVisitor {
            de: self,
            len: len,
            end: end,
        })?;
        assert_next_token(self, end)?;
        Ok(value)
    }

    fn visit_map<V>(
        &mut self,
        len: Option<usize>,
        end: Token,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_map(DeserializerMapVisitor {
            de: self,
            len: len,
            end: end,
        })?;
        assert_next_token(self, end)?;
        Ok(value)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit seq map identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let token = self.next_token()?;
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
            Token::None => visitor.visit_none(),
            Token::Some => visitor.visit_some(self),
            Token::Unit | Token::UnitStruct { .. } => visitor.visit_unit(),
            Token::NewtypeStruct { .. } => visitor.visit_newtype_struct(self),
            Token::Seq { len } => self.visit_seq(len, Token::SeqEnd, visitor),
            Token::Tuple { len } => self.visit_seq(Some(len), Token::TupleEnd, visitor),
            Token::TupleStruct { len, .. } => {
                self.visit_seq(Some(len), Token::TupleStructEnd, visitor)
            }
            Token::Map { len } => self.visit_map(len, Token::MapEnd, visitor),
            Token::Struct { len, .. } => self.visit_map(Some(len), Token::StructEnd, visitor),
            Token::Enum { .. } => {
                let variant = self.next_token()?;
                let next = self.peek_token()?;
                match (variant, next) {
                    (Token::Str(variant), Token::Unit) => {
                        self.next_token()?;
                        visitor.visit_str(variant)
                    }
                    (Token::BorrowedStr(variant), Token::Unit) => {
                        self.next_token()?;
                        visitor.visit_borrowed_str(variant)
                    }
                    (Token::String(variant), Token::Unit) => {
                        self.next_token()?;
                        visitor.visit_string(variant.to_string())
                    }
                    (Token::Bytes(variant), Token::Unit) => {
                        self.next_token()?;
                        visitor.visit_bytes(variant)
                    }
                    (Token::BorrowedBytes(variant), Token::Unit) => {
                        self.next_token()?;
                        visitor.visit_borrowed_bytes(variant)
                    }
                    (Token::ByteBuf(variant), Token::Unit) => {
                        self.next_token()?;
                        visitor.visit_byte_buf(variant.to_vec())
                    }
                    (Token::U8(variant), Token::Unit) => {
                        self.next_token()?;
                        visitor.visit_u8(variant)
                    }
                    (Token::U16(variant), Token::Unit) => {
                        self.next_token()?;
                        visitor.visit_u16(variant)
                    }
                    (Token::U32(variant), Token::Unit) => {
                        self.next_token()?;
                        visitor.visit_u32(variant)
                    }
                    (Token::U64(variant), Token::Unit) => {
                        self.next_token()?;
                        visitor.visit_u64(variant)
                    }
                    (variant, Token::Unit) => Err(unexpected(variant)),
                    (variant, _) => {
                        visitor.visit_map(EnumMapVisitor::new(self, variant, EnumFormat::Any))
                    }
                }
            }
            Token::UnitVariant { variant, .. } => visitor.visit_str(variant),
            Token::NewtypeVariant { variant, .. } => visitor.visit_map(EnumMapVisitor::new(
                self,
                Token::Str(variant),
                EnumFormat::Any,
            )),
            Token::TupleVariant { variant, .. } => visitor.visit_map(EnumMapVisitor::new(
                self,
                Token::Str(variant),
                EnumFormat::Seq,
            )),
            Token::StructVariant { variant, .. } => visitor.visit_map(EnumMapVisitor::new(
                self,
                Token::Str(variant),
                EnumFormat::Map,
            )),
            Token::SeqEnd
            | Token::TupleEnd
            | Token::TupleStructEnd
            | Token::MapEnd
            | Token::StructEnd
            | Token::TupleVariantEnd
            | Token::StructVariantEnd => Err(unexpected(token)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_token()? {
            Token::Unit | Token::None => {
                self.next_token()?;
                visitor.visit_none()
            }
            Token::Some => {
                self.next_token()?;
                visitor.visit_some(self)
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_token()? {
            Token::Enum { name: n } if name == n => {
                self.next_token()?;

                visitor.visit_enum(DeserializerEnumVisitor { de: self })
            }
            Token::UnitVariant { name: n, .. }
            | Token::NewtypeVariant { name: n, .. }
            | Token::TupleVariant { name: n, .. }
            | Token::StructVariant { name: n, .. }
                if name == n =>
            {
                visitor.visit_enum(DeserializerEnumVisitor { de: self })
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_token()? {
            Token::UnitStruct { .. } => {
                assert_next_token(self, Token::UnitStruct { name: name })?;
                visitor.visit_unit()
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_token()? {
            Token::NewtypeStruct { .. } => {
                assert_next_token(self, Token::NewtypeStruct { name: name })?;
                visitor.visit_newtype_struct(self)
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_token()? {
            Token::Unit | Token::UnitStruct { .. } => {
                self.next_token()?;
                visitor.visit_unit()
            }
            Token::Seq { .. } => {
                self.next_token()?;
                self.visit_seq(Some(len), Token::SeqEnd, visitor)
            }
            Token::Tuple { .. } => {
                self.next_token()?;
                self.visit_seq(Some(len), Token::TupleEnd, visitor)
            }
            Token::TupleStruct { .. } => {
                self.next_token()?;
                self.visit_seq(Some(len), Token::TupleStructEnd, visitor)
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_token()? {
            Token::Unit => {
                self.next_token()?;
                visitor.visit_unit()
            }
            Token::UnitStruct { .. } => {
                assert_next_token(self, Token::UnitStruct { name: name })?;
                visitor.visit_unit()
            }
            Token::Seq { .. } => {
                self.next_token()?;
                self.visit_seq(Some(len), Token::SeqEnd, visitor)
            }
            Token::Tuple { .. } => {
                self.next_token()?;
                self.visit_seq(Some(len), Token::TupleEnd, visitor)
            }
            Token::TupleStruct { len: n, .. } => {
                assert_next_token(self, Token::TupleStruct { name: name, len: n })?;
                self.visit_seq(Some(len), Token::TupleStructEnd, visitor)
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_token()? {
            Token::Struct { len: n, .. } => {
                assert_next_token(self, Token::Struct { name: name, len: n })?;
                self.visit_map(Some(fields.len()), Token::StructEnd, visitor)
            }
            Token::Map { .. } => {
                self.next_token()?;
                self.visit_map(Some(fields.len()), Token::MapEnd, visitor)
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn is_human_readable(&self) -> bool {
        panic!(
            "Types which have different human-readable and compact representations \
             must explicitly mark their test cases with `serde_test::Configure`"
        );
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerSeqVisitor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: Option<usize>,
    end: Token,
}

impl<'de, 'a> SeqAccess<'de> for DeserializerSeqVisitor<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.peek_token_opt() == Some(self.end) {
            return Ok(None);
        }
        self.len = self.len.map(|len| len.saturating_sub(1));
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerMapVisitor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: Option<usize>,
    end: Token,
}

impl<'de, 'a> MapAccess<'de> for DeserializerMapVisitor<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.de.peek_token_opt() == Some(self.end) {
            return Ok(None);
        }
        self.len = self.len.map(|len| len.saturating_sub(1));
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

//////////////////////////////////////////////////////////////////////////

struct DeserializerEnumVisitor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> EnumAccess<'de> for DeserializerEnumVisitor<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self), Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.de.peek_token()? {
            Token::UnitVariant { variant: v, .. }
            | Token::NewtypeVariant { variant: v, .. }
            | Token::TupleVariant { variant: v, .. }
            | Token::StructVariant { variant: v, .. } => {
                let de = v.into_deserializer();
                let value = seed.deserialize(de)?;
                Ok((value, self))
            }
            _ => {
                let value = seed.deserialize(&mut *self.de)?;
                Ok((value, self))
            }
        }
    }
}

impl<'de, 'a> VariantAccess<'de> for DeserializerEnumVisitor<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.de.peek_token()? {
            Token::UnitVariant { .. } => {
                self.de.next_token()?;
                Ok(())
            }
            _ => Deserialize::deserialize(self.de),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.de.peek_token()? {
            Token::NewtypeVariant { .. } => {
                self.de.next_token()?;
                seed.deserialize(self.de)
            }
            _ => seed.deserialize(self.de),
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.de.peek_token()? {
            Token::TupleVariant { len: enum_len, .. } => {
                let token = self.de.next_token()?;

                if len == enum_len {
                    self.de
                        .visit_seq(Some(len), Token::TupleVariantEnd, visitor)
                } else {
                    Err(unexpected(token))
                }
            }
            Token::Seq {
                len: Some(enum_len),
            } => {
                let token = self.de.next_token()?;

                if len == enum_len {
                    self.de.visit_seq(Some(len), Token::SeqEnd, visitor)
                } else {
                    Err(unexpected(token))
                }
            }
            _ => de::Deserializer::deserialize_any(self.de, visitor),
        }
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.de.peek_token()? {
            Token::StructVariant { len: enum_len, .. } => {
                let token = self.de.next_token()?;

                if fields.len() == enum_len {
                    self.de
                        .visit_map(Some(fields.len()), Token::StructVariantEnd, visitor)
                } else {
                    Err(unexpected(token))
                }
            }
            Token::Map {
                len: Some(enum_len),
            } => {
                let token = self.de.next_token()?;

                if fields.len() == enum_len {
                    self.de
                        .visit_map(Some(fields.len()), Token::MapEnd, visitor)
                } else {
                    Err(unexpected(token))
                }
            }
            _ => de::Deserializer::deserialize_any(self.de, visitor),
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

impl<'de, 'a> MapAccess<'de> for EnumMapVisitor<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.variant.take() {
            Some(Token::Str(variant)) => seed.deserialize(variant.into_deserializer()).map(Some),
            Some(Token::Bytes(variant)) => seed
                .deserialize(BytesDeserializer { value: variant })
                .map(Some),
            Some(Token::U32(variant)) => seed.deserialize(variant.into_deserializer()).map(Some),
            Some(other) => Err(unexpected(other)),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.format {
            EnumFormat::Seq => {
                let value = {
                    let visitor = DeserializerSeqVisitor {
                        de: self.de,
                        len: None,
                        end: Token::TupleVariantEnd,
                    };
                    seed.deserialize(SeqAccessDeserializer::new(visitor))?
                };
                assert_next_token(self.de, Token::TupleVariantEnd)?;
                Ok(value)
            }
            EnumFormat::Map => {
                let value = {
                    let visitor = DeserializerMapVisitor {
                        de: self.de,
                        len: None,
                        end: Token::StructVariantEnd,
                    };
                    seed.deserialize(MapAccessDeserializer::new(visitor))?
                };
                assert_next_token(self.de, Token::StructVariantEnd)?;
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bytes(self.value)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
