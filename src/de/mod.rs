//! Generic deserialization framework.

pub mod impls;
pub mod value;

///////////////////////////////////////////////////////////////////////////////

pub trait Error {
    fn syntax_error() -> Self;

    fn end_of_stream_error() -> Self;

    fn unknown_field_error(field: &str) -> Self;

    fn missing_field_error(field: &'static str) -> Self;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Deserialize {
    /// Deserialize this value given this `Deserializer`.
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer;
}

///////////////////////////////////////////////////////////////////////////////

/// `Deserializer` is an abstract trait that can deserialize values into a `Visitor`.
pub trait Deserializer {
    type Error: Error;

    /// The `visit` method walks a visitor through a value as it is being deserialized.
    fn visit<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// The `visit_option` method allows a `Deserialize` type to inform the `Deserializer` that
    /// it's expecting an optional value. This allows deserializers that encode an optional value
    /// as a nullable value to convert the null value into a `None`, and a regular value as
    /// `Some(value)`.
    #[inline]
    fn visit_option<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// The `visit_seq` method allows a `Deserialize` type to inform the `Deserializer` that it's
    /// expecting a sequence of values. This allows deserializers to parse sequences that aren't
    /// tagged as sequences.
    #[inline]
    fn visit_seq<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// The `visit_map` method allows a `Deserialize` type to inform the `Deserializer` that it's
    /// expecting a map of values. This allows deserializers to parse sequences that aren't tagged
    /// as maps.
    #[inline]
    fn visit_map<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// The `visit_named_unit` method allows a `Deserialize` type to inform the `Deserializer` that
    /// it's expecting a named unit. This allows deserializers to a named unit that aren't tagged
    /// as a named unit.
    #[inline]
    fn visit_named_unit<V>(&mut self, _name: &str, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// The `visit_named_seq` method allows a `Deserialize` type to inform the `Deserializer` that
    /// it's expecting a named sequence of values. This allows deserializers to parse sequences
    /// that aren't tagged as sequences.
    #[inline]
    fn visit_named_seq<V>(&mut self, _name: &str, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_seq(visitor)
    }

    /// The `visit_named_map` method allows a `Deserialize` type to inform the `Deserializer` that
    /// it's expecting a map of values. This allows deserializers to parse sequences that aren't
    /// tagged as maps.
    #[inline]
    fn visit_named_map<V>(&mut self, _name: &str, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_map(visitor)
    }

    /// The `visit_enum` method allows a `Deserialize` type to inform the `Deserializer` that it's
    /// expecting an enum value. This allows deserializers that provide a custom enumeration
    /// serialization to properly deserialize the type.
    #[inline]
    fn visit_enum<V>(&mut self, _enum: &str, _visitor: V) -> Result<V::Value, Self::Error>
        where V: EnumVisitor,
    {
        Err(Error::syntax_error())
    }

    /// The `visit_bytes` method allows a `Deserialize` type to inform the `Deserializer` that it's
    /// expecting a `Vec<u8>`. This allows deserializers that provide a custom byte vector
    /// serialization to properly deserialize the type.
    #[inline]
    fn visit_bytes<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
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
    type Value;

    fn visit_bool<E>(&mut self, _v: bool) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
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
        Err(Error::syntax_error())
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
        Err(Error::syntax_error())
    }

    fn visit_f32<E>(&mut self, v: f32) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_f64<E>(&mut self, _v: f64) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
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
        Err(Error::syntax_error())
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
        Err(Error::syntax_error())
    }

    #[inline]
    fn visit_named_unit<E>(&mut self, _name: &str) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_unit()
    }

    fn visit_none<E>(&mut self) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
    }

    fn visit_some<D>(&mut self, _deserializer: &mut D) -> Result<Self::Value, D::Error>
        where D: Deserializer,
    {
        Err(Error::syntax_error())
    }

    fn visit_seq<V>(&mut self, _visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqVisitor,
    {
        Err(Error::syntax_error())
    }

    fn visit_map<V>(&mut self, _visitor: V) -> Result<Self::Value, V::Error>
        where V: MapVisitor,
    {
        Err(Error::syntax_error())
    }

    fn visit_bytes<E>(&mut self, _v: &[u8]) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
    }

    fn visit_byte_buf<E>(&mut self, _v: Vec<u8>) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
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
        Err(Error::missing_field_error(field))
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
        Err(Error::syntax_error())
    }

    /// `visit_seq` is called when deserializing a tuple-like variant.
    fn visit_seq<V>(&mut self, _visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor
    {
        Err(Error::syntax_error())
    }

    /// `visit_map` is called when deserializing a struct-like variant.
    fn visit_map<V>(&mut self, _visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor
    {
        Err(Error::syntax_error())
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

    fn visit_seq<V>(&mut self, visitor: V) -> Result<V::Value, T::Error>
        where V: Visitor,
    {
        (**self).visit_seq(visitor)
    }

    fn visit_map<V>(&mut self, visitor: V) -> Result<V::Value, T::Error>
        where V: Visitor,
    {
        (**self).visit_map(visitor)
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
