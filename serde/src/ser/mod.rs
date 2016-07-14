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
    /// A state object that is returned from `serialize_seq` and needs to be re-inserted into
    /// `serialize_seq_end` when the serialization of the sequence is done
    type SeqState;
    /// A state object that is returned from `serialize_map` and needs to be re-inserted into
    /// `serialize_map_end` when the serialization of the map is done
    type MapState;

    /// Serializes a `bool` value.
    fn serialize_bool(&mut self, v: bool) -> Result<(), Self::Error>;

    /// Serializes a `isize` value. By default it casts the value to a `i64` and
    /// passes it to the `serialize_i64` method.
    #[inline]
    fn serialize_isize(&mut self, v: isize) -> Result<(), Self::Error> {
        self.serialize_i64(v as i64)
    }

    /// Serializes a `i8` value. By default it casts the value to a `i64` and
    /// passes it to the `serialize_i64` method.
    #[inline]
    fn serialize_i8(&mut self, v: i8) -> Result<(), Self::Error> {
        self.serialize_i64(v as i64)
    }

    /// Serializes a `i16` value. By default it casts the value to a `i64` and
    /// passes it to the `serialize_i64` method.
    #[inline]
    fn serialize_i16(&mut self, v: i16) -> Result<(), Self::Error> {
        self.serialize_i64(v as i64)
    }

    /// Serializes a `i32` value. By default it casts the value to a `i64` and
    /// passes it to the `serialize_i64` method.
    #[inline]
    fn serialize_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        self.serialize_i64(v as i64)
    }

    /// Serializes a `i64` value.
    #[inline]
    fn serialize_i64(&mut self, v: i64) -> Result<(), Self::Error>;

    /// Serializes a `usize` value. By default it casts the value to a `u64` and
    /// passes it to the `serialize_u64` method.
    #[inline]
    fn serialize_usize(&mut self, v: usize) -> Result<(), Self::Error> {
        self.serialize_u64(v as u64)
    }

    /// Serializes a `u8` value. By default it casts the value to a `u64` and passes
    /// it to the `serialize_u64` method.
    #[inline]
    fn serialize_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        self.serialize_u64(v as u64)
    }

    /// Serializes a `u32` value. By default it casts the value to a `u64` and passes
    /// it to the `serialize_u64` method.
    #[inline]
    fn serialize_u16(&mut self, v: u16) -> Result<(), Self::Error> {
        self.serialize_u64(v as u64)
    }

    /// Serializes a `u32` value. By default it casts the value to a `u64` and passes
    /// it to the `serialize_u64` method.
    #[inline]
    fn serialize_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        self.serialize_u64(v as u64)
    }

    /// `Serializes a `u64` value.
    #[inline]
    fn serialize_u64(&mut self, v: u64) -> Result<(), Self::Error>;

    /// Serializes a `f32` value. By default it casts the value to a `f64` and passes
    /// it to the `serialize_f64` method.
    #[inline]
    fn serialize_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        self.serialize_f64(v as f64)
    }

    /// Serializes a `f64` value.
    fn serialize_f64(&mut self, v: f64) -> Result<(), Self::Error>;

    /// Serializes a character. By default it serializes it as a `&str` containing a
    /// single character.
    #[inline]
    fn serialize_char(&mut self, v: char) -> Result<(), Self::Error> {
        self.serialize_str(::utils::encode_utf8(v).as_str())
    }

    /// Serializes a `&str`.
    fn serialize_str(&mut self, value: &str) -> Result<(), Self::Error>;

    /// Enables those serialization formats that support serializing
    /// byte slices separately from generic arrays. By default it serializes as a regular array.
    #[inline]
    fn serialize_bytes(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        let state = try!(self.serialize_seq(Some(value.len())));
        for b in value.iter() {
            try!(self.serialize_seq_elt(b));
        }
        self.serialize_seq_end(Some(value.len()), state)
    }

    /// Serializes a `()` value.
    fn serialize_unit(&mut self) -> Result<(), Self::Error>;

    /// Serializes a unit struct value.
    ///
    /// By default, unit structs are serialized as a `()`.
    #[inline]
    fn serialize_unit_struct(&mut self, _name: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    /// Serializes a unit variant, otherwise known as a variant with no arguments.
    ///
    /// By default, unit variants are serialized as a `()`.
    #[inline]
    fn serialize_unit_variant(&mut self,
                              _name: &'static str,
                              _variant_index: usize,
                              _variant: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    /// Allows a tuple struct with a single element, also known as a
    /// newtyped value, to be more efficiently serialized than a tuple struct with multiple items.
    /// By default it just serializes the value as a tuple struct sequence.
    #[inline]
    fn serialize_newtype_struct<T>(&mut self,
                                   name: &'static str,
                                   value: T) -> Result<(), Self::Error>
        where T: Serialize,
    {
        let state = try!(self.serialize_tuple_struct(name, 1));
        try!(self.serialize_tuple_struct_elt(value));
        self.serialize_tuple_struct_end(name, 1, state)
    }

    /// Allows a variant with a single item to be more efficiently
    /// serialized than a variant with multiple items. By default it just serializes the value as a
    /// tuple variant sequence.
    #[inline]
    fn serialize_newtype_variant<T>(&mut self,
                                    name: &'static str,
                                    variant_index: usize,
                                    variant: &'static str,
                                    value: T) -> Result<(), Self::Error>
        where T: Serialize,
    {
        let state = try!(self.serialize_tuple_variant(name, variant_index, variant, 1));
        try!(self.serialize_tuple_variant_elt(value));
        self.serialize_tuple_variant_end(name, variant_index, variant, 1, state)
    }

    /// Serializes a `None` value..serialize
    fn serialize_none(&mut self) -> Result<(), Self::Error>;

    /// Serializes a `Some(...)` value.
    fn serialize_some<V>(&mut self, value: V) -> Result<(), Self::Error>
        where V: Serialize;

    /// Serializes a sequence.
    ///
    /// Callees of this method need to construct a `SeqVisitor`, which iterates through each item
    /// in the sequence.
    fn serialize_seq(&mut self, len: Option<usize>) -> Result<Self::SeqState, Self::Error>;

    /// Serializes a sequence element.
    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize;

    /// Finish serializing a sequence.
    fn serialize_seq_end(&mut self, len: Option<usize>, state: Self::SeqState) -> Result<(), Self::Error>;

    /// Serializes a tuple.
    ///
    /// By default this serializes a tuple as a sequence.
    #[inline]
    fn serialize_tuple(&mut self, len: usize) -> Result<Self::SeqState, Self::Error>
    {
        self.serialize_seq(Some(len))
    }

    /// Serializes a tuple element.
    ///
    /// By default, tuples are serialized as a sequence.
    #[inline]
    fn serialize_tuple_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize {
        self.serialize_seq_elt(value)
    }

    /// Finishes serialization of a tuple.
    ///
    /// By default, tuples are serialized as a sequence.
    #[inline]
    fn serialize_tuple_end(&mut self, len: usize, state: Self::SeqState) -> Result<(), Self::Error> {
        self.serialize_seq_end(Some(len), state)
    }

    /// Serializes a fixed-size array.
    ///
    /// By default this serializes an array as a sequence.
    #[inline]
    fn serialize_fixed_size_array(&mut self, size: usize) -> Result<Self::SeqState, Self::Error>
    {
        self.serialize_seq(Some(size))
    }

    /// Serializes a tuple struct.
    ///
    /// By default, tuple structs are serialized as a tuple.
    #[inline]
    fn serialize_tuple_struct(&mut self,
                              _name: &'static str,
                              len: usize,
                             ) -> Result<Self::SeqState, Self::Error>
    {
        self.serialize_tuple(len)
    }

    /// Serializes a tuple struct element.
    ///
    /// By default, tuple struct elements are serialized as a tuple element.
    #[inline]
    fn serialize_tuple_struct_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize
    {
        self.serialize_tuple_elt(value)
    }

    /// Finishes serialization of a tuple struct.
    ///
    /// By default, tuple structs are serialized as a sequence.
    #[inline]
    fn serialize_tuple_struct_end(&mut self,
                              _name: &'static str,
                              len: usize,
                              state: Self::SeqState,
                              ) -> Result<(), Self::Error> {
        self.serialize_tuple_end(len, state)
    }

    /// Serializes a tuple variant.
    ///
    /// By default, tuple variants are serialized as a tuple struct.
    #[inline]
    fn serialize_tuple_variant(&mut self,
                               _name: &'static str,
                               _variant_index: usize,
                               variant: &'static str,
                               len: usize,
                               ) -> Result<Self::SeqState, Self::Error>
    {
        self.serialize_tuple_struct(variant, len)
    }

    /// Serializes a tuple variant element.
    ///
    /// By default, tuple variants are serialized as tuples.
    #[inline]
    fn serialize_tuple_variant_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize
    {
        self.serialize_tuple_struct_elt(value)
    }

    /// Finishes serialization of a tuple variant.
    ///
    /// By default, tuple variants are serialized as tuples.
    #[inline]
    fn serialize_tuple_variant_end(&mut self,
                               _name: &'static str,
                               _variant_index: usize,
                               variant: &'static str,
                               len: usize,
                               state: Self::SeqState,
                               ) -> Result<(), Self::Error> {
        self.serialize_tuple_struct_end(variant, len, state)
    }

    /// Serialize a map.
    fn serialize_map(&mut self, len: Option<usize>) -> Result<Self::MapState, Self::Error>;

    /// Serialize a map element
    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize;

    /// Finishes serializing a map
    fn serialize_map_end(&mut self, len: Option<usize>, state: Self::MapState) -> Result<(), Self::Error>;

    /// Serializes a struct.
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    #[inline]
    fn serialize_struct(&mut self,
                        _name: &'static str,
                        len: usize,
                        ) -> Result<Self::MapState, Self::Error>
    {
        self.serialize_map(Some(len))
    }

    /// Serialize a struct field
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    fn serialize_struct_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize {
        self.serialize_map_elt(key, value)
    }

    /// Finishes serializing a struct
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    fn serialize_struct_end(&mut self,
                        _name: &'static str,
                        len: usize,
                        state: Self::MapState,
                        ) -> Result<(), Self::Error> {
        self.serialize_map_end(Some(len), state)
    }

    /// Serializes a struct variant.
    ///
    /// By default, struct variants are serialized as a struct.
    #[inline]
    fn serialize_struct_variant(&mut self,
                                   _name: &'static str,
                                   _variant_index: usize,
                                   variant: &'static str,
                                   len: usize,
                               ) -> Result<Self::MapState, Self::Error>
    {
        self.serialize_struct(variant, len)
    }

    /// Serialize a struct variant element
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    fn serialize_struct_variant_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize {
        self.serialize_struct_elt(key, value)
    }

    /// Finishes serializing a struct variant
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    fn serialize_struct_variant_end(&mut self,
                                   _name: &'static str,
                                   _variant_index: usize,
                                   variant: &'static str,
                                   len: usize,
                                   state: Self::MapState,
                               ) -> Result<(), Self::Error> {
        self.serialize_struct_end(variant, len, state)
    }
}
