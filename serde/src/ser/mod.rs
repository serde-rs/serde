//! Generic serialization framework.

#[cfg(feature = "std")]
use std::error;
#[cfg(not(feature = "std"))]
use error;

#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::String;

pub mod impls;

///////////////////////////////////////////////////////////////////////////////

/// `Error` is a trait that allows a `Serialize` to generically create a
/// `Serializer` error.
pub trait Error: Sized + error::Error {
    /// Raised when there is general error when deserializing a type.
    #[cfg(any(feature = "std", feature = "collections"))]
    fn custom<T: Into<String>>(msg: T) -> Self;

    /// Raised when there is general error when deserializing a type.
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
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer;
}

///////////////////////////////////////////////////////////////////////////////

/// A trait that describes a type that can serialize a stream of values into the underlying format.
pub trait Serializer {
    /// The error type that can be returned if some error occurs during serialization.
    type Error: Error;

    /// A state object that is initialized by `serialize_seq`, passed to
    /// `serialize_seq_elt`, and consumed by `serialize_seq_end`. Use `()` if no
    /// state is required.
    type SeqState;
    /// A state object that is initialized by `serialize_tuple`, passed to
    /// `serialize_tuple_elt`, and consumed by `serialize_tuple_end`. Use `()`
    /// if no state is required.
    type TupleState;
    /// A state object that is initialized by `serialize_tuple_struct`, passed
    /// to `serialize_tuple_struct_elt`, and consumed by
    /// `serialize_tuple_struct_end`. Use `()` if no state is required.
    type TupleStructState;
    /// A state object that is initialized by `serialize_tuple_variant`, passed
    /// to `serialize_tuple_variant_elt`, and consumed by
    /// `serialize_tuple_variant_end`. Use `()` if no state is required.
    type TupleVariantState;
    /// A state object that is initialized by `serialize_map`, passed to
    /// `serialize_map_elt`, and consumed by `serialize_map_end`. Use `()` if no
    /// state is required.
    type MapState;
    /// A state object that is initialized by `serialize_struct`, passed to
    /// `serialize_struct_elt`, and consumed by `serialize_struct_end`. Use `()`
    /// if no state is required.
    type StructState;
    /// A state object that is initialized by `serialize_struct_variant`, passed
    /// to `serialize_struct_variant_elt`, and consumed by
    /// `serialize_struct_variant_end`. Use `()` if no state is required.
    type StructVariantState;

    /// Serializes a `bool` value.
    fn serialize_bool(&mut self, v: bool) -> Result<(), Self::Error>;

    /// Serializes an `isize` value. If the format does not differentiate
    /// between `isize` and `i64`, a reasonable implementation would be to cast
    /// the value to `i64` and forward to `serialize_i64`.
    fn serialize_isize(&mut self, v: isize) -> Result<(), Self::Error>;

    /// Serializes an `i8` value. If the format does not differentiate between
    /// `i8` and `i64`, a reasonable implementation would be to cast the value
    /// to `i64` and forward to `serialize_i64`.
    fn serialize_i8(&mut self, v: i8) -> Result<(), Self::Error>;

    /// Serializes an `i16` value. If the format does not differentiate between
    /// `i16` and `i64`, a reasonable implementation would be to cast the value
    /// to `i64` and forward to `serialize_i64`.
    fn serialize_i16(&mut self, v: i16) -> Result<(), Self::Error>;

    /// Serializes an `i32` value. If the format does not differentiate between
    /// `i32` and `i64`, a reasonable implementation would be to cast the value
    /// to `i64` and forward to `serialize_i64`.
    fn serialize_i32(&mut self, v: i32) -> Result<(), Self::Error>;

    /// Serializes an `i64` value.
    fn serialize_i64(&mut self, v: i64) -> Result<(), Self::Error>;

    /// Serializes a `usize` value. If the format does not differentiate between
    /// `usize` and `u64`, a reasonable implementation would be to cast the
    /// value to `u64` and forward to `serialize_u64`.
    fn serialize_usize(&mut self, v: usize) -> Result<(), Self::Error>;

    /// Serializes a `u8` value. If the format does not differentiate between
    /// `u8` and `u64`, a reasonable implementation would be to cast the value
    /// to `u64` and forward to `serialize_u64`.
    fn serialize_u8(&mut self, v: u8) -> Result<(), Self::Error>;

    /// Serializes a `u16` value. If the format does not differentiate between
    /// `u16` and `u64`, a reasonable implementation would be to cast the value
    /// to `u64` and forward to `serialize_u64`.
    fn serialize_u16(&mut self, v: u16) -> Result<(), Self::Error>;

    /// Serializes a `u32` value. If the format does not differentiate between
    /// `u32` and `u64`, a reasonable implementation would be to cast the value
    /// to `u64` and forward to `serialize_u64`.
    fn serialize_u32(&mut self, v: u32) -> Result<(), Self::Error>;

    /// `Serializes a `u64` value.
    fn serialize_u64(&mut self, v: u64) -> Result<(), Self::Error>;

    /// Serializes an `f32` value. If the format does not differentiate between
    /// `f32` and `f64`, a reasonable implementation would be to cast the value
    /// to `f64` and forward to `serialize_f64`.
    fn serialize_f32(&mut self, v: f32) -> Result<(), Self::Error>;

    /// Serializes an `f64` value.
    fn serialize_f64(&mut self, v: f64) -> Result<(), Self::Error>;

    /// Serializes a character.
    fn serialize_char(&mut self, v: char) -> Result<(), Self::Error>;

    /// Serializes a `&str`.
    fn serialize_str(&mut self, value: &str) -> Result<(), Self::Error>;

    /// Enables serializers to serialize byte slices more compactly or more
    /// efficiently than other types of slices. If no efficient implementation
    /// is available, a reasonable implementation would be to forward to
    /// `serialize_seq`.
    fn serialize_bytes(&mut self, value: &[u8]) -> Result<(), Self::Error>;

    /// Serializes a `()` value.
    fn serialize_unit(&mut self) -> Result<(), Self::Error>;

    /// Serializes a unit struct value. A reasonable implementation would be to
    /// forward to `serialize_unit`.
    fn serialize_unit_struct(
        &mut self,
        name: &str,
    ) -> Result<(), Self::Error>;

    /// Serializes a unit variant, otherwise known as a variant with no
    /// arguments. A reasonable implementation would be to forward to
    /// `serialize_unit`.
    fn serialize_unit_variant(
        &mut self,
        name: &str,
        variant_index: usize,
        variant: &str,
    ) -> Result<(), Self::Error>;

    /// Allows a tuple struct with a single element, also known as a newtype
    /// struct, to be more efficiently serialized than a tuple struct with
    /// multiple items. A reasonable implementation would be to forward to
    /// `serialize_tuple_struct`.
    fn serialize_newtype_struct<T: Serialize>(
        &mut self,
        name: &str,
        value: T,
    ) -> Result<(), Self::Error>;

    /// Allows a variant with a single item to be more efficiently serialized
    /// than a variant with multiple items. A reasonable implementation would be
    /// to forward to `serialize_tuple_variant`.
    fn serialize_newtype_variant<T: Serialize>(
        &mut self,
        name: &str,
        variant_index: usize,
        variant: &str,
        value: T,
    ) -> Result<(), Self::Error>;

    /// Serializes a `None` value.
    fn serialize_none(&mut self) -> Result<(), Self::Error>;

    /// Serializes a `Some(...)` value.
    fn serialize_some<T: Serialize>(
        &mut self,
        value: T,
    ) -> Result<(), Self::Error>;

    /// Begins to serialize a sequence. This call must be followed by zero or
    /// more calls to `serialize_seq_elt`, then a call to `serialize_seq_end`.
    fn serialize_seq(
        &mut self,
        len: Option<usize>,
    ) -> Result<Self::SeqState, Self::Error>;

    /// Serializes a sequence element. Must have previously called
    /// `serialize_seq`.
    fn serialize_seq_elt<T: Serialize>(
        &mut self,
        state: &mut Self::SeqState,
        value: T,
    ) -> Result<(), Self::Error>;

    /// Finishes serializing a sequence.
    fn serialize_seq_end(
        &mut self,
        state: Self::SeqState,
    ) -> Result<(), Self::Error>;

    /// Begins to serialize a sequence whose length will be known at
    /// deserialization time. This call must be followed by zero or more calls
    /// to `serialize_seq_elt`, then a call to `serialize_seq_end`. A reasonable
    /// implementation would be to forward to `serialize_seq`.
    fn serialize_seq_fixed_size(
        &mut self,
        size: usize,
    ) -> Result<Self::SeqState, Self::Error>;

    /// Begins to serialize a tuple. This call must be followed by zero or more
    /// calls to `serialize_tuple_elt`, then a call to `serialize_tuple_end`. A
    /// reasonable implementation would be to forward to `serialize_seq`.
    fn serialize_tuple(
        &mut self,
        len: usize,
    ) -> Result<Self::TupleState, Self::Error>;

    /// Serializes a tuple element. Must have previously called
    /// `serialize_tuple`.
    fn serialize_tuple_elt<T: Serialize>(
        &mut self,
        state: &mut Self::TupleState,
        value: T,
    ) -> Result<(), Self::Error>;

    /// Finishes serializing a tuple.
    fn serialize_tuple_end(
        &mut self,
        state: Self::TupleState,
    ) -> Result<(), Self::Error>;

    /// Begins to serialize a tuple struct. This call must be followed by zero
    /// or more calls to `serialize_tuple_struct_elt`, then a call to
    /// `serialize_tuple_struct_end`. A reasonable implementation would be to
    /// forward to `serialize_tuple`.
    fn serialize_tuple_struct(
        &mut self,
        name: &str,
        len: usize,
    ) -> Result<Self::TupleStructState, Self::Error>;

    /// Serializes a tuple struct element. Must have previously called
    /// `serialize_tuple_struct`.
    fn serialize_tuple_struct_elt<T: Serialize>(
        &mut self,
        state: &mut Self::TupleStructState,
        value: T,
    ) -> Result<(), Self::Error>;

    /// Finishes serializing a tuple struct.
    fn serialize_tuple_struct_end(
        &mut self,
        state: Self::TupleStructState,
    ) -> Result<(), Self::Error>;

    /// Begins to serialize a tuple variant. This call must be followed by zero
    /// or more calls to `serialize_tuple_variant_elt`, then a call to
    /// `serialize_tuple_variant_end`. A reasonable implementation would be to
    /// forward to `serialize_tuple_struct`.
    fn serialize_tuple_variant(
        &mut self,
        name: &str,
        variant_index: usize,
        variant: &str,
        len: usize,
    ) -> Result<Self::TupleVariantState, Self::Error>;

    /// Serializes a tuple variant element. Must have previously called
    /// `serialize_tuple_variant`.
    fn serialize_tuple_variant_elt<T: Serialize>(
        &mut self,
        state: &mut Self::TupleVariantState,
        value: T,
    ) -> Result<(), Self::Error>;

    /// Finishes serializing a tuple variant.
    fn serialize_tuple_variant_end(
        &mut self,
        state: Self::TupleVariantState,
    ) -> Result<(), Self::Error>;

    /// Begins to serialize a map. This call must be followed by zero or more
    /// calls to `serialize_map_elt`, then a call to `serialize_map_end`.
    fn serialize_map(
        &mut self,
        len: Option<usize>,
    ) -> Result<Self::MapState, Self::Error>;

    /// Serialize a map element. Must have previously called `serialize_map`.
    fn serialize_map_elt<K: Serialize, V: Serialize>(
        &mut self,
        state: &mut Self::MapState,
        key: K,
        value: V,
    ) -> Result<(), Self::Error>;

    /// Finishes serializing a map.
    fn serialize_map_end(
        &mut self,
        state: Self::MapState,
    ) -> Result<(), Self::Error>;

    /// Begins to serialize a struct. This call must be followed by zero or more
    /// calls to `serialize_struct_elt`, then a call to `serialize_struct_end`.
    fn serialize_struct(
        &mut self,
        name: &str,
        len: usize,
    ) -> Result<Self::StructState, Self::Error>;

    /// Serializes a struct field. Must have previously called
    /// `serialize_struct`.
    fn serialize_struct_elt<V: Serialize>(
        &mut self,
        state: &mut Self::StructState,
        key: &str,
        value: V,
    ) -> Result<(), Self::Error>;

    /// Finishes serializing a struct.
    fn serialize_struct_end(
        &mut self,
        state: Self::StructState,
    ) -> Result<(), Self::Error>;

    /// Begins to serialize a struct variant. This call must be followed by zero
    /// or more calls to `serialize_struct_variant_elt`, then a call to
    /// `serialize_struct_variant_end`.
    fn serialize_struct_variant(
        &mut self,
        name: &str,
        variant_index: usize,
        variant: &str,
        len: usize,
    ) -> Result<Self::StructVariantState, Self::Error>;

    /// Serialize a struct variant element. Must have previously called
    /// `serialize_struct_variant`.
    fn serialize_struct_variant_elt<V: Serialize>(
        &mut self,
        state: &mut Self::StructVariantState,
        key: &str,
        value: V,
    ) -> Result<(), Self::Error>;

    /// Finishes serializing a struct variant.
    fn serialize_struct_variant_end(
        &mut self,
        state: Self::StructVariantState,
    ) -> Result<(), Self::Error>;
}
