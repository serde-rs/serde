//! Generic serialization framework.
//! # For Developers who want to serialize objects
//! Implement the `Serialize` trait for the type of objects you want to serialize. Call methods of
//! the `serializer` object. For which methods to call and how to do so, look at the documentation
//! of the `Serializer` trait.
//!
//! # For Serialization Format Developers
//! Implement the `Serializer` trait for a structure that contains fields that enable it to write
//! the serialization result to your target. When a method's argument is an object of type
//! `Serialize`, you can either forward the serializer object (`self`) or create a new one,
//! depending on the quirks of your format.

#[cfg(feature = "std")]
use std::error;
#[cfg(not(feature = "std"))]
use error;

#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::String;

#[cfg(feature = "unstable")]
use core::marker::PhantomData;
#[cfg(feature = "unstable")]
use core::cell::RefCell;

pub mod impls;

///////////////////////////////////////////////////////////////////////////////

/// `Error` is a trait that allows a `Serialize` to generically create a
/// `Serializer` error.
pub trait Error: Sized + error::Error {
    /// Raised when there is a general error when serializing a type.
    #[cfg(any(feature = "std", feature = "collections"))]
    fn custom<T: Into<String>>(msg: T) -> Self;

    /// Raised when there is a general error when serializing a type.
    #[cfg(all(not(feature = "std"), not(feature = "collections")))]
    fn custom<T: Into<&'static str>>(msg: T) -> Self;

    /// Raised when a `Serialize` was passed an incorrect value.
    fn invalid_value(msg: &str) -> Self {
        Error::custom(format!("invalid value: {}", msg))
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A trait that describes a type that can be serialized by a `Serializer`.
pub trait Serialize {
    /// Serializes this value into this serializer.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer;
}

///////////////////////////////////////////////////////////////////////////////

/// A trait that describes a type that can serialize a stream of values into the underlying format.
///
/// # For `Serialize` Developers
/// Non-aggregate types like integers and strings can be serialized directly by calling the
/// appropriate function. For Aggregate types there's an initial `serialize_T` method that yields
/// a State object that you should not interact with. For each part of the aggregate there's a
/// `serialize_T_elt` method that allows you to pass values or key/value pairs. The types of the
/// values or the keys may change between calls, but the serialization format may not necessarily
/// accept it. The `serialize_T_elt` method also takes a mutable reference to the state object.
/// Make sure that you always use the same state object and only the state object that was returned
/// by the `serialize_T` method. Finally, when your object is done, call the `serialize_T_end`
/// method and pass the state object by value
///
/// # For Serialization Format Developers
/// If your format has different situations where it accepts different types, create a
/// `Serializer` for each situation. You can create the sub-`Serializer` in one of the aggregate
/// `serialize_T` methods and return it as a state object. Remember to also set the corresponding
/// associated type `TState`. In the `serialize_T_elt` methods you will be given a mutable
/// reference to that state. You do not need to do any additional checks for the correctness of the
/// state object, as it is expected that the user will not modify it. Due to the generic nature
/// of the `Serialize` impls, modifying the object is impossible on stable Rust.
pub trait Serializer {
    /// Trickery to enforce correct use of the `Serialize` trait. Every
    /// `Serializer` should set `Ok = ()`.
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: Error;

    /// Type returned from `serialize_seq` and `serialize_seq_fixed_size` for
    /// serializing the content of the sequence.
    type SerializeSeq: SerializeSeq<Ok=Self::Ok, Error=Self::Error>;

    /// Type returned from `serialize_tuple` for serializing the content of the
    /// tuple.
    type SerializeTuple: SerializeTuple<Ok=Self::Ok, Error=Self::Error>;

    /// Type returned from `serialize_tuple_struct` for serializing the content
    /// of the tuple struct.
    type SerializeTupleStruct: SerializeTupleStruct<Ok=Self::Ok, Error=Self::Error>;

    /// Type returned from `serialize_tuple_variant` for serializing the content
    /// of the tuple variant.
    type SerializeTupleVariant: SerializeTupleVariant<Ok=Self::Ok, Error=Self::Error>;

    /// Type returned from `serialize_map` for serializing the content of the
    /// map.
    type SerializeMap: SerializeMap<Ok=Self::Ok, Error=Self::Error>;

    /// Type returned from `serialize_struct` for serializing the content of the
    /// struct.
    type SerializeStruct: SerializeStruct<Ok=Self::Ok, Error=Self::Error>;

    /// Type returned from `serialize_struct_variant` for serializing the
    /// content of the struct variant.
    type SerializeStructVariant: SerializeStructVariant<Ok=Self::Ok, Error=Self::Error>;

    /// Serializes a `bool` value.
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error>;

    /// Serializes an `isize` value. If the format does not differentiate
    /// between `isize` and `i64`, a reasonable implementation would be to cast
    /// the value to `i64` and forward to `serialize_i64`.
    fn serialize_isize(self, v: isize) -> Result<Self::Ok, Self::Error>;

    /// Serializes an `i8` value. If the format does not differentiate between
    /// `i8` and `i64`, a reasonable implementation would be to cast the value
    /// to `i64` and forward to `serialize_i64`.
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error>;

    /// Serializes an `i16` value. If the format does not differentiate between
    /// `i16` and `i64`, a reasonable implementation would be to cast the value
    /// to `i64` and forward to `serialize_i64`.
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error>;

    /// Serializes an `i32` value. If the format does not differentiate between
    /// `i32` and `i64`, a reasonable implementation would be to cast the value
    /// to `i64` and forward to `serialize_i64`.
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error>;

    /// Serializes an `i64` value.
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error>;

    /// Serializes a `usize` value. If the format does not differentiate between
    /// `usize` and `u64`, a reasonable implementation would be to cast the
    /// value to `u64` and forward to `serialize_u64`.
    fn serialize_usize(self, v: usize) -> Result<Self::Ok, Self::Error>;

    /// Serializes a `u8` value. If the format does not differentiate between
    /// `u8` and `u64`, a reasonable implementation would be to cast the value
    /// to `u64` and forward to `serialize_u64`.
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error>;

    /// Serializes a `u16` value. If the format does not differentiate between
    /// `u16` and `u64`, a reasonable implementation would be to cast the value
    /// to `u64` and forward to `serialize_u64`.
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error>;

    /// Serializes a `u32` value. If the format does not differentiate between
    /// `u32` and `u64`, a reasonable implementation would be to cast the value
    /// to `u64` and forward to `serialize_u64`.
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error>;

    /// `Serializes a `u64` value.
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error>;

    /// Serializes an `f32` value. If the format does not differentiate between
    /// `f32` and `f64`, a reasonable implementation would be to cast the value
    /// to `f64` and forward to `serialize_f64`.
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error>;

    /// Serializes an `f64` value.
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error>;

    /// Serializes a character. If the format does not support characters,
    /// it is reasonable to serialize it as a single element `str` or a `u32`.
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error>;

    /// Serializes a `&str`.
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error>;

    /// Enables serializers to serialize byte slices more compactly or more
    /// efficiently than other types of slices. If no efficient implementation
    /// is available, a reasonable implementation would be to forward to
    /// `serialize_seq`. If forwarded, the implementation looks usually just like this:
    /// ```rust
    /// let mut seq = self.serialize_seq(Some(value.len()))?;
    /// for b in value {
    ///     seq.serialize_elem(b)?;
    /// }
    /// seq.serialize_end()
    /// ```
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error>;

    /// Serializes a `()` value. It's reasonable to just not serialize anything.
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error>;

    /// Serializes a unit struct value. A reasonable implementation would be to
    /// forward to `serialize_unit`.
    fn serialize_unit_struct(
        self,
        name: &'static str,
    ) -> Result<Self::Ok, Self::Error>;

    /// Serializes a unit variant, otherwise known as a variant with no
    /// arguments. A reasonable implementation would be to forward to
    /// `serialize_unit`.
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: usize,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error>;

    /// Allows a tuple struct with a single element, also known as a newtype
    /// struct, to be more efficiently serialized than a tuple struct with
    /// multiple items. A reasonable implementation would be to forward to
    /// `serialize_tuple_struct` or to just serialize the inner value without wrapping.
    fn serialize_newtype_struct<T: Serialize>(
        self,
        name: &'static str,
        value: T,
    ) -> Result<Self::Ok, Self::Error>;

    /// Allows a variant with a single item to be more efficiently serialized
    /// than a variant with multiple items. A reasonable implementation would be
    /// to forward to `serialize_tuple_variant`.
    fn serialize_newtype_variant<T: Serialize>(
        self,
        name: &'static str,
        variant_index: usize,
        variant: &'static str,
        value: T,
    ) -> Result<Self::Ok, Self::Error>;

    /// Serializes a `None` value.
    fn serialize_none(self) -> Result<Self::Ok, Self::Error>;

    /// Serializes a `Some(...)` value.
    fn serialize_some<T: Serialize>(
        self,
        value: T,
    ) -> Result<Self::Ok, Self::Error>;

    /// Begins to serialize a sequence. This call must be followed by zero or
    /// more calls to `serialize_seq_elt`, then a call to `serialize_seq_end`.
    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> Result<Self::SerializeSeq, Self::Error>;

    /// Begins to serialize a sequence whose length will be known at
    /// deserialization time. This call must be followed by zero or more calls
    /// to `serialize_seq_elt`, then a call to `serialize_seq_end`. A reasonable
    /// implementation would be to forward to `serialize_seq`.
    fn serialize_seq_fixed_size(
        self,
        size: usize,
    ) -> Result<Self::SerializeSeq, Self::Error>;

    /// Begins to serialize a tuple. This call must be followed by zero or more
    /// calls to `serialize_tuple_elt`, then a call to `serialize_tuple_end`. A
    /// reasonable implementation would be to forward to `serialize_seq`.
    fn serialize_tuple(
        self,
        len: usize,
    ) -> Result<Self::SerializeTuple, Self::Error>;

    /// Begins to serialize a tuple struct. This call must be followed by zero
    /// or more calls to `serialize_tuple_struct_elt`, then a call to
    /// `serialize_tuple_struct_end`. A reasonable implementation would be to
    /// forward to `serialize_tuple`.
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error>;

    /// Begins to serialize a tuple variant. This call must be followed by zero
    /// or more calls to `serialize_tuple_variant_elt`, then a call to
    /// `serialize_tuple_variant_end`. A reasonable implementation would be to
    /// forward to `serialize_tuple_struct`.
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: usize,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error>;

    /// Begins to serialize a map. This call must be followed by zero or more
    /// calls to `serialize_map_key` and `serialize_map_value`, then a call to
    /// `serialize_map_end`.
    fn serialize_map(
        self,
        len: Option<usize>,
    ) -> Result<Self::SerializeMap, Self::Error>;

    /// Begins to serialize a struct. This call must be followed by zero or more
    /// calls to `serialize_struct_elt`, then a call to `serialize_struct_end`.
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error>;

    /// Begins to serialize a struct variant. This call must be followed by zero
    /// or more calls to `serialize_struct_variant_elt`, then a call to
    /// `serialize_struct_variant_end`.
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: usize,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error>;
}

/// Returned from `Serializer::serialize_seq` and
/// `Serializer::serialize_seq_fixed_size`.
pub trait SerializeSeq {
    /// Trickery to enforce correct use of the `Serialize` trait. Every
    /// `SerializeSeq` should set `Ok = ()`.
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: Error;

    /// Serializes a sequence element.
    fn serialize_elem<T: Serialize>(&mut self, value: T) -> Result<(), Self::Error>;

    /// Finishes serializing a sequence.
    fn serialize_end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_tuple`.
pub trait SerializeTuple {
    /// Trickery to enforce correct use of the `Serialize` trait. Every
    /// `SerializeTuple` should set `Ok = ()`.
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: Error;

    /// Serializes a tuple element.
    fn serialize_elem<T: Serialize>(&mut self, value: T) -> Result<(), Self::Error>;

    /// Finishes serializing a tuple.
    fn serialize_end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_tuple_struct`.
pub trait SerializeTupleStruct {
    /// Trickery to enforce correct use of the `Serialize` trait. Every
    /// `SerializeTupleStruct` should set `Ok = ()`.
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: Error;

    /// Serializes a tuple struct element.
    fn serialize_elem<T: Serialize>(&mut self, value: T) -> Result<(), Self::Error>;

    /// Finishes serializing a tuple struct.
    fn serialize_end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_tuple_variant`.
pub trait SerializeTupleVariant {
    /// Trickery to enforce correct use of the `Serialize` trait. Every
    /// `SerializeTupleVariant` should set `Ok = ()`.
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: Error;

    /// Serializes a tuple variant element.
    fn serialize_elem<T: Serialize>(&mut self, value: T) -> Result<(), Self::Error>;

    /// Finishes serializing a tuple variant.
    fn serialize_end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_map`.
pub trait SerializeMap {
    /// Trickery to enforce correct use of the `Serialize` trait. Every
    /// `SerializeMap` should set `Ok = ()`.
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: Error;

    /// Serialize a map key.
    fn serialize_key<T: Serialize>(&mut self, key: T) -> Result<(), Self::Error>;

    /// Serialize a map value.
    fn serialize_value<T: Serialize>(&mut self, value: T) -> Result<(), Self::Error>;

    /// Finishes serializing a map.
    fn serialize_end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_struct`.
pub trait SerializeStruct {
    /// Trickery to enforce correct use of the `Serialize` trait. Every
    /// `SerializeStruct` should set `Ok = ()`.
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: Error;

    /// Serializes a struct field.
    fn serialize_field<V: Serialize>(&mut self, key: &'static str, value: V) -> Result<(), Self::Error>;

    /// Finishes serializing a struct.
    fn serialize_end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_struct_variant`.
pub trait SerializeStructVariant {
    /// Trickery to enforce correct use of the `Serialize` trait. Every
    /// `SerializeStructVariant` should set `Ok = ()`.
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: Error;

    /// Serialize a struct variant element.
    fn serialize_field<V: Serialize>(&mut self, key: &'static str, value: V) -> Result<(), Self::Error>;

    /// Finishes serializing a struct variant.
    fn serialize_end(self) -> Result<Self::Ok, Self::Error>;
}

/// A wrapper type for iterators that implements `Serialize` for iterators whose items implement
/// `Serialize`. Don't use multiple times. Create new versions of this with the `iterator` function
/// every time you want to serialize an iterator.
#[cfg(feature = "unstable")]
pub struct Iterator<I>(RefCell<Option<I>>)
    where <I as IntoIterator>::Item: Serialize,
          I: IntoIterator;

/// Creates a temporary type that can be passed to any function expecting a `Serialize` and will
/// serialize the given iterator as a sequence
#[cfg(feature = "unstable")]
pub fn iterator<I>(iter: I) -> Iterator<I>
    where <I as IntoIterator>::Item: Serialize,
          I: IntoIterator
{
    Iterator(RefCell::new(Some(iter)))
}
