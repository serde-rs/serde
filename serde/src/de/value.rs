//! This module supports deserializing from primitives with the `ValueDeserializer` trait.

#[cfg(feature = "std")]
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, btree_map, btree_set, hash_map,
                       hash_set};
#[cfg(feature = "std")]
use std::borrow::Cow;
#[cfg(feature = "std")]
use std::vec;

#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::{BTreeMap, BTreeSet, Vec, String, btree_map, btree_set, vec};
#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::borrow::Cow;
#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::boxed::Box;
#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::string::ToString;

#[cfg(feature = "std")]
use core::hash::Hash;
#[cfg(feature = "std")]
use std::error;
#[cfg(not(feature = "std"))]
use error;

use core::fmt::{self, Display};
use core::iter::{self, Iterator};
use core::marker::PhantomData;

use de::{self, Expected, SeqVisitor};
use bytes;

///////////////////////////////////////////////////////////////////////////////

/// This represents all the possible errors that can occur using the `ValueDeserializer`.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    err: ErrorImpl,
}

#[cfg(any(feature = "std", feature = "collections"))]
type ErrorImpl = Box<str>;
#[cfg(not(any(feature = "std", feature = "collections")))]
type ErrorImpl = ();

impl de::Error for Error {
    #[cfg(any(feature = "std", feature = "collections"))]
    fn custom<T: Display>(msg: T) -> Self {
        Error { err: msg.to_string().into_boxed_str() }
    }

    #[cfg(not(any(feature = "std", feature = "collections")))]
    fn custom<T: Display>(_msg: T) -> Self {
        Error { err: () }
    }
}

impl Display for Error {
    #[cfg(any(feature = "std", feature = "collections"))]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str(&self.err)
    }

    #[cfg(not(any(feature = "std", feature = "collections")))]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str("Serde deserialization error")
    }
}

impl error::Error for Error {
    #[cfg(any(feature = "std", feature = "collections"))]
    fn description(&self) -> &str {
        &self.err
    }

    #[cfg(not(any(feature = "std", feature = "collections")))]
    fn description(&self) -> &str {
        "Serde deserialization error"
    }
}

///////////////////////////////////////////////////////////////////////////////

/// This trait converts primitive types into a deserializer.
pub trait ValueDeserializer<E: de::Error = Error> {
    /// The actual deserializer type.
    type Deserializer: de::Deserializer<Error = E>;

    /// Convert this value into a deserializer.
    fn into_deserializer(self) -> Self::Deserializer;
}

///////////////////////////////////////////////////////////////////////////////

impl<E> ValueDeserializer<E> for ()
    where E: de::Error
{
    type Deserializer = UnitDeserializer<E>;

    fn into_deserializer(self) -> UnitDeserializer<E> {
        UnitDeserializer { marker: PhantomData }
    }
}

/// A helper deserializer that deserializes a `()`.
pub struct UnitDeserializer<E> {
    marker: PhantomData<E>,
}

impl<E> de::Deserializer for UnitDeserializer<E>
    where E: de::Error
{
    type Error = E;

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        seq_fixed_size bytes map unit_struct newtype_struct tuple_struct struct
        struct_field tuple enum ignored_any byte_buf
    }

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_unit()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_none()
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! primitive_deserializer {
    ($ty:ty, $name:ident, $method:ident $($cast:tt)*) => {
        /// A helper deserializer that deserializes a number.
        pub struct $name<E> {
            value: $ty,
            marker: PhantomData<E>
        }

        impl<E> ValueDeserializer<E> for $ty
            where E: de::Error,
        {
            type Deserializer = $name<E>;

            fn into_deserializer(self) -> $name<E> {
                $name {
                    value: self,
                    marker: PhantomData,
                }
            }
        }

        impl<E> de::Deserializer for $name<E>
            where E: de::Error,
        {
            type Error = E;

            forward_to_deserialize! {
                bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit
                option seq seq_fixed_size bytes map unit_struct newtype_struct
                tuple_struct struct struct_field tuple enum ignored_any byte_buf
            }

            fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where V: de::Visitor,
            {
                visitor.$method(self.value $($cast)*)
            }
        }
    }
}

primitive_deserializer!(bool, BoolDeserializer, visit_bool);
primitive_deserializer!(i8, I8Deserializer, visit_i8);
primitive_deserializer!(i16, I16Deserializer, visit_i16);
primitive_deserializer!(i32, I32Deserializer, visit_i32);
primitive_deserializer!(i64, I64Deserializer, visit_i64);
primitive_deserializer!(isize, IsizeDeserializer, visit_i64 as i64);
primitive_deserializer!(u8, U8Deserializer, visit_u8);
primitive_deserializer!(u16, U16Deserializer, visit_u16);
primitive_deserializer!(u64, U64Deserializer, visit_u64);
primitive_deserializer!(usize, UsizeDeserializer, visit_u64 as u64);
primitive_deserializer!(f32, F32Deserializer, visit_f32);
primitive_deserializer!(f64, F64Deserializer, visit_f64);
primitive_deserializer!(char, CharDeserializer, visit_char);

/// A helper deserializer that deserializes a number.
pub struct U32Deserializer<E> {
    value: u32,
    marker: PhantomData<E>,
}

impl<E> ValueDeserializer<E> for u32
    where E: de::Error
{
    type Deserializer = U32Deserializer<E>;

    fn into_deserializer(self) -> U32Deserializer<E> {
        U32Deserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

impl<E> de::Deserializer for U32Deserializer<E>
    where E: de::Error
{
    type Error = E;

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct struct_field tuple ignored_any byte_buf
    }

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_u32(self.value)
    }

    fn deserialize_enum<V>(self,
                           _name: &str,
                           _variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_enum(self)
    }
}

impl<E> de::EnumVisitor for U32Deserializer<E>
    where E: de::Error
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn visit_variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
        where T: de::DeserializeSeed
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a `&str`.
pub struct StrDeserializer<'a, E> {
    value: &'a str,
    marker: PhantomData<E>,
}

impl<'a, E> ValueDeserializer<E> for &'a str
    where E: de::Error
{
    type Deserializer = StrDeserializer<'a, E>;

    fn into_deserializer(self) -> StrDeserializer<'a, E> {
        StrDeserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

impl<'a, E> de::Deserializer for StrDeserializer<'a, E>
    where E: de::Error
{
    type Error = E;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_str(self.value)
    }

    fn deserialize_enum<V>(self,
                           _name: &str,
                           _variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct struct_field tuple ignored_any byte_buf
    }
}

impl<'a, E> de::EnumVisitor for StrDeserializer<'a, E>
    where E: de::Error
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn visit_variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
        where T: de::DeserializeSeed
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a `String`.
#[cfg(any(feature = "std", feature = "collections"))]
pub struct StringDeserializer<E> {
    value: String,
    marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<E> ValueDeserializer<E> for String
    where E: de::Error
{
    type Deserializer = StringDeserializer<E>;

    fn into_deserializer(self) -> StringDeserializer<E> {
        StringDeserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<E> de::Deserializer for StringDeserializer<E>
    where E: de::Error
{
    type Error = E;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_string(self.value)
    }

    fn deserialize_enum<V>(self,
                           _name: &str,
                           _variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct struct_field tuple ignored_any byte_buf
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'a, E> de::EnumVisitor for StringDeserializer<E>
    where E: de::Error
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn visit_variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
        where T: de::DeserializeSeed
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a `String`.
#[cfg(any(feature = "std", feature = "collections"))]
pub struct CowStrDeserializer<'a, E> {
    value: Cow<'a, str>,
    marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'a, E> ValueDeserializer<E> for Cow<'a, str>
    where E: de::Error
{
    type Deserializer = CowStrDeserializer<'a, E>;

    fn into_deserializer(self) -> CowStrDeserializer<'a, E> {
        CowStrDeserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'a, E> de::Deserializer for CowStrDeserializer<'a, E>
    where E: de::Error
{
    type Error = E;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        match self.value {
            Cow::Borrowed(string) => visitor.visit_str(string),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }

    fn deserialize_enum<V>(self,
                           _name: &str,
                           _variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct struct_field tuple ignored_any byte_buf
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'a, E> de::EnumVisitor for CowStrDeserializer<'a, E>
    where E: de::Error
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn visit_variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
        where T: de::DeserializeSeed
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a sequence.
pub struct SeqDeserializer<I, E> {
    iter: iter::Fuse<I>,
    count: usize,
    marker: PhantomData<E>,
}

impl<I, E> SeqDeserializer<I, E>
    where I: Iterator,
          E: de::Error
{
    /// Construct a new `SeqDeserializer<I>`.
    pub fn new(iter: I) -> Self {
        SeqDeserializer {
            iter: iter.fuse(),
            count: 0,
            marker: PhantomData,
        }
    }

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
            Err(de::Error::invalid_length(self.count + remaining, &ExpectedInSeq(self.count)))
        }
    }
}

impl<I, T, E> de::Deserializer for SeqDeserializer<I, E>
    where I: Iterator<Item = T>,
          T: ValueDeserializer<E>,
          E: de::Error
{
    type Error = E;

    fn deserialize<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        let v = try!(visitor.visit_seq(&mut self));
        try!(self.end());
        Ok(v)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct struct_field tuple enum ignored_any byte_buf
    }
}

impl<I, T, E> de::SeqVisitor for SeqDeserializer<I, E>
    where I: Iterator<Item = T>,
          T: ValueDeserializer<E>,
          E: de::Error
{
    type Error = E;

    fn visit_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>, Self::Error>
        where V: de::DeserializeSeed
    {
        match self.iter.next() {
            Some(value) => {
                self.count += 1;
                seed.deserialize(value.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
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

///////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "collections"))]
impl<T, E> ValueDeserializer<E> for Vec<T>
    where T: ValueDeserializer<E>,
          E: de::Error
{
    type Deserializer = SeqDeserializer<vec::IntoIter<T>, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.into_iter())
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<T, E> ValueDeserializer<E> for BTreeSet<T>
    where T: ValueDeserializer<E> + Eq + Ord,
          E: de::Error
{
    type Deserializer = SeqDeserializer<btree_set::IntoIter<T>, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.into_iter())
    }
}

#[cfg(feature = "std")]
impl<T, E> ValueDeserializer<E> for HashSet<T>
    where T: ValueDeserializer<E> + Eq + Hash,
          E: de::Error
{
    type Deserializer = SeqDeserializer<hash_set::IntoIter<T>, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.into_iter())
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a sequence using a `SeqVisitor`.
pub struct SeqVisitorDeserializer<V_, E> {
    visitor: V_,
    marker: PhantomData<E>,
}

impl<V_, E> SeqVisitorDeserializer<V_, E>
    where V_: de::SeqVisitor<Error = E>,
          E: de::Error
{
    /// Construct a new `SeqVisitorDeserializer<V_, E>`.
    pub fn new(visitor: V_) -> Self {
        SeqVisitorDeserializer {
            visitor: visitor,
            marker: PhantomData,
        }
    }
}

impl<V_, E> de::Deserializer for SeqVisitorDeserializer<V_, E>
    where V_: de::SeqVisitor<Error = E>,
          E: de::Error
{
    type Error = E;

    fn deserialize<V: de::Visitor>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(self.visitor)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct struct_field tuple enum ignored_any byte_buf
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a map.
pub struct MapDeserializer<I, E>
    where I: Iterator,
          I::Item: private::Pair,
          <I::Item as private::Pair>::First: ValueDeserializer<E>,
          <I::Item as private::Pair>::Second: ValueDeserializer<E>,
          E: de::Error
{
    iter: iter::Fuse<I>,
    value: Option<<I::Item as private::Pair>::Second>,
    count: usize,
    marker: PhantomData<E>,
}

impl<I, E> MapDeserializer<I, E>
    where I: Iterator,
          I::Item: private::Pair,
          <I::Item as private::Pair>::First: ValueDeserializer<E>,
          <I::Item as private::Pair>::Second: ValueDeserializer<E>,
          E: de::Error
{
    /// Construct a new `MapDeserializer<I, K, V, E>`.
    pub fn new(iter: I) -> Self {
        MapDeserializer {
            iter: iter.fuse(),
            value: None,
            count: 0,
            marker: PhantomData,
        }
    }

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
            Err(de::Error::invalid_length(self.count + remaining, &ExpectedInMap(self.count)))
        }
    }

    fn next_pair
        (&mut self)
         -> Option<(<I::Item as private::Pair>::First, <I::Item as private::Pair>::Second)> {
        match self.iter.next() {
            Some(kv) => {
                self.count += 1;
                Some(private::Pair::split(kv))
            }
            None => None,
        }
    }
}

impl<I, E> de::Deserializer for MapDeserializer<I, E>
    where I: Iterator,
          I::Item: private::Pair,
          <I::Item as private::Pair>::First: ValueDeserializer<E>,
          <I::Item as private::Pair>::Second: ValueDeserializer<E>,
          E: de::Error
{
    type Error = E;

    fn deserialize<V_>(mut self, visitor: V_) -> Result<V_::Value, Self::Error>
        where V_: de::Visitor
    {
        let value = try!(visitor.visit_map(&mut self));
        try!(self.end());
        Ok(value)
    }

    fn deserialize_seq<V_>(mut self, visitor: V_) -> Result<V_::Value, Self::Error>
        where V_: de::Visitor
    {
        let value = try!(visitor.visit_seq(&mut self));
        try!(self.end());
        Ok(value)
    }

    fn deserialize_seq_fixed_size<V_>(self,
                                      _len: usize,
                                      visitor: V_)
                                      -> Result<V_::Value, Self::Error>
        where V_: de::Visitor
    {
        self.deserialize_seq(visitor)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        bytes map unit_struct newtype_struct tuple_struct struct struct_field
        tuple enum ignored_any byte_buf
    }
}

impl<I, E> de::MapVisitor for MapDeserializer<I, E>
    where I: Iterator,
          I::Item: private::Pair,
          <I::Item as private::Pair>::First: ValueDeserializer<E>,
          <I::Item as private::Pair>::Second: ValueDeserializer<E>,
          E: de::Error
{
    type Error = E;

    fn visit_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: de::DeserializeSeed
    {
        match self.next_pair() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn visit_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
        where T: de::DeserializeSeed
    {
        let value = self.value.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let value = value.expect("MapVisitor::visit_value called before visit_key");
        seed.deserialize(value.into_deserializer())
    }

    fn visit_seed<TK, TV>(&mut self,
                          kseed: TK,
                          vseed: TV)
                          -> Result<Option<(TK::Value, TV::Value)>, Self::Error>
        where TK: de::DeserializeSeed,
              TV: de::DeserializeSeed
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I, E> de::SeqVisitor for MapDeserializer<I, E>
    where I: Iterator,
          I::Item: private::Pair,
          <I::Item as private::Pair>::First: ValueDeserializer<E>,
          <I::Item as private::Pair>::Second: ValueDeserializer<E>,
          E: de::Error
{
    type Error = E;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: de::DeserializeSeed
    {
        match self.next_pair() {
            Some((k, v)) => {
                let de = PairDeserializer(k, v, PhantomData);
                seed.deserialize(de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// Used in the `impl SeqVisitor for MapDeserializer` to visit the map as a
// sequence of pairs.
struct PairDeserializer<A, B, E>(A, B, PhantomData<E>);

impl<A, B, E> de::Deserializer for PairDeserializer<A, B, E>
    where A: ValueDeserializer<E>,
          B: ValueDeserializer<E>,
          E: de::Error
{
    type Error = E;

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        bytes map unit_struct newtype_struct tuple_struct struct struct_field
        tuple enum ignored_any byte_buf
    }

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        let mut pair_visitor = PairVisitor(Some(self.0), Some(self.1), PhantomData);
        let pair = try!(visitor.visit_seq(&mut pair_visitor));
        if pair_visitor.1.is_none() {
            Ok(pair)
        } else {
            let remaining = pair_visitor.size_hint().0;
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(de::Error::invalid_length(2, &ExpectedInSeq(2 - remaining)))
        }
    }

    fn deserialize_seq_fixed_size<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
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

impl<A, B, E> de::SeqVisitor for PairVisitor<A, B, E>
    where A: ValueDeserializer<E>,
          B: ValueDeserializer<E>,
          E: de::Error
{
    type Error = E;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: de::DeserializeSeed
    {
        if let Some(k) = self.0.take() {
            seed.deserialize(k.into_deserializer()).map(Some)
        } else if let Some(v) = self.1.take() {
            seed.deserialize(v.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = if self.0.is_some() {
            2
        } else if self.1.is_some() {
            1
        } else {
            0
        };
        (len, Some(len))
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

///////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "collections"))]
impl<K, V, E> ValueDeserializer<E> for BTreeMap<K, V>
    where K: ValueDeserializer<E> + Eq + Ord,
          V: ValueDeserializer<E>,
          E: de::Error
{
    type Deserializer = MapDeserializer<btree_map::IntoIter<K, V>, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapDeserializer::new(self.into_iter())
    }
}

#[cfg(feature = "std")]
impl<K, V, E> ValueDeserializer<E> for HashMap<K, V>
    where K: ValueDeserializer<E> + Eq + Hash,
          V: ValueDeserializer<E>,
          E: de::Error
{
    type Deserializer = MapDeserializer<hash_map::IntoIter<K, V>, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapDeserializer::new(self.into_iter())
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a map using a `MapVisitor`.
pub struct MapVisitorDeserializer<V_, E> {
    visitor: V_,
    marker: PhantomData<E>,
}

impl<V_, E> MapVisitorDeserializer<V_, E>
    where V_: de::MapVisitor<Error = E>,
          E: de::Error
{
    /// Construct a new `MapVisitorDeserializer<V_, E>`.
    pub fn new(visitor: V_) -> Self {
        MapVisitorDeserializer {
            visitor: visitor,
            marker: PhantomData,
        }
    }
}

impl<V_, E> de::Deserializer for MapVisitorDeserializer<V_, E>
    where V_: de::MapVisitor<Error = E>,
          E: de::Error
{
    type Error = E;

    fn deserialize<V: de::Visitor>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(self.visitor)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct struct_field tuple enum ignored_any byte_buf
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<'a, E> ValueDeserializer<E> for bytes::Bytes<'a>
    where E: de::Error
{
    type Deserializer = BytesDeserializer<'a, E>;

    fn into_deserializer(self) -> BytesDeserializer<'a, E> {
        BytesDeserializer {
            value: self.into(),
            marker: PhantomData,
        }
    }
}

/// A helper deserializer that deserializes a `&[u8]`.
pub struct BytesDeserializer<'a, E> {
    value: &'a [u8],
    marker: PhantomData<E>,
}

impl<'a, E> de::Deserializer for BytesDeserializer<'a, E>
    where E: de::Error
{
    type Error = E;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_bytes(self.value)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct struct_field tuple enum ignored_any byte_buf
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "collections"))]
impl<E> ValueDeserializer<E> for bytes::ByteBuf
    where E: de::Error
{
    type Deserializer = ByteBufDeserializer<E>;

    fn into_deserializer(self) -> Self::Deserializer {
        ByteBufDeserializer {
            value: self.into(),
            marker: PhantomData,
        }
    }
}

/// A helper deserializer that deserializes a `Vec<u8>`.
#[cfg(any(feature = "std", feature = "collections"))]
pub struct ByteBufDeserializer<E> {
    value: Vec<u8>,
    marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<E> de::Deserializer for ByteBufDeserializer<E>
    where E: de::Error
{
    type Error = E;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_byte_buf(self.value)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct struct_field tuple enum ignored_any byte_buf
    }
}

///////////////////////////////////////////////////////////////////////////////

mod private {
    use de::{self, Unexpected};
    use core::marker::PhantomData;

    pub struct UnitOnly<E> {
        marker: PhantomData<E>,
    }

    pub fn unit_only<T, E>(t: T) -> (T, UnitOnly<E>) {
        (t, UnitOnly { marker: PhantomData })
    }

    impl<E> de::VariantVisitor for UnitOnly<E>
        where E: de::Error
    {
        type Error = E;

        fn visit_unit(self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn visit_newtype_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
            where T: de::DeserializeSeed
        {
            Err(de::Error::invalid_type(Unexpected::UnitVariant, &"newtype variant"))
        }

        fn visit_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
            where V: de::Visitor
        {
            Err(de::Error::invalid_type(Unexpected::UnitVariant, &"tuple variant"))
        }

        fn visit_struct<V>(self,
                           _fields: &'static [&'static str],
                           _visitor: V)
                           -> Result<V::Value, Self::Error>
            where V: de::Visitor
        {
            Err(de::Error::invalid_type(Unexpected::UnitVariant, &"struct variant"))
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
}
