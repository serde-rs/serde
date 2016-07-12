use std::marker::PhantomData;

use serde::ser::{
    self,
    Serialize,
};

use error::Error;
use token::Token;

pub struct Serializer<'a, I>
    where I: Iterator<Item=&'a Token<'a>>,
{
    tokens: I,
    phantom: PhantomData<&'a Token<'a>>,
}

impl<'a, I> Serializer<'a, I>
    where I: Iterator<Item=&'a Token<'a>>,
{
    pub fn new(tokens: I) -> Serializer<'a, I> {
        Serializer {
            tokens: tokens,
            phantom: PhantomData,
        }
    }

    pub fn next_token(&mut self) -> Option<&'a Token<'a>> {
        self.tokens.next()
    }
}

impl<'a, I> ser::Serializer for Serializer<'a, I>
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
        where T: Serialize,
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
        where V: Serialize,
    {
        assert_eq!(self.tokens.next(), Some(&Token::Option(true)));
        value.serialize(self)
    }

    fn serialize_seq<'b>(&'b mut self, len: Option<usize>) -> Result<(), Error>
    {
        assert_eq!(self.tokens.next(), Some(&Token::SeqStart(len)));
        Ok(())
    }

    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.tokens.next(), Some(&Token::SeqSep));
        value.serialize(self)
    }

    fn serialize_seq_end(&mut self, _len: Option<usize>) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::SeqEnd));
        Ok(())
    }

    fn serialize_fixed_size_array<'b>(&'b mut self, len: usize) -> Result<(), Error>
    {
        assert_eq!(self.tokens.next(), Some(&Token::SeqArrayStart(len)));
        Ok(())
    }

    fn serialize_tuple<'b>(&'b mut self, len: usize) -> Result<(), Error>
    {
        assert_eq!(self.tokens.next(), Some(&Token::TupleStart(len)));
        Ok(())
    }

    fn serialize_tuple_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.tokens.next(), Some(&Token::TupleSep));
        value.serialize(self)
    }

    fn serialize_tuple_end(&mut self, _len: usize) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::TupleEnd));
        Ok(())
    }

    fn serialize_newtype_struct<T>(&mut self,
                                   name: &'static str,
                                   value: T) -> Result<(), Error>
        where T: Serialize,
    {
        assert_eq!(self.tokens.next(), Some(&Token::StructNewType(name)));
        value.serialize(self)
    }

    fn serialize_tuple_struct<'b>(&'b mut self, name: &str, len: usize) -> Result<(), Error>
    {
        assert_eq!(self.tokens.next(), Some(&Token::TupleStructStart(name, len)));

        Ok(())
    }

    fn serialize_tuple_struct_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.tokens.next(), Some(&Token::TupleStructSep));
        value.serialize(self)
    }

    fn serialize_tuple_struct_end(&mut self, _name: &str, _len: usize) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::TupleStructEnd));
        Ok(())
    }

    fn serialize_tuple_variant(&mut self,
                               name: &str,
                               _variant_index: usize,
                               variant: &str,
                               len: usize) -> Result<(), Error>
    {
        assert_eq!(self.tokens.next(), Some(&Token::EnumSeqStart(name, variant, len)));

        Ok(())
    }

    fn serialize_tuple_variant_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: Serialize
    {
        assert_eq!(self.tokens.next(), Some(&Token::EnumSeqSep));
        value.serialize(self)
    }

    fn serialize_tuple_variant_end(&mut self,
                               _name: &str,
                               _variant_index: usize,
                               _variant: &str,
                               _len: usize) -> Result<(), Error> {
        assert_eq!(self.tokens.next(), Some(&Token::EnumSeqEnd));
        Ok(())
    }

    fn serialize_map(&mut self, len: Option<usize>) -> Result<(), Error>
    {
        assert_eq!(self.tokens.next(), Some(&Token::MapStart(len)));

        Ok(())
    }

    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize {
        assert_eq!(self.tokens.next(), Some(&Token::MapSep));
        try!(key.serialize(self));
        value.serialize(self)
    }

    fn serialize_map_end(&mut self, _len: Option<usize>) -> Result<(), Self::Error> {
        assert_eq!(self.tokens.next(), Some(&Token::MapEnd));
        Ok(())
    }

    fn serialize_struct(&mut self, name: &str, len: usize) -> Result<(), Error>
    {
        assert_eq!(self.tokens.next(), Some(&Token::StructStart(name, len)));

        Ok(())
    }

    fn serialize_struct_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize {
        assert_eq!(self.tokens.next(), Some(&Token::StructSep));
        try!(key.serialize(self));
        value.serialize(self)
    }

    fn serialize_struct_end(&mut self, _name: &str, _len: usize) -> Result<(), Self::Error> {
        assert_eq!(self.tokens.next(), Some(&Token::StructEnd));
        Ok(())
    }

    fn serialize_struct_variant<'b>(&'b mut self,
                                name: &str,
                                _variant_index: usize,
                                variant: &str,
                                len: usize) -> Result<(), Error>
    {
        assert_eq!(self.tokens.next(), Some(&Token::EnumMapStart(name, variant, len)));

        Ok(())
    }

    fn serialize_struct_variant_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize {
        assert_eq!(self.tokens.next(), Some(&Token::EnumMapSep));
        try!(key.serialize(self));
        value.serialize(self)
    }

    fn serialize_struct_variant_end(&mut self,
                                _name: &str,
                                _variant_index: usize,
                                _variant: &str,
                                _len: usize) -> Result<(), Self::Error> {
        assert_eq!(self.tokens.next(), Some(&Token::EnumMapEnd));
        Ok(())
    }
}
