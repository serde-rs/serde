//! Data structures for buffering self-describing formats.
//!
//! ```
//! # use serde::de::{Deserialize, value, IntoDeserializer, content::Content};
//! let content = Content::from(32);
//! let deserializer = IntoDeserializer::<value::Error>::into_deserializer(content);
//! assert_eq!(u32::deserialize(deserializer).unwrap(), 32);
//! ```

use std::fmt;

use crate::{de, private, Deserialize};

use super::Visitor;

/// An efficient buffer for arbitrary self-describing *content*.
#[derive(Clone)]
#[repr(transparent)]
pub struct Content<'de>(private::de::Content<'de>);

impl<'de> Deserialize<'de> for Content<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: crate::Deserializer<'de>,
    {
        Ok(Self(private::de::Content::deserialize(deserializer)?))
    }
}

impl<'de, E: de::Error> de::IntoDeserializer<'de, E> for Content<'de> {
    type Deserializer = ContentDeserializer<'de, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        ContentDeserializer(private::de::ContentDeserializer::new(self.0))
    }
}

macro_rules! impl_from_for_content {
    ($($type:ty => $constructor:ident ,)*) => {
        $(
            impl<'de> From<$type> for Content<'de> {
                fn from(value: $type) -> Self {
                    Self(private::de::Content::$constructor(value))
                }
            }
        )*
    };
}

impl_from_for_content! {
    bool => Bool,

    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,

    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,

    f32 => F32,
    f64 => F64,

    char => Char,

    String => String,
    Vec<u8> => ByteBuf,
}

impl<'de> From<&'de str> for Content<'de> {
    fn from(value: &'de str) -> Self {
        Self(private::de::Content::Str(value))
    }
}

impl<'de> From<&'de [u8]> for Content<'de> {
    fn from(value: &'de [u8]) -> Self {
        Self(private::de::Content::Bytes(value))
    }
}

/// A [`Visitor`] for constructing [`Content`].
pub struct ContentVisitor<'de>(private::de::ContentVisitor<'de>);

impl<'de> ContentVisitor<'de> {
    /// Constructs a new [`ContentVisitor`].
    pub fn new() -> Self {
        Self(private::de::ContentVisitor::new())
    }
}

macro_rules! impl_fn_delegate_visit {
    ($($func:ident($type:ty),)*) => {
        $(
            #[inline]
            fn $func<E>(self, value: $type) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                Ok(Content(self.0.$func(value)?))
            }
        )*
    };
}

impl<'de> Visitor<'de> for ContentVisitor<'de> {
    type Value = Content<'de>;

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.expecting(formatter)
    }

    impl_fn_delegate_visit! {
        visit_bool(bool),

        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),

        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),

        visit_f32(f32),
        visit_f64(f64),

        visit_char(char),

        visit_str(&str),
        visit_borrowed_str(&'de str),
        visit_string(String),

        visit_bytes(&[u8]),
        visit_borrowed_bytes(&'de [u8]),
        visit_byte_buf(Vec<u8>),
    }

    serde_if_integer128! {
        impl_fn_delegate_visit! {
            visit_u128(u128),
            visit_i128(i128),
        }
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content(self.0.visit_none()?))
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: crate::Deserializer<'de>,
    {
        Ok(Content(self.0.visit_some(deserializer)?))
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content(self.0.visit_unit()?))
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: crate::Deserializer<'de>,
    {
        Ok(Content(self.0.visit_newtype_struct(deserializer)?))
    }

    #[inline]
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        Ok(Content(self.0.visit_seq(seq)?))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        Ok(Content(self.0.visit_map(map)?))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        Ok(Content(self.0.visit_enum(data)?))
    }

    fn __private_visit_untagged_option<D>(self, deserializer: D) -> Result<Self::Value, ()>
    where
        D: crate::Deserializer<'de>,
    {
        Ok(Content(
            self.0.__private_visit_untagged_option(deserializer)?,
        ))
    }
}

/// A deserializer for buffered [`Content`].
pub struct ContentDeserializer<'de, E: de::Error>(private::de::ContentDeserializer<'de, E>);

macro_rules! impl_fn_delegate_deserialize {
    ($($func:ident,)*) => {
        $(
            #[inline]
            fn $func<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                self.0.$func(visitor)
            }
        )*
    };
}

impl<'de, E: de::Error> de::Deserializer<'de> for ContentDeserializer<'de, E> {
    type Error = E;

    impl_fn_delegate_deserialize!(
        deserialize_any,
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any,
    );

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_unit_struct(name, visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_newtype_struct(name, visitor)
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_tuple_struct(name, len, visitor)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_struct(name, fields, visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_enum(name, variants, visitor)
    }

    #[inline]
    fn __deserialize_content<V>(
        self,
        actually_private: crate::actually_private::T,
        visitor: V,
    ) -> Result<private::de::Content<'de>, Self::Error>
    where
        V: de::Visitor<'de, Value = private::de::Content<'de>>,
    {
        self.0.__deserialize_content(actually_private, visitor)
    }
}
