//! Generic deserialization framework.

pub mod impls;
pub mod value;

///////////////////////////////////////////////////////////////////////////////

/// `Error` is a trait that allows a `Deserialize` to generically create a
/// `Deserializer` error.
pub trait Error: Sized {
    /// Raised when there is general error when deserializing a type.
    fn syntax(msg: &str) -> Self;

    /// Raised when a fixed sized sequence or map was passed in the wrong amount of arguments.
    fn length_mismatch(_len: usize) -> Self {
        Error::syntax("incorrect length")
    }

    /// Raised when a `Deserialize` was passed an incorrect type.
    fn type_mismatch(_type: Type) -> Self {
        Error::syntax("incorrect type")
    }

    /// Raised when a `Deserialize` type unexpectedly hit the end of the stream.
    fn end_of_stream() -> Self;

    /// Raised when a `Deserialize` struct type received an unexpected struct field.
    fn unknown_field(field: &str) -> Self;

    /// Raised when a `Deserialize` struct type did not receive a field.
    fn missing_field(field: &'static str) -> Self;
}

/// `Type` represents all the primitive types that can be deserialized. This is used by
/// `Error::kind_mismatch`.
pub enum Type {
    Bool,
    Usize,
    U8,
    U16,
    U32,
    U64,
    Isize,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Char,
    Str,
    String,
    Unit,
    Option,
    Seq,
    Map,
    UnitStruct,
    NewtypeStruct,
    TupleStruct,
    Struct,
    Tuple,
    Enum,
    StructVariant,
    TupleVariant,
    UnitVariant,
    Bytes,
}

///////////////////////////////////////////////////////////////////////////////

pub trait Deserialize: Sized {
    /// Deserialize this value given this `Deserializer`.
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer;
}

///////////////////////////////////////////////////////////////////////////////

/// `Deserializer` is a trait that can deserialize values by threading a `Visitor` trait through a
/// value. It supports two entry point styles which enables different kinds of deserialization.
///
/// 1) The `visit` method. File formats like JSON embed the type of it's construct in it's file
///    format. This allows the `Deserializer` to deserialize into a generic type like
///    `json::Value`, which can represent all JSON types.
///
/// 2) The `visit_*` methods. File formats like bincode do not embed in it's format how to decode
///    it's values. It relies instead on the `Deserialize` type to hint to the `Deserializer` with
///    the `visit_*` methods how it should parse the next value. One downside though to only
///    supporting the `visit_*` types is that it does not allow for deserializing into a generic
///    `json::Value`-esque type.
pub trait Deserializer {
    type Error: Error;

    /// This method walks a visitor through a value as it is being deserialized.
    fn visit<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a `bool` value.
    #[inline]
    fn visit_bool<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `usize` value.
    #[inline]
    fn visit_usize<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_u64(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `u8` value.
    #[inline]
    fn visit_u8<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_u64(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `u16` value.
    #[inline]
    fn visit_u16<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_u64(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `u32` value.
    #[inline]
    fn visit_u32<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_u64(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `u64` value.
    #[inline]
    fn visit_u64<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `isize` value.
    #[inline]
    fn visit_isize<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_i64(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `i8` value.
    #[inline]
    fn visit_i8<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_i64(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `i16` value.
    #[inline]
    fn visit_i16<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_i64(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `i32` value.
    #[inline]
    fn visit_i32<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_i64(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `i64` value.
    #[inline]
    fn visit_i64<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a `f32` value.
    #[inline]
    fn visit_f32<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_f64(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a `f64` value.
    #[inline]
    fn visit_f64<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a `char` value.
    #[inline]
    fn visit_char<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a `&str` value.
    #[inline]
    fn visit_str<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a `String` value.
    #[inline]
    fn visit_string<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_str(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `unit` value.
    #[inline]
    fn visit_unit<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an `Option` value. This allows
    /// deserializers that encode an optional value as a nullable value to convert the null value
    /// into a `None`, and a regular value as `Some(value)`.
    #[inline]
    fn visit_option<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a sequence value. This allows
    /// deserializers to parse sequences that aren't tagged as sequences.
    #[inline]
    fn visit_seq<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a map of values. This allows
    /// deserializers to parse sequences that aren't tagged as maps.
    #[inline]
    fn visit_map<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a unit struct. This allows
    /// deserializers to a unit struct that aren't tagged as a unit struct.
    #[inline]
    fn visit_unit_struct<V>(&mut self,
                            _name: &'static str,
                            visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_unit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a newtype struct. This allows
    /// deserializers to a newtype struct that aren't tagged as a newtype struct.
    #[inline]
    fn visit_newtype_struct<V>(&mut self,
                               name: &'static str,
                               visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_tuple_struct(name, 1, visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a tuple struct. This allows
    /// deserializers to parse sequences that aren't tagged as sequences.
    #[inline]
    fn visit_tuple_struct<V>(&mut self,
                             _name: &'static str,
                             len: usize,
                             visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_tuple(len, visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a struct. This allows
    /// deserializers to parse sequences that aren't tagged as maps.
    #[inline]
    fn visit_struct<V>(&mut self,
                       _name: &'static str,
                       _fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_map(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a tuple value. This allows
    /// deserializers that provide a custom tuple serialization to properly deserialize the type.
    #[inline]
    fn visit_tuple<V>(&mut self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_seq(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting an enum value. This allows
    /// deserializers that provide a custom enumeration serialization to properly deserialize the
    /// type.
    #[inline]
    fn visit_enum<V>(&mut self,
                     _enum: &'static str,
                     _variants: &'static [&'static str],
                     _visitor: V) -> Result<V::Value, Self::Error>
        where V: EnumVisitor,
    {
        Err(Error::syntax("expected an enum"))
    }

    /// This method hints that the `Deserialize` type is expecting a `Vec<u8>`. This allows
    /// deserializers that provide a custom byte vector serialization to properly deserialize the
    /// type.
    #[inline]
    fn visit_bytes<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_seq(visitor)
    }

    /// Specify a format string for the deserializer.
    ///
    /// The deserializer format is used to determine which format
    /// specific field attributes should be used with the
    /// deserializer.
    fn format() -> &'static str {
        ""
    }
}

///////////////////////////////////////////////////////////////////////////////

pub trait Visitor {
    type Value: Deserialize;

    fn visit_bool<E>(&mut self, _v: bool) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::type_mismatch(Type::Bool))
    }

    fn visit_isize<E>(&mut self, v: isize) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i8<E>(&mut self, v: i8) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i16<E>(&mut self, v: i16) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i32<E>(&mut self, v: i32) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i64<E>(&mut self, _v: i64) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::type_mismatch(Type::I64))
    }

    fn visit_usize<E>(&mut self, v: usize) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u8<E>(&mut self, v: u8) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u16<E>(&mut self, v: u16) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u32<E>(&mut self, v: u32) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u64<E>(&mut self, _v: u64) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::type_mismatch(Type::U64))
    }

    fn visit_f32<E>(&mut self, v: f32) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_f64<E>(&mut self, _v: f64) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::type_mismatch(Type::F64))
    }

    #[inline]
    fn visit_char<E>(&mut self, v: char) -> Result<Self::Value, E>
        where E: Error,
    {
        // FIXME: this allocation is required in order to be compatible with stable rust, which
        // doesn't support encoding a `char` into a stack buffer.
        self.visit_string(v.to_string())
    }

    fn visit_str<E>(&mut self, _v: &str) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::type_mismatch(Type::Str))
    }

    #[inline]
    fn visit_string<E>(&mut self, v: String) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_str(&v)
    }

    fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::type_mismatch(Type::Unit))
    }

    #[inline]
    fn visit_unit_struct<E>(&mut self, _name: &'static str) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_unit()
    }

    fn visit_none<E>(&mut self) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::type_mismatch(Type::Option))
    }

    fn visit_some<D>(&mut self, _deserializer: &mut D) -> Result<Self::Value, D::Error>
        where D: Deserializer,
    {
        Err(Error::type_mismatch(Type::Option))
    }

    fn visit_newtype_struct<D>(&mut self, _deserializer: &mut D) -> Result<Self::Value, D::Error>
        where D: Deserializer,
    {
        Err(Error::type_mismatch(Type::NewtypeStruct))
    }

    fn visit_seq<V>(&mut self, _visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqVisitor,
    {
        Err(Error::type_mismatch(Type::Seq))
    }

    fn visit_map<V>(&mut self, _visitor: V) -> Result<Self::Value, V::Error>
        where V: MapVisitor,
    {
        Err(Error::type_mismatch(Type::Map))
    }

    fn visit_bytes<E>(&mut self, _v: &[u8]) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::type_mismatch(Type::Bytes))
    }

    fn visit_byte_buf<E>(&mut self, v: Vec<u8>) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_bytes(&v)
    }
}

///////////////////////////////////////////////////////////////////////////////

pub trait SeqVisitor {
    type Error: Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Self::Error>
        where T: Deserialize;

    fn end(&mut self) -> Result<(), Self::Error>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<'a, V> SeqVisitor for &'a mut V where V: SeqVisitor {
    type Error = V::Error;

    #[inline]
    fn visit<T>(&mut self) -> Result<Option<T>, V::Error>
        where T: Deserialize
    {
        (**self).visit()
    }

    #[inline]
    fn end(&mut self) -> Result<(), V::Error> {
        (**self).end()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

pub trait MapVisitor {
    type Error: Error;

    #[inline]
    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
        where K: Deserialize,
              V: Deserialize,
    {
        match try!(self.visit_key()) {
            Some(key) => {
                let value = try!(self.visit_value());
                Ok(Some((key, value)))
            }
            None => Ok(None)
        }
    }

    fn visit_key<K>(&mut self) -> Result<Option<K>, Self::Error>
        where K: Deserialize;

    fn visit_value<V>(&mut self) -> Result<V, Self::Error>
        where V: Deserialize;

    fn end(&mut self) -> Result<(), Self::Error>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V, Self::Error>
        where V: Deserialize,
    {
        Err(Error::missing_field(field))
    }
}

impl<'a, V_> MapVisitor for &'a mut V_ where V_: MapVisitor {
    type Error = V_::Error;

    #[inline]
    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, V_::Error>
        where K: Deserialize,
              V: Deserialize,
    {
        (**self).visit()
    }

    #[inline]
    fn visit_key<K>(&mut self) -> Result<Option<K>, V_::Error>
        where K: Deserialize
    {
        (**self).visit_key()
    }

    #[inline]
    fn visit_value<V>(&mut self) -> Result<V, V_::Error>
        where V: Deserialize
    {
        (**self).visit_value()
    }

    #[inline]
    fn end(&mut self) -> Result<(), V_::Error> {
        (**self).end()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

/// `EnumVisitor` is a visitor that is created by the `Deserialize` and passed to the
/// `Deserializer` in order to deserialize enums.
pub trait EnumVisitor {
    type Value;

    fn visit<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
        where V: VariantVisitor;
}

///////////////////////////////////////////////////////////////////////////////

/// `VariantVisitor` is a visitor that is created by the `Deserializer` and passed to the
/// `Deserialize` in order to deserialize a specific enum variant.
pub trait VariantVisitor {
    type Error: Error;

    /// `visit_variant` is called to identify which variant to deserialize.
    fn visit_variant<V>(&mut self) -> Result<V, Self::Error>
        where V: Deserialize;

    /// `visit_unit` is called when deserializing a variant with no values.
    fn visit_unit(&mut self) -> Result<(), Self::Error> {
        Err(Error::type_mismatch(Type::UnitVariant))
    }

    /// `visit_newtype` is called when deserializing a variant with a single value. By default this
    /// uses the `visit_tuple` method to deserialize the value.
    #[inline]
    fn visit_newtype<T>(&mut self) -> Result<T, Self::Error>
        where T: Deserialize,
    {
        let (value,) = try!(self.visit_tuple(1, impls::TupleVisitor1::new()));
        Ok(value)
    }

    /// `visit_tuple` is called when deserializing a tuple-like variant.
    fn visit_tuple<V>(&mut self,
                      _len: usize,
                      _visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor
    {
        Err(Error::type_mismatch(Type::TupleVariant))
    }

    /// `visit_struct` is called when deserializing a struct-like variant.
    fn visit_struct<V>(&mut self,
                       _fields: &'static [&'static str],
                       _visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor
    {
        Err(Error::type_mismatch(Type::StructVariant))
    }
}

impl<'a, T> VariantVisitor for &'a mut T where T: VariantVisitor {
    type Error = T::Error;

    fn visit_variant<V>(&mut self) -> Result<V, T::Error>
        where V: Deserialize
    {
        (**self).visit_variant()
    }

    fn visit_unit(&mut self) -> Result<(), T::Error> {
        (**self).visit_unit()
    }

    fn visit_newtype<D>(&mut self) -> Result<D, T::Error>
        where D: Deserialize,
    {
        (**self).visit_newtype()
    }

    fn visit_tuple<V>(&mut self,
                      len: usize,
                      visitor: V) -> Result<V::Value, T::Error>
        where V: Visitor,
    {
        (**self).visit_tuple(len, visitor)
    }

    fn visit_struct<V>(&mut self,
                    fields: &'static [&'static str],
                    visitor: V) -> Result<V::Value, T::Error>
        where V: Visitor,
    {
        (**self).visit_struct(fields, visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

pub trait EnumSeqVisitor {
    type Value;

    fn visit<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqVisitor;
}

///////////////////////////////////////////////////////////////////////////////

pub trait EnumMapVisitor {
    type Value;

    fn visit<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
        where V: MapVisitor;
}
