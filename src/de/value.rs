use std::collections::{
    BTreeMap,
    BTreeSet,
    HashMap,
    HashSet,
    btree_map,
    btree_set,
    hash_map,
    hash_set,
};
use std::hash::Hash;
use std::vec;

use de;

///////////////////////////////////////////////////////////////////////////////

pub enum Error {
    SyntaxError,
    EndOfStreamError,
    UnknownFieldError(String),
    MissingFieldError(&'static str),
}

impl de::Error for Error {
    fn syntax_error() -> Self { Error::SyntaxError }
    fn end_of_stream_error() -> Self { Error::EndOfStreamError }
    fn unknown_field_error(field: &str) -> Self { Error::UnknownFieldError(field.to_string()) }
    fn missing_field_error(field: &'static str) -> Self { Error::MissingFieldError(field) }
}

///////////////////////////////////////////////////////////////////////////////

pub trait ValueDeserializer {
    type Deserializer: de::Deserializer<Error=Error>;

    fn into_deserializer(self) -> Self::Deserializer;
}

///////////////////////////////////////////////////////////////////////////////

impl ValueDeserializer for () {
    type Deserializer = UnitDeserializer;

    fn into_deserializer(self) -> UnitDeserializer {
        UnitDeserializer
    }
}

/// A helper deserializer that deserializes a `()`.
pub struct UnitDeserializer;

impl de::Deserializer for UnitDeserializer {
    type Error = Error;

    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_unit()
    }

    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_none()
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! primitive_deserializer {
    ($ty:ty, $name:ident, $method:ident) => {
        pub struct $name(Option<$ty>);

        impl ValueDeserializer for $ty {
            type Deserializer = $name;

            fn into_deserializer(self) -> $name {
                $name(Some(self))
            }
        }

        impl de::Deserializer for $name {
            type Error = Error;

            fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
                where V: de::Visitor,
            {
                match self.0.take() {
                    Some(v) => visitor.$method(v),
                    None => Err(de::Error::end_of_stream_error()),
                }
            }
        }
    }
}

primitive_deserializer!(bool, BoolDeserializer, visit_bool);
primitive_deserializer!(i8, I8Deserializer, visit_i8);
primitive_deserializer!(i16, I16Deserializer, visit_i16);
primitive_deserializer!(i32, I32Deserializer, visit_i32);
primitive_deserializer!(i64, I64Deserializer, visit_i64);
primitive_deserializer!(isize, IsizeDeserializer, visit_isize);
primitive_deserializer!(u8, U8Deserializer, visit_u8);
primitive_deserializer!(u16, U16Deserializer, visit_u16);
primitive_deserializer!(u32, U32Deserializer, visit_u32);
primitive_deserializer!(u64, U64Deserializer, visit_u64);
primitive_deserializer!(usize, UsizeDeserializer, visit_usize);
primitive_deserializer!(f32, F32Deserializer, visit_f32);
primitive_deserializer!(f64, F64Deserializer, visit_f64);
primitive_deserializer!(char, CharDeserializer, visit_char);

///////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a `&str`.
pub struct StrDeserializer<'a>(Option<&'a str>);

impl<'a> ValueDeserializer for &'a str {
    type Deserializer = StrDeserializer<'a>;

    fn into_deserializer(self) -> StrDeserializer<'a> {
        StrDeserializer(Some(self))
    }
}

impl<'a> de::Deserializer for StrDeserializer<'a> {
    type Error = Error;

    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.0.take() {
            Some(v) => visitor.visit_str(v),
            None => Err(de::Error::end_of_stream_error()),
        }
    }

    fn visit_enum<V>(&mut self, _name: &str, mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        visitor.visit(self)
    }
}

impl<'a> de::VariantVisitor for StrDeserializer<'a> {
    type Error = Error;

    fn visit_variant<T>(&mut self) -> Result<T, Error>
        where T: de::Deserialize,
    {
        de::Deserialize::deserialize(self)
    }

    fn visit_unit(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a `String`.
pub struct StringDeserializer(Option<String>);

impl ValueDeserializer for String {
    type Deserializer = StringDeserializer;

    fn into_deserializer(self) -> StringDeserializer {
        StringDeserializer(Some(self))
    }
}

impl de::Deserializer for StringDeserializer {
    type Error = Error;

    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.0.take() {
            Some(string) => visitor.visit_string(string),
            None => Err(de::Error::end_of_stream_error()),
        }
    }

    fn visit_enum<V>(&mut self, _name: &str, mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        visitor.visit(self)
    }
}

impl<'a> de::VariantVisitor for StringDeserializer {
    type Error = Error;

    fn visit_variant<T>(&mut self) -> Result<T, Error>
        where T: de::Deserialize,
    {
        de::Deserialize::deserialize(self)
    }

    fn visit_unit(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct SeqDeserializer<I> {
    iter: I,
    len: usize,
}

impl<I> SeqDeserializer<I> {
    pub fn new(iter: I, len: usize) -> Self {
        SeqDeserializer {
            iter: iter,
            len: len,
        }
    }
}

impl<I, T> de::Deserializer for SeqDeserializer<I>
    where I: Iterator<Item=T>,
          T: ValueDeserializer,
{
    type Error = Error;

    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(self)
    }
}

impl<I, T> de::SeqVisitor for SeqDeserializer<I>
    where I: Iterator<Item=T>,
          T: ValueDeserializer,
{
    type Error = Error;

    fn visit<V>(&mut self) -> Result<Option<V>, Error>
        where V: de::Deserialize
    {
        match self.iter.next() {
            Some(value) => {
                self.len -= 1;
                let mut de = value.into_deserializer();
                Ok(Some(try!(de::Deserialize::deserialize(&mut de))))
            }
            None => Ok(None),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(de::Error::end_of_stream_error())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<T> ValueDeserializer for Vec<T>
    where T: ValueDeserializer,
{
    type Deserializer = SeqDeserializer<vec::IntoIter<T>>;

    fn into_deserializer(self) -> SeqDeserializer<vec::IntoIter<T>> {
        let len = self.len();
        SeqDeserializer::new(self.into_iter(), len)
    }
}

impl<T> ValueDeserializer for BTreeSet<T>
    where T: ValueDeserializer + Eq + Ord,
{
    type Deserializer = SeqDeserializer<btree_set::IntoIter<T>>;

    fn into_deserializer(self) -> SeqDeserializer<btree_set::IntoIter<T>> {
        let len = self.len();
        SeqDeserializer::new(self.into_iter(), len)
    }
}

impl<T> ValueDeserializer for HashSet<T>
    where T: ValueDeserializer + Eq + Hash,
{
    type Deserializer = SeqDeserializer<hash_set::IntoIter<T>>;

    fn into_deserializer(self) -> SeqDeserializer<hash_set::IntoIter<T>> {
        let len = self.len();
        SeqDeserializer::new(self.into_iter(), len)
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct MapDeserializer<I, K, V>
    where I: Iterator<Item=(K, V)>,
          K: ValueDeserializer,
          V: ValueDeserializer,
{
    iter: I,
    value: Option<V>,
    len: usize,
}

impl<I, K, V> MapDeserializer<I, K, V>
    where I: Iterator<Item=(K, V)>,
          K: ValueDeserializer,
          V: ValueDeserializer,
{
    pub fn new(iter: I, len: usize) -> Self {
        MapDeserializer {
            iter: iter,
            value: None,
            len: len,
        }
    }
}

impl<I, K, V> de::Deserializer for MapDeserializer<I, K, V>
    where I: Iterator<Item=(K, V)>,
          K: ValueDeserializer,
          V: ValueDeserializer,
{
    type Error = Error;

    fn visit<V_>(&mut self, mut visitor: V_) -> Result<V_::Value, Error>
        where V_: de::Visitor,
    {
        visitor.visit_map(self)
    }
}

impl<I, K, V> de::MapVisitor for MapDeserializer<I, K, V>
    where I: Iterator<Item=(K, V)>,
          K: ValueDeserializer,
          V: ValueDeserializer,
{
    type Error = Error;

    fn visit_key<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.len -= 1;
                self.value = Some(value);
                let mut de = key.into_deserializer();
                Ok(Some(try!(de::Deserialize::deserialize(&mut de))))
            }
            None => Ok(None),
        }
    }

    fn visit_value<T>(&mut self) -> Result<T, Error>
        where T: de::Deserialize,
    {
        match self.value.take() {
            Some(value) => {
                let mut de = value.into_deserializer();
                de::Deserialize::deserialize(&mut de)
            }
            None => Err(de::Error::syntax_error())
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(de::Error::end_of_stream_error())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<K, V> ValueDeserializer for BTreeMap<K, V>
    where K: ValueDeserializer + Eq + Ord,
          V: ValueDeserializer,
{
    type Deserializer = MapDeserializer<btree_map::IntoIter<K, V>, K, V>;

    fn into_deserializer(self) -> MapDeserializer<btree_map::IntoIter<K, V>, K, V> {
        let len = self.len();
        MapDeserializer::new(self.into_iter(), len)
    }
}

impl<K, V> ValueDeserializer for HashMap<K, V>
    where K: ValueDeserializer + Eq + Hash,
          V: ValueDeserializer,
{
    type Deserializer = MapDeserializer<hash_map::IntoIter<K, V>, K, V>;

    fn into_deserializer(self) -> MapDeserializer<hash_map::IntoIter<K, V>, K, V> {
        let len = self.len();
        MapDeserializer::new(self.into_iter(), len)
    }
}
