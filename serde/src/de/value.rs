// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Building blocks for deserializing basic values using the `IntoDeserializer`
//! trait.
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_derive;
//!
//! extern crate serde;
//!
//! use std::str::FromStr;
//! use serde::de::{value, Deserialize, IntoDeserializer};
//!
//! #[derive(Deserialize)]
//! enum Setting {
//!     On,
//!     Off,
//! }
//!
//! impl FromStr for Setting {
//!     type Err = value::Error;
//!
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         Self::deserialize(s.into_deserializer())
//!     }
//! }
//! #
//! # fn main() {}
//! ```

use lib::*;

use self::private::{First, Second};
use de::{self, Expected, IntoDeserializer, SeqAccess};
use private::de::size_hint;
use ser;

////////////////////////////////////////////////////////////////////////////////

/// A minimal representation of all possible errors that can occur using the
/// `IntoDeserializer` trait.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    err: ErrorImpl,
}

#[cfg(any(feature = "std", feature = "alloc"))]
type ErrorImpl = Box<str>;
#[cfg(not(any(feature = "std", feature = "alloc")))]
type ErrorImpl = ();

impl de::Error for Error {
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error {
            err: msg.to_string().into_boxed_str(),
        }
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        let _ = msg;
        Error { err: () }
    }
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        de::Error::custom(msg)
    }
}

impl Display for Error {
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str(&self.err)
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str("Serde deserialization error")
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn description(&self) -> &str {
        &self.err
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'de, E> IntoDeserializer<'de, E> for ()
where
    E: de::Error,
{
    type Deserializer = UnitDeserializer<E>;

    fn into_deserializer(self) -> UnitDeserializer<E> {
        UnitDeserializer {
            marker: PhantomData,
        }
    }
}

/// A deserializer holding a `()`.
#[derive(Clone, Debug)]
pub struct UnitDeserializer<E> {
    marker: PhantomData<E>,
}

impl<'de, E> de::Deserializer<'de> for UnitDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct seq tuple tuple_struct map
        struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_none()
    }
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! primitive_deserializer {
    ($ty:ty, $doc:tt, $name:ident, $method:ident $($cast:tt)*) => {
        #[doc = "A deserializer holding"]
        #[doc = $doc]
        #[derive(Clone, Debug)]
        pub struct $name<E> {
            value: $ty,
            marker: PhantomData<E>
        }

        impl<'de, E> IntoDeserializer<'de, E> for $ty
        where
            E: de::Error,
        {
            type Deserializer = $name<E>;

            fn into_deserializer(self) -> $name<E> {
                $name {
                    value: self,
                    marker: PhantomData,
                }
            }
        }

        impl<'de, E> de::Deserializer<'de> for $name<E>
        where
            E: de::Error,
        {
            type Error = E;

            forward_to_deserialize_any! {
                bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
                byte_buf option unit unit_struct newtype_struct seq tuple
                tuple_struct map struct enum identifier ignored_any
            }

            fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                visitor.$method(self.value $($cast)*)
            }
        }
    }
}

primitive_deserializer!(bool, "a `bool`.", BoolDeserializer, visit_bool);
primitive_deserializer!(i8, "an `i8`.", I8Deserializer, visit_i8);
primitive_deserializer!(i16, "an `i16`.", I16Deserializer, visit_i16);
primitive_deserializer!(i32, "an `i32`.", I32Deserializer, visit_i32);
primitive_deserializer!(i64, "an `i64`.", I64Deserializer, visit_i64);
primitive_deserializer!(isize, "an `isize`.", IsizeDeserializer, visit_i64 as i64);
primitive_deserializer!(u8, "a `u8`.", U8Deserializer, visit_u8);
primitive_deserializer!(u16, "a `u16`.", U16Deserializer, visit_u16);
primitive_deserializer!(u64, "a `u64`.", U64Deserializer, visit_u64);
primitive_deserializer!(usize, "a `usize`.", UsizeDeserializer, visit_u64 as u64);
primitive_deserializer!(f32, "an `f32`.", F32Deserializer, visit_f32);
primitive_deserializer!(f64, "an `f64`.", F64Deserializer, visit_f64);
primitive_deserializer!(char, "a `char`.", CharDeserializer, visit_char);

/// A deserializer holding a `u32`.
#[derive(Clone, Debug)]
pub struct U32Deserializer<E> {
    value: u32,
    marker: PhantomData<E>,
}

impl<'de, E> IntoDeserializer<'de, E> for u32
where
    E: de::Error,
{
    type Deserializer = U32Deserializer<E>;

    fn into_deserializer(self) -> U32Deserializer<E> {
        U32Deserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

impl<'de, E> de::Deserializer<'de> for U32Deserializer<E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.value)
    }

    fn deserialize_enum<V>(
        self,
        name: &str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let _ = name;
        let _ = variants;
        visitor.visit_enum(self)
    }
}

impl<'de, E> de::EnumAccess<'de> for U32Deserializer<E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `&str`.
#[derive(Clone, Debug)]
pub struct StrDeserializer<'a, E> {
    value: &'a str,
    marker: PhantomData<E>,
}

impl<'de, 'a, E> IntoDeserializer<'de, E> for &'a str
where
    E: de::Error,
{
    type Deserializer = StrDeserializer<'a, E>;

    fn into_deserializer(self) -> StrDeserializer<'a, E> {
        StrDeserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

impl<'de, 'a, E> de::Deserializer<'de> for StrDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.value)
    }

    fn deserialize_enum<V>(
        self,
        name: &str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let _ = name;
        let _ = variants;
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }
}

impl<'de, 'a, E> de::EnumAccess<'de> for StrDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `&str` with a lifetime tied to another
/// deserializer.
#[derive(Clone, Debug)]
pub struct BorrowedStrDeserializer<'de, E> {
    value: &'de str,
    marker: PhantomData<E>,
}

impl<'de, E> BorrowedStrDeserializer<'de, E> {
    /// Create a new borrowed deserializer from the given string.
    pub fn new(value: &'de str) -> BorrowedStrDeserializer<'de, E> {
        BorrowedStrDeserializer {
            value: value,
            marker: PhantomData,
        }
    }
}

impl<'de, E> de::Deserializer<'de> for BorrowedStrDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.value)
    }

    fn deserialize_enum<V>(
        self,
        name: &str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let _ = name;
        let _ = variants;
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }
}

impl<'de, E> de::EnumAccess<'de> for BorrowedStrDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `String`.
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Clone, Debug)]
pub struct StringDeserializer<E> {
    value: String,
    marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, E> IntoDeserializer<'de, E> for String
where
    E: de::Error,
{
    type Deserializer = StringDeserializer<E>;

    fn into_deserializer(self) -> StringDeserializer<E> {
        StringDeserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, E> de::Deserializer<'de> for StringDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.value)
    }

    fn deserialize_enum<V>(
        self,
        name: &str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let _ = name;
        let _ = variants;
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, 'a, E> de::EnumAccess<'de> for StringDeserializer<E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `Cow<str>`.
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Clone, Debug)]
pub struct CowStrDeserializer<'a, E> {
    value: Cow<'a, str>,
    marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, 'a, E> IntoDeserializer<'de, E> for Cow<'a, str>
where
    E: de::Error,
{
    type Deserializer = CowStrDeserializer<'a, E>;

    fn into_deserializer(self) -> CowStrDeserializer<'a, E> {
        CowStrDeserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, 'a, E> de::Deserializer<'de> for CowStrDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Cow::Borrowed(string) => visitor.visit_str(string),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let _ = name;
        let _ = variants;
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, 'a, E> de::EnumAccess<'de> for CowStrDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `&[u8]` with a lifetime tied to another
/// deserializer.
#[derive(Clone, Debug)]
pub struct BorrowedBytesDeserializer<'de, E> {
    value: &'de [u8],
    marker: PhantomData<E>,
}

impl<'de, E> BorrowedBytesDeserializer<'de, E> {
    /// Create a new borrowed deserializer from the given byte slice.
    pub fn new(value: &'de [u8]) -> BorrowedBytesDeserializer<'de, E> {
        BorrowedBytesDeserializer {
            value: value,
            marker: PhantomData,
        }
    }
}

impl<'de, E> de::Deserializer<'de> for BorrowedBytesDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.value)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct identifier ignored_any enum
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer that iterates over a sequence.
#[derive(Clone, Debug)]
pub struct SeqDeserializer<I, E> {
    iter: iter::Fuse<I>,
    count: usize,
    marker: PhantomData<E>,
}

impl<I, E> SeqDeserializer<I, E>
where
    I: Iterator,
{
    /// Construct a new `SeqDeserializer<I, E>`.
    pub fn new(iter: I) -> Self {
        SeqDeserializer {
            iter: iter.fuse(),
            count: 0,
            marker: PhantomData,
        }
    }
}

impl<I, E> SeqDeserializer<I, E>
where
    I: Iterator,
    E: de::Error,
{
    /// Check for remaining elements after passing a `SeqDeserializer` to
    /// `Visitor::visit_seq`.
    pub fn end(mut self) -> Result<(), E> {
        let mut remaining = 0;
        while self.iter.next().is_some() {
            remaining += 1;
        }
        if remaining == 0 {
            Ok(())
        } else {
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(de::Error::invalid_length(
                self.count + remaining,
                &ExpectedInSeq(self.count),
            ))
        }
    }
}

impl<'de, I, T, E> de::Deserializer<'de> for SeqDeserializer<I, E>
where
    I: Iterator<Item = T>,
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let v = try!(visitor.visit_seq(&mut self));
        try!(self.end());
        Ok(v)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

impl<'de, I, T, E> de::SeqAccess<'de> for SeqDeserializer<I, E>
where
    I: Iterator<Item = T>,
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn next_element_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => {
                self.count += 1;
                seed.deserialize(value.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        size_hint::from_bounds(&self.iter)
    }
}

struct ExpectedInSeq(usize);

impl Expected for ExpectedInSeq {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 1 {
            write!(formatter, "1 element in sequence")
        } else {
            write!(formatter, "{} elements in sequence", self.0)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, T, E> IntoDeserializer<'de, E> for Vec<T>
where
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Deserializer = SeqDeserializer<<Self as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.into_iter())
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, T, E> IntoDeserializer<'de, E> for BTreeSet<T>
where
    T: IntoDeserializer<'de, E> + Eq + Ord,
    E: de::Error,
{
    type Deserializer = SeqDeserializer<<Self as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.into_iter())
    }
}

#[cfg(feature = "std")]
impl<'de, T, S, E> IntoDeserializer<'de, E> for HashSet<T, S>
where
    T: IntoDeserializer<'de, E> + Eq + Hash,
    S: BuildHasher,
    E: de::Error,
{
    type Deserializer = SeqDeserializer<<Self as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.into_iter())
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `SeqAccess`.
#[derive(Clone, Debug)]
pub struct SeqAccessDeserializer<A> {
    seq: A,
}

impl<A> SeqAccessDeserializer<A> {
    /// Construct a new `SeqAccessDeserializer<A>`.
    pub fn new(seq: A) -> Self {
        SeqAccessDeserializer { seq: seq }
    }
}

impl<'de, A> de::Deserializer<'de> for SeqAccessDeserializer<A>
where
    A: de::SeqAccess<'de>,
{
    type Error = A::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self.seq)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer that iterates over a map.
pub struct MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
{
    iter: iter::Fuse<I>,
    value: Option<Second<I::Item>>,
    count: usize,
    lifetime: PhantomData<&'de ()>,
    error: PhantomData<E>,
}

impl<'de, I, E> MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
{
    /// Construct a new `MapDeserializer<I, E>`.
    pub fn new(iter: I) -> Self {
        MapDeserializer {
            iter: iter.fuse(),
            value: None,
            count: 0,
            lifetime: PhantomData,
            error: PhantomData,
        }
    }
}

impl<'de, I, E> MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
    E: de::Error,
{
    /// Check for remaining elements after passing a `MapDeserializer` to
    /// `Visitor::visit_map`.
    pub fn end(mut self) -> Result<(), E> {
        let mut remaining = 0;
        while self.iter.next().is_some() {
            remaining += 1;
        }
        if remaining == 0 {
            Ok(())
        } else {
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(de::Error::invalid_length(
                self.count + remaining,
                &ExpectedInMap(self.count),
            ))
        }
    }
}

impl<'de, I, E> MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
{
    fn next_pair(&mut self) -> Option<(First<I::Item>, Second<I::Item>)> {
        match self.iter.next() {
            Some(kv) => {
                self.count += 1;
                Some(private::Pair::split(kv))
            }
            None => None,
        }
    }
}

impl<'de, I, E> de::Deserializer<'de> for MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
    First<I::Item>: IntoDeserializer<'de, E>,
    Second<I::Item>: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = try!(visitor.visit_map(&mut self));
        try!(self.end());
        Ok(value)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = try!(visitor.visit_seq(&mut self));
        try!(self.end());
        Ok(value)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let _ = len;
        self.deserialize_seq(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct tuple_struct map struct
        enum identifier ignored_any
    }
}

impl<'de, I, E> de::MapAccess<'de> for MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
    First<I::Item>: IntoDeserializer<'de, E>,
    Second<I::Item>: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.next_pair() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let value = self.value.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let value = value.expect("MapAccess::visit_value called before visit_key");
        seed.deserialize(value.into_deserializer())
    }

    fn next_entry_seed<TK, TV>(
        &mut self,
        kseed: TK,
        vseed: TV,
    ) -> Result<Option<(TK::Value, TV::Value)>, Self::Error>
    where
        TK: de::DeserializeSeed<'de>,
        TV: de::DeserializeSeed<'de>,
    {
        match self.next_pair() {
            Some((key, value)) => {
                let key = try!(kseed.deserialize(key.into_deserializer()));
                let value = try!(vseed.deserialize(value.into_deserializer()));
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        size_hint::from_bounds(&self.iter)
    }
}

impl<'de, I, E> de::SeqAccess<'de> for MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
    First<I::Item>: IntoDeserializer<'de, E>,
    Second<I::Item>: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.next_pair() {
            Some((k, v)) => {
                let de = PairDeserializer(k, v, PhantomData);
                seed.deserialize(de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        size_hint::from_bounds(&self.iter)
    }
}

// Cannot #[derive(Clone)] because of the bound `Second<I::Item>: Clone`.
impl<'de, I, E> Clone for MapDeserializer<'de, I, E>
where
    I: Iterator + Clone,
    I::Item: private::Pair,
    Second<I::Item>: Clone,
{
    fn clone(&self) -> Self {
        MapDeserializer {
            iter: self.iter.clone(),
            value: self.value.clone(),
            count: self.count,
            lifetime: self.lifetime,
            error: self.error,
        }
    }
}

// Cannot #[derive(Debug)] because of the bound `Second<I::Item>: Debug`.
impl<'de, I, E> Debug for MapDeserializer<'de, I, E>
where
    I: Iterator + Debug,
    I::Item: private::Pair,
    Second<I::Item>: Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("MapDeserializer")
            .field("iter", &self.iter)
            .field("value", &self.value)
            .field("count", &self.count)
            .field("lifetime", &self.lifetime)
            .field("error", &self.error)
            .finish()
    }
}

// Used in the `impl SeqAccess for MapDeserializer` to visit the map as a
// sequence of pairs.
struct PairDeserializer<A, B, E>(A, B, PhantomData<E>);

impl<'de, A, B, E> de::Deserializer<'de> for PairDeserializer<A, B, E>
where
    A: IntoDeserializer<'de, E>,
    B: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct tuple_struct map struct
        enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let mut pair_visitor = PairVisitor(Some(self.0), Some(self.1), PhantomData);
        let pair = try!(visitor.visit_seq(&mut pair_visitor));
        if pair_visitor.1.is_none() {
            Ok(pair)
        } else {
            let remaining = pair_visitor.size_hint().unwrap();
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(de::Error::invalid_length(2, &ExpectedInSeq(2 - remaining)))
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if len == 2 {
            self.deserialize_seq(visitor)
        } else {
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(de::Error::invalid_length(2, &ExpectedInSeq(len)))
        }
    }
}

struct PairVisitor<A, B, E>(Option<A>, Option<B>, PhantomData<E>);

impl<'de, A, B, E> de::SeqAccess<'de> for PairVisitor<A, B, E>
where
    A: IntoDeserializer<'de, E>,
    B: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(k) = self.0.take() {
            seed.deserialize(k.into_deserializer()).map(Some)
        } else if let Some(v) = self.1.take() {
            seed.deserialize(v.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        if self.0.is_some() {
            Some(2)
        } else if self.1.is_some() {
            Some(1)
        } else {
            Some(0)
        }
    }
}

struct ExpectedInMap(usize);

impl Expected for ExpectedInMap {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 1 {
            write!(formatter, "1 element in map")
        } else {
            write!(formatter, "{} elements in map", self.0)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, K, V, E> IntoDeserializer<'de, E> for BTreeMap<K, V>
where
    K: IntoDeserializer<'de, E> + Eq + Ord,
    V: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Deserializer = MapDeserializer<'de, <Self as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapDeserializer::new(self.into_iter())
    }
}

#[cfg(feature = "std")]
impl<'de, K, V, S, E> IntoDeserializer<'de, E> for HashMap<K, V, S>
where
    K: IntoDeserializer<'de, E> + Eq + Hash,
    V: IntoDeserializer<'de, E>,
    S: BuildHasher,
    E: de::Error,
{
    type Deserializer = MapDeserializer<'de, <Self as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapDeserializer::new(self.into_iter())
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `MapAccess`.
#[derive(Clone, Debug)]
pub struct MapAccessDeserializer<A> {
    map: A,
}

impl<A> MapAccessDeserializer<A> {
    /// Construct a new `MapAccessDeserializer<A>`.
    pub fn new(map: A) -> Self {
        MapAccessDeserializer { map: map }
    }
}

impl<'de, A> de::Deserializer<'de> for MapAccessDeserializer<A>
where
    A: de::MapAccess<'de>,
{
    type Error = A::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self.map)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

////////////////////////////////////////////////////////////////////////////////

mod private {
    use lib::*;

    use de::{self, Unexpected};

    #[derive(Clone, Debug)]
    pub struct UnitOnly<E> {
        marker: PhantomData<E>,
    }

    pub fn unit_only<T, E>(t: T) -> (T, UnitOnly<E>) {
        (
            t,
            UnitOnly {
                marker: PhantomData,
            },
        )
    }

    impl<'de, E> de::VariantAccess<'de> for UnitOnly<E>
    where
        E: de::Error,
    {
        type Error = E;

        fn unit_variant(self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            Err(de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            ))
        }

        fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            Err(de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            ))
        }

        fn struct_variant<V>(
            self,
            _fields: &'static [&'static str],
            _visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            Err(de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            ))
        }
    }

    /// Avoid having to restate the generic types on `MapDeserializer`. The
    /// `Iterator::Item` contains enough information to figure out K and V.
    pub trait Pair {
        type First;
        type Second;
        fn split(self) -> (Self::First, Self::Second);
    }

    impl<A, B> Pair for (A, B) {
        type First = A;
        type Second = B;
        fn split(self) -> (A, B) {
            self
        }
    }

    pub type First<T> = <T as Pair>::First;
    pub type Second<T> = <T as Pair>::Second;
}
