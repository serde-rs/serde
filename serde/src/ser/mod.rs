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

/// A trait that described a type that can serialize elements of a Map
pub trait MapSerializer : Sized {
    /// The error type that can be returned if some error occurs during serialization.
    type Error: Error;

    /// Serializes an element of a map (key-value pair).
    fn serialize_elt<S: ?Sized, K, V>(&mut self, serializer: &mut S, key: K, value: V) -> Result<(), Self::Error>
        where K: Serialize,
              V: Serialize,
              S: Serializer<Error = Self::Error>;

    /// Finish serializing the map
    fn drop<S: ?Sized>(self, serializer: &mut S) -> Result<(), Self::Error>
        where S: Serializer<Error = Self::Error>;
}

/// A trait that described a type that can serialize elements of a Sequence
pub trait SeqSerializer : Sized {
    /// The error type that can be returned if some error occurs during serialization.
    type Error: Error;

    /// Serializes an element of a sequence.
    fn serialize_elt<S: ?Sized, T>(&mut self, serializer: &mut S, value: T) -> Result<(), Self::Error>
        where T: Serialize,
              S: Serializer<Error = Self::Error>;

    /// Finish serializing the map
    fn drop<S: ?Sized>(self, serializer: &mut S) -> Result<(), Self::Error>
        where S: Serializer<Error = Self::Error>;
}

/// A helper type for serializing maps. This is a workaround for HKL associated types
pub struct MapHelper<'a, S: Serializer + 'a + ?Sized>(&'a mut S, S::MapSerializer);

impl<'a, S: Serializer + 'a + ?Sized> MapHelper<'a, S> {
    /// Create a new MapHelper
    pub fn new(s: &'a mut S, ms: S::MapSerializer) -> Self {
        MapHelper(s, ms)
    }
}

impl<'a, S: Serializer + ?Sized> MapHelper<'a, S> {
    /// Serializes an element of a map (key-value pair).
    pub fn serialize_elt<K, V>(&mut self, key: K, value: V) -> Result<(), S::Error>
        where K: Serialize,
              V: Serialize {
        self.1.serialize_elt(self.0, key, value)
    }
    /// Closes the sequence
    pub fn drop(self) -> Result<(), S::Error> {
        self.1.drop(self.0)
    }
}

/// A helper type for serializing sequences. This is a workaround for HKL associated types
pub struct SeqHelper<'a, S: Serializer + 'a + ?Sized>(&'a mut S, S::SeqSerializer);

impl<'a, S: Serializer + 'a + ?Sized> SeqHelper<'a, S> {
    /// Create a new SeqHelper
    pub fn new(s: &'a mut S, ms: S::SeqSerializer) -> Self {
        SeqHelper(s, ms)
    }
}

impl<'a, S: Serializer + ?Sized> SeqHelper<'a, S> {
    /// Serializes an element of a sequence.
    pub fn serialize_elt<V>(&mut self, value: V) -> Result<(), S::Error>
        where V: Serialize {
        self.1.serialize_elt(self.0, value)
    }
    /// Closes the sequence
    pub fn drop(self) -> Result<(), S::Error> {
        self.1.drop(self.0)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A trait that describes a type that can serialize a stream of values into the underlying format.
pub trait Serializer {
    /// The error type that can be returned if some error occurs during serialization.
    type Error: Error;

    /// A sub-type used to serialize sequences
    type SeqSerializer : SeqSerializer<Error = Self::Error>;
    /// A sub-type used to serialize maps
    type MapSerializer : MapSerializer<Error = Self::Error>;

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
        let mut seq_serializer = try!(self.serialize_seq(Some(value.len())));
        for b in value.iter() {
            try!(seq_serializer.serialize_elt(b));
        }
        seq_serializer.drop()
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
        let mut ser = try!(self.serialize_tuple_struct(name, 1));
        try!(ser.serialize_elt((value,)));
        ser.drop()
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
        let mut ser = try!(self.serialize_tuple_variant(name, variant_index, variant, 1));
        try!(ser.serialize_elt((value,)));
        ser.drop()
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
    fn serialize_seq<'a>(&'a mut self, len: Option<usize>) -> Result<SeqHelper<'a, Self>, Self::Error>;

    /// Serializes a sequence element.
    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize;

    /// Finish serializing a sequence.
    fn serialize_seq_end(&mut self) -> Result<(), Self::Error>;

    /// Serializes a tuple.
    ///
    /// By default this serializes a tuple as a sequence.
    #[inline]
    fn serialize_tuple<'a>(&'a mut self, len: usize) -> Result<SeqHelper<'a, Self>, Self::Error>
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
    fn serialize_tuple_end(&mut self) -> Result<(), Self::Error> {
        self.serialize_seq_end()
    }

    /// Serializes a fixed-size array.
    ///
    /// By default this serializes an array as a sequence.
    #[inline]
    fn serialize_fixed_size_array<'a>(&'a mut self, size: usize) -> Result<SeqHelper<'a, Self>, Self::Error>
    {
        self.serialize_seq(Some(size))
    }

    /// Serializes a tuple struct.
    ///
    /// By default, tuple structs are serialized as a tuple.
    #[inline]
    fn serialize_tuple_struct<'a>(&'a mut self,
                                 _name: &'static str,
                                 len: usize,
                                 ) -> Result<SeqHelper<'a, Self>, Self::Error>
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
    fn serialize_tuple_struct_end(&mut self) -> Result<(), Self::Error> {
        self.serialize_tuple_end()
    }

    /// Serializes a tuple variant.
    ///
    /// By default, tuple variants are serialized as a tuple struct.
    #[inline]
    fn serialize_tuple_variant<'a>(&'a mut self,
                                  _name: &'static str,
                                  _variant_index: usize,
                                  variant: &'static str,
                                  len: usize,
                                  ) -> Result<SeqHelper<'a, Self>, Self::Error>
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
        self.serialize_tuple_elt(value)
    }

    /// Finishes serialization of a tuple variant.
    ///
    /// By default, tuple variants are serialized as tuples.
    #[inline]
    fn serialize_tuple_variant_end(&mut self) -> Result<(), Self::Error> {
        self.serialize_tuple_end()
    }

    /// Serialize a map.
    fn serialize_map<'a>(&'a mut self, len: Option<usize>) -> Result<MapHelper<'a, Self>, Self::Error>;

    /// Serialize a map element
    fn serialize_map_elt<'a, K, V>(&'a mut self, key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize;

    /// Finishes serializing a map
    fn serialize_map_end<'a>(&'a mut self) -> Result<(), Self::Error>;

    /// Serializes a struct.
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    #[inline]
    fn serialize_struct<'a>(&'a mut self,
                           _name: &'static str,
                           len: usize,
                           ) -> Result<MapHelper<'a, Self>, Self::Error>
    {
        self.serialize_map(Some(len))
    }

    /// Serialize a struct field
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    fn serialize_struct_elt<'a, K, V>(&'a mut self, key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize {
        self.serialize_map_elt(key, value)
    }

    /// Finishes serializing a struct
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    fn serialize_struct_end<'a>(&'a mut self) -> Result<(), Self::Error> {
        self.serialize_map_end()
    }

    /// Serializes a struct variant.
    ///
    /// By default, struct variants are serialized as a struct.
    #[inline]
    fn serialize_struct_variant<'a>(&'a mut self,
                                   _name: &'static str,
                                   _variant_index: usize,
                                   variant: &'static str,
                                   len: usize,
                                   ) -> Result<MapHelper<'a, Self>, Self::Error>
    {
        self.serialize_struct(variant, len)
    }

    /// Serialize a struct variant element
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    fn serialize_struct_variant_elt<'a, K, V>(&'a mut self, key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize {
        self.serialize_struct_elt(key, value)
    }

    /// Finishes serializing a struct variant
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    fn serialize_struct_variant_end<'a>(&'a mut self) -> Result<(), Self::Error> {
        self.serialize_struct_end()
    }
}
