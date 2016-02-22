//! Generic serialization framework.

use std::error;

pub mod impls;

///////////////////////////////////////////////////////////////////////////////

/// `Error` is a trait that allows a `Serialize` to generically create a
/// `Serializer` error.
pub trait Error: Sized + error::Error {
    /// Raised when there is general error when deserializing a type.
    fn syntax(msg: &str) -> Self;

    /// Raised when a `Serialize` was passed an incorrect value.
    fn invalid_value(msg: &str) -> Self {
        Error::syntax(msg)
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
        // FIXME: this allocation is required in order to be compatible with stable rust, which
        // doesn't support encoding a `char` into a stack buffer.
        self.serialize_str(&v.to_string())
    }

    /// Serializes a `&str`.
    fn serialize_str(&mut self, value: &str) -> Result<(), Self::Error>;

    /// Enables those serialization formats that support serializing
    /// byte slices separately from generic arrays. By default it serializes as a regular array.
    #[inline]
    fn serialize_bytes(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        self.serialize_seq(impls::SeqIteratorVisitor::new(value.iter(), Some(value.len())))
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
        self.serialize_tuple_struct(name, Some(value))
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
        self.serialize_tuple_variant(
            name,
            variant_index,
            variant,
            Some(value))
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
    fn serialize_seq<V>(&mut self, visitor: V) -> Result<(), Self::Error>
        where V: SeqVisitor;

    /// Serializes a sequence element.
    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize;

    /// Serializes a tuple.
    ///
    /// By default this serializes a tuple as a sequence.
    #[inline]
    fn serialize_tuple<V>(&mut self, visitor: V) -> Result<(), Self::Error>
        where V: SeqVisitor,
    {
        self.serialize_seq(visitor)
    }

    /// Serializes a tuple element.
    ///
    /// By default, tuples are serialized as a sequence.
    #[inline]
    fn serialize_tuple_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize
    {
        self.serialize_seq_elt(value)
    }

    /// Serializes a tuple struct.
    ///
    /// By default, tuple structs are serialized as a tuple.
    #[inline]
    fn serialize_tuple_struct<V>(&mut self,
                                 _name: &'static str,
                                 visitor: V) -> Result<(), Self::Error>
        where V: SeqVisitor,
    {
        self.serialize_tuple(visitor)
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

    /// Serializes a tuple variant.
    ///
    /// By default, tuple variants are serialized as a tuple struct.
    #[inline]
    fn serialize_tuple_variant<V>(&mut self,
                                  _name: &'static str,
                                  _variant_index: usize,
                                  variant: &'static str,
                                  visitor: V) -> Result<(), Self::Error>
        where V: SeqVisitor,
    {
        self.serialize_tuple_struct(variant, visitor)
    }

    /// Serializes a tuple element.
    ///
    /// By default, tuples are serialized as a sequence.
    #[inline]
    fn serialize_tuple_variant_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize
    {
        self.serialize_tuple_struct_elt(value)
    }

    /// Serializes a map.
    ///
    /// Callees of this method need to construct a `MapVisitor`, which iterates through each item
    /// in the map.
    fn serialize_map<V>(&mut self, visitor: V) -> Result<(), Self::Error>
        where V: MapVisitor;

    /// Serializes a map element (key-value pair).
    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error>
        where K: Serialize,
              V: Serialize;

    /// Serializes a struct.
    ///
    /// By default, structs are serialized as a map with the field name as the key.
    #[inline]
    fn serialize_struct<V>(&mut self,
                           _name: &'static str,
                           visitor: V) -> Result<(), Self::Error>
        where V: MapVisitor,
    {
        self.serialize_map(visitor)
    }

    /// Serializes an element of a struct.
    ///
    /// By default, struct elements are serialized as a map element with the field name as the key.
    #[inline]
    fn serialize_struct_elt<V>(&mut self,
                               key: &'static str,
                               value: V) -> Result<(), Self::Error>
        where V: Serialize,
    {
        self.serialize_map_elt(key, value)
    }

    /// Serializes a struct variant.
    ///
    /// By default, struct variants are serialized as a struct.
    #[inline]
    fn serialize_struct_variant<V>(&mut self,
                                   _name: &'static str,
                                   _variant_index: usize,
                                   variant: &'static str,
                                   visitor: V) -> Result<(), Self::Error>
        where V: MapVisitor,
    {
        self.serialize_struct(variant, visitor)
    }

    /// Serializes an element of a struct variant.
    ///
    /// By default, struct variant elements are serialized as a struct element.
    #[inline]
    fn serialize_struct_variant_elt<V>(&mut self,
                                       key: &'static str,
                                       value: V) -> Result<(), Self::Error>
        where V: Serialize,
    {
        self.serialize_struct_elt(key, value)
    }
}

/// A trait that is used by a `Serialize` to iterate through a sequence.
#[cfg_attr(feature = "nightly-testing", allow(len_without_is_empty))]
pub trait SeqVisitor {
    /// Serializes a sequence item in the serializer.
    ///
    /// This returns `Ok(Some(()))` when there are more items to serialize, or `Ok(None)` when
    /// complete.
    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
        where S: Serializer;

    /// Return the length of the sequence if known.
    #[inline]
    fn len(&self) -> Option<usize> {
        None
    }
}

/// A trait that is used by a `Serialize` to iterate through a map.
#[cfg_attr(feature = "nightly-testing", allow(len_without_is_empty))]
pub trait MapVisitor {
    /// Serializes a map item in the serializer.
    ///
    /// This returns `Ok(Some(()))` when there are more items to serialize, or `Ok(None)` when
    /// complete.
    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
        where S: Serializer;

    /// Return the length of the map if known.
    #[inline]
    fn len(&self) -> Option<usize> {
        None
    }
}
