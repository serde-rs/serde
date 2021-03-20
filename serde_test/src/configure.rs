use std::any::Any;
use std::fmt;
use std::marker::PhantomData;

use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Readable<T: ?Sized>(T);
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Compact<T: ?Sized>(T);
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WithContext<C, T: ?Sized>(PhantomData<C>, T);

trait Wrapper {
    type Wrapped;
    fn unwrap(self) -> Self::Wrapped;
}
impl<T> From<T> for Readable<T> {
    fn from(t: T) -> Self { Readable(t) }
}
impl<T> Wrapper for Readable<T> {
    type Wrapped = T;
    fn unwrap(self) -> T { self.0 }
}
impl<T: ?Sized> AsRef<T> for Readable<T> {
    fn as_ref(&self) -> &T { &self.0 }
}
impl<T: ?Sized> AsMut<T> for Readable<T> {
    fn as_mut(&mut self) -> &mut T { &mut self.0 }
}
impl<T> From<T> for Compact<T> {
    fn from(t: T) -> Self { Compact(t) }
}
impl<T> Wrapper for Compact<T> {
    type Wrapped = T;
    fn unwrap(self) -> T { self.0 }
}
impl<T: ?Sized> AsRef<T> for Compact<T> {
    fn as_ref(&self) -> &T { &self.0 }
}
impl<T: ?Sized> AsMut<T> for Compact<T> {
    fn as_mut(&mut self) -> &mut T { &mut self.0 }
}
impl<C, T> From<T> for WithContext<C, T> {
    fn from(t: T) -> Self { WithContext(PhantomData, t) }
}
impl<C, T> Wrapper for WithContext<C, T> {
    type Wrapped = T;
    fn unwrap(self) -> T { self.1 }
}
impl<C, T: ?Sized> AsRef<T> for WithContext<C, T> {
    fn as_ref(&self) -> &T { &self.1 }
}
impl<C, T: ?Sized> AsMut<T> for WithContext<C, T> {
    fn as_mut(&mut self) -> &mut T { &mut self.1 }
}

pub trait Context {
    type C: 'static;
    fn context() -> &'static Self::C;
}

/// Trait to determine whether a value is represented in human-readable or
/// compact form, or has other context.
///
/// ```edition2018
/// use std::any::Any;
/// use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeMap};
/// use serde_test::{assert_tokens, Configure, Context, Token};
///
/// enum Naming {
///     Short,
///     Unnamed,
/// }
///
/// #[derive(Debug, PartialEq)]
/// struct Example(u8, u8);
///
/// impl Serialize for Example {
///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
///     where
///         S: Serializer,
///     {
///         if let Some(Naming::Short) = serializer
///             .get_context::<Naming>()
///             .downcast_ref::<Naming>()
///         {
///             let mut map_serializer = serializer.serialize_map(Some(2))?;
///             map_serializer.serialize_key(&'a')?;
///             map_serializer.serialize_value(&self.0)?;
///             map_serializer.serialize_key(&'b')?;
///             map_serializer.serialize_value(&self.1)?;
///             map_serializer.end()
///         } else if serializer.is_human_readable() {
///             format!("{}.{}", self.0, self.1).serialize(serializer)
///         } else {
///             (self.0, self.1).serialize(serializer)
///         }
///     }
/// }
///
/// impl<'de> Deserialize<'de> for Example {
///     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
///     where
///         D: Deserializer<'de>,
///     {
///         use serde::de::Error;
///         if let Some(Naming::Short) = deserializer
///             .get_context::<Naming>()
///             .downcast_ref::<Naming>()
///         {
///             let map = std::collections::HashMap::<char, u8>::deserialize(deserializer)?;
///             Ok(Example(
///                 *map.get(&'a').ok_or(D::Error::custom("missing 'a' key"))?,
///                 *map.get(&'b').ok_or(D::Error::custom("missing 'b' key"))?,
///             ))
///         } else if deserializer.is_human_readable() {
///             let s = String::deserialize(deserializer)?;
///             let parts: Vec<_> = s.split('.').collect();
///             Ok(Example(
///                 parts[0].parse().map_err(D::Error::custom)?,
///                 parts[1].parse().map_err(D::Error::custom)?,
///             ))
///         } else {
///             let (x, y) = Deserialize::deserialize(deserializer)?;
///             Ok(Example(x, y))
///         }
///     }
/// }
///
/// fn main() {
///     #[derive(Debug, PartialEq)]
///     struct ShortNamingContext;
///     impl Context for ShortNamingContext {
///         type C = Naming;
///         fn context() -> &'static Naming { &Naming::Short }
///     }
///     assert_tokens(
///         &Example(1, 0).with_context::<ShortNamingContext>(),
///         &[
///             Token::Map { len: Some(2) },
///             Token::Char('a'),
///             Token::U8(1),
///             Token::Char('b'),
///             Token::U8(0),
///             Token::MapEnd,
///         ]
///     );
///
///     #[derive(Debug, PartialEq)]
///     struct NoNamingContext;
///     impl Context for NoNamingContext {
///         type C = Naming;
///         fn context() -> &'static Naming { &Naming::Unnamed }
///     }
///     assert_tokens(
///         &Example(1, 0).with_context::<NoNamingContext>().compact(),
///         &[
///             Token::Tuple { len: 2 },
///             Token::U8(1),
///             Token::U8(0),
///             Token::TupleEnd,
///         ],
///     );
///     assert_tokens(
///         &Example(1, 0).with_context::<NoNamingContext>().readable(),
///         &[Token::Str("1.0"),
///     ]);
/// }
/// ```
pub trait Configure {
    /// Marks `self` as using `is_human_readable == true`
    fn readable(self) -> Readable<Self>
    where
        Self: Sized,
    {
        self.into()
    }
    /// Marks `self` as using `is_human_readable == false`
    fn compact(self) -> Compact<Self>
    where
        Self: Sized,
    {
        self.into()
    }
    /// Marks `self` as using given context
    fn with_context<C>(self) -> WithContext<C, Self>
    where
        Self: Sized,
    {
        self.into()
}
}

impl<T: ?Sized> Configure for T {}

impl<T: ?Sized> Serialize for Readable<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_ref().serialize(Readable::from(serializer))
    }
}
impl<T: ?Sized> Serialize for Compact<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_ref().serialize(Compact::from(serializer))
    }
}
impl<C: Context, T: ?Sized> Serialize for WithContext<C, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_ref().serialize(WithContext::<C, _>::from(serializer))
    }
}
impl<'de, T> Deserialize<'de> for Readable<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(Readable::from(deserializer)).map(Into::into)
    }
}
impl<'de, T> Deserialize<'de> for Compact<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(Compact::from(deserializer)).map(Into::into)
    }
}
impl<'de, C: Context, T> Deserialize<'de> for WithContext<C, T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(WithContext::<C, _>::from(deserializer)).map(Into::into)
    }
}
impl<'de, T> DeserializeSeed<'de> for Readable<T>
where
    T: DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.unwrap().deserialize(Readable::from(deserializer))
    }
}
impl<'de, T> DeserializeSeed<'de> for Compact<T>
where
    T: DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.unwrap().deserialize(Compact::from(deserializer))
    }
}
impl<'de, C: Context, T> DeserializeSeed<'de> for WithContext<C, T>
where
    T: DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.unwrap().deserialize(WithContext::<C, _>::from(deserializer))
    }
}

macro_rules! forward_method {
    ($name: ident (self $(, $arg: ident : $arg_type: ty)* ) -> $return_type: ty) => {
        fn $name (self $(, $arg : $arg_type)* ) -> $return_type {
            self.unwrap().$name( $($arg),* )
        }
    };
}

macro_rules! forward_serialize_methods {
    ( $( $name: ident $arg_type: ty ),* ) => {
        $(
            forward_method!($name(self, v : $arg_type) -> Result<Self::Ok, Self::Error>);
        )*
    };
}

macro_rules! impl_serializer {
    ($wrapper:ident $(<$context:ident>)*, $is_human_readable:expr) => {
        impl<$($context: Context,)* S> Serializer for $wrapper<$($context,)* S>
        where
            S: Serializer,
        {
            type Ok = S::Ok;
            type Error = S::Error;

            type SerializeSeq = $wrapper<$($context,)* S::SerializeSeq>;
            type SerializeTuple = $wrapper<$($context,)* S::SerializeTuple>;
            type SerializeTupleStruct = $wrapper<$($context,)* S::SerializeTupleStruct>;
            type SerializeTupleVariant = $wrapper<$($context,)* S::SerializeTupleVariant>;
            type SerializeMap = $wrapper<$($context,)* S::SerializeMap>;
            type SerializeStruct = $wrapper<$($context,)* S::SerializeStruct>;
            type SerializeStructVariant = $wrapper<$($context,)* S::SerializeStructVariant>;

            fn is_human_readable(&self) -> bool {
                $(
                    let _ = stringify!($context);
                    return self.as_ref().is_human_readable();
                )*

                #[allow(unreachable_code)]
                $is_human_readable
            }

            #[allow(bare_trait_objects)] // to support rustc < 1.27
            fn get_context<T: ?Sized + Any>(&self) -> &Any {
                $(return $context::context();)*

                #[allow(unreachable_code)]
                self.as_ref().get_context::<T>()
            }

            forward_serialize_methods! {
                serialize_bool bool,
                serialize_i8 i8,
                serialize_i16 i16,
                serialize_i32 i32,
                serialize_i64 i64,
                serialize_u8 u8,
                serialize_u16 u16,
                serialize_u32 u32,
                serialize_u64 u64,
                serialize_f32 f32,
                serialize_f64 f64,
                serialize_char char,
                serialize_str &str,
                serialize_bytes &[u8],
                serialize_unit_struct &'static str

            }

            fn serialize_unit(self) -> Result<S::Ok, S::Error> {
                self.unwrap().serialize_unit()
            }

            fn serialize_unit_variant(
                self,
                name: &'static str,
                variant_index: u32,
                variant: &'static str,
            ) -> Result<S::Ok, S::Error> {
                self.unwrap().serialize_unit_variant(name, variant_index, variant)
            }

            fn serialize_newtype_struct<T: ?Sized>(
                self,
                name: &'static str,
                value: &T,
            ) -> Result<S::Ok, S::Error>
            where
                T: Serialize,
            {
                self.unwrap().serialize_newtype_struct(name, &$wrapper::$(<$context, _>::)*from(value))
            }

            fn serialize_newtype_variant<T: ?Sized>(
                self,
                name: &'static str,
                variant_index: u32,
                variant: &'static str,
                value: &T,
            ) -> Result<S::Ok, S::Error>
            where
                T: Serialize,
            {
                self.unwrap()
                    .serialize_newtype_variant(name, variant_index, variant, &$wrapper::$(<$context, _>::)*from(value))
            }

            fn serialize_none(self) -> Result<S::Ok, Self::Error> {
                self.unwrap().serialize_none()
            }

            fn serialize_some<T: ?Sized>(self, value: &T) -> Result<S::Ok, Self::Error>
            where
                T: Serialize,
            {
                self.unwrap().serialize_some(&$wrapper::$(<$context, _>::)*from(value))
            }

            fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
                self.unwrap().serialize_seq(len).map(From::from)
            }

            fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
                self.unwrap().serialize_tuple(len).map(From::from)
            }

            fn serialize_tuple_struct(
                self,
                name: &'static str,
                len: usize,
            ) -> Result<Self::SerializeTupleStruct, Self::Error> {
                self.unwrap().serialize_tuple_struct(name, len).map(From::from)
            }

            fn serialize_tuple_variant(
                self,
                name: &'static str,
                variant_index: u32,
                variant: &'static str,
                len: usize,
            ) -> Result<Self::SerializeTupleVariant, Self::Error> {
                self.unwrap()
                    .serialize_tuple_variant(name, variant_index, variant, len)
                    .map(From::from)
            }

            fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
                self.unwrap().serialize_map(len).map(From::from)
            }

            fn serialize_struct(
                self,
                name: &'static str,
                len: usize,
            ) -> Result<Self::SerializeStruct, Self::Error> {
                self.unwrap().serialize_struct(name, len).map(From::from)
            }

            fn serialize_struct_variant(
                self,
                name: &'static str,
                variant_index: u32,
                variant: &'static str,
                len: usize,
            ) -> Result<Self::SerializeStructVariant, Self::Error> {
                self.unwrap()
                    .serialize_struct_variant(name, variant_index, variant, len)
                    .map(From::from)
            }
        }

        impl<$($context: Context,)* S> SerializeSeq for $wrapper<$($context,)* S>
        where
            S: SerializeSeq,
        {
            type Ok = S::Ok;
            type Error = S::Error;
            fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
            where
                T: Serialize,
            {
                self.as_mut().serialize_element(&$wrapper::$(<$context, _>::)*from(value))
            }
            fn end(self) -> Result<S::Ok, S::Error> {
                self.unwrap().end()
            }
        }

        impl<$($context: Context,)* S> SerializeTuple for $wrapper<$($context,)* S>
        where
            S: SerializeTuple,
        {
            type Ok = S::Ok;
            type Error = S::Error;
            fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
            where
                T: Serialize,
            {
                self.as_mut().serialize_element(&$wrapper::$(<$context, _>::)*from(value))
            }
            fn end(self) -> Result<S::Ok, S::Error> {
                self.unwrap().end()
            }
        }

        impl<$($context: Context,)* S> SerializeTupleStruct for $wrapper<$($context,)* S>
        where
            S: SerializeTupleStruct,
        {
            type Ok = S::Ok;
            type Error = S::Error;
            fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
            where
                T: Serialize,
            {
                self.as_mut().serialize_field(&$wrapper::$(<$context, _>::)*from(value))
            }
            fn end(self) -> Result<S::Ok, S::Error> {
                self.unwrap().end()
            }
        }

        impl<$($context: Context,)* S> SerializeTupleVariant for $wrapper<$($context,)* S>
        where
            S: SerializeTupleVariant,
        {
            type Ok = S::Ok;
            type Error = S::Error;
            fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
            where
                T: Serialize,
            {
                self.as_mut().serialize_field(&$wrapper::$(<$context, _>::)*from(value))
            }
            fn end(self) -> Result<S::Ok, S::Error> {
                self.unwrap().end()
            }
        }

        impl<$($context: Context,)* S> SerializeMap for $wrapper<$($context,)* S>
        where
            S: SerializeMap,
        {
            type Ok = S::Ok;
            type Error = S::Error;
            fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), S::Error>
            where
                T: Serialize,
            {
                self.as_mut().serialize_key(&$wrapper::$(<$context, _>::)*from(key))
            }
            fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
            where
                T: Serialize,
            {
                self.as_mut().serialize_value(&$wrapper::$(<$context, _>::)*from(value))
            }
            fn serialize_entry<K: ?Sized, V: ?Sized>(
                &mut self,
                key: &K,
                value: &V,
            ) -> Result<(), S::Error>
            where
                K: Serialize,
                V: Serialize,
            {
                self.as_mut().serialize_entry(key, &$wrapper::$(<$context, _>::)*from(value))
            }
            fn end(self) -> Result<S::Ok, S::Error> {
                self.unwrap().end()
            }
        }

        impl<$($context: Context,)* S> SerializeStruct for $wrapper<$($context,)* S>
        where
            S: SerializeStruct,
        {
            type Ok = S::Ok;
            type Error = S::Error;
            fn serialize_field<T: ?Sized>(
                &mut self,
                name: &'static str,
                field: &T,
            ) -> Result<(), S::Error>
            where
                T: Serialize,
            {
                self.as_mut().serialize_field(name, &$wrapper::$(<$context, _>::)*from(field))
            }
            fn end(self) -> Result<S::Ok, S::Error> {
                self.unwrap().end()
            }
        }

        impl<$($context: Context,)* S> SerializeStructVariant for $wrapper<$($context,)* S>
        where
            S: SerializeStructVariant,
        {
            type Ok = S::Ok;
            type Error = S::Error;
            fn serialize_field<T: ?Sized>(
                &mut self,
                name: &'static str,
                field: &T,
            ) -> Result<(), S::Error>
            where
                T: Serialize,
            {
                self.as_mut().serialize_field(name, &$wrapper::$(<$context, _>::)*from(field))
            }
            fn end(self) -> Result<S::Ok, S::Error> {
                self.unwrap().end()
            }
        }
    };
}

impl_serializer!(Readable, true);
impl_serializer!(Compact, false);
impl_serializer!(WithContext<C>, true);

use serde::de::{DeserializeSeed, EnumAccess, Error, MapAccess, SeqAccess, VariantAccess, Visitor};

macro_rules! forward_deserialize_methods {
    ( $wrapper : ty { $( $name: ident ),* } ) => {
        $(
            fn $name<V>(self, visitor: V) -> Result<V::Value, D::Error>
            where
                V: Visitor<'de>,
            {
                self.unwrap().$name(<$wrapper>::from(visitor))
            }
        )*
    };
}

macro_rules! impl_deserializer {
    ($wrapper:ident $(<$context:ident>)*, $is_human_readable:expr) => {
        impl<'de, $($context: Context,)* D> Deserializer<'de> for $wrapper<$($context,)* D>
        where
            D: Deserializer<'de>,
        {
            type Error = D::Error;

            forward_deserialize_methods! {
                $wrapper<$($context,)* _> {
                    deserialize_any,
                    deserialize_bool,
                    deserialize_u8,
                    deserialize_u16,
                    deserialize_u32,
                    deserialize_u64,
                    deserialize_i8,
                    deserialize_i16,
                    deserialize_i32,
                    deserialize_i64,
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
                    deserialize_ignored_any
                }
            }

            fn deserialize_unit_struct<V>(
                self,
                name: &'static str,
                visitor: V,
            ) -> Result<V::Value, D::Error>
            where
                V: Visitor<'de>,
            {
                self.unwrap().deserialize_unit_struct(name, $wrapper::$(<$context, _>::)*from(visitor))
            }
            fn deserialize_newtype_struct<V>(
                self,
                name: &'static str,
                visitor: V,
            ) -> Result<V::Value, D::Error>
            where
                V: Visitor<'de>,
            {
                self.unwrap().deserialize_newtype_struct(name, $wrapper::$(<$context, _>::)*from(visitor))
            }
            fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, D::Error>
            where
                V: Visitor<'de>,
            {
                self.unwrap().deserialize_tuple(len, $wrapper::$(<$context, _>::)*from(visitor))
            }
            fn deserialize_tuple_struct<V>(
                self,
                name: &'static str,
                len: usize,
                visitor: V,
            ) -> Result<V::Value, D::Error>
            where
                V: Visitor<'de>,
            {
                self.unwrap()
                    .deserialize_tuple_struct(name, len, $wrapper::$(<$context, _>::)*from(visitor))
            }
            fn deserialize_struct<V>(
                self,
                name: &'static str,
                fields: &'static [&'static str],
                visitor: V,
            ) -> Result<V::Value, D::Error>
            where
                V: Visitor<'de>,
            {
                self.unwrap().deserialize_struct(name, fields, $wrapper::$(<$context, _>::)*from(visitor))
            }
            fn deserialize_enum<V>(
                self,
                name: &'static str,
                variants: &'static [&'static str],
                visitor: V,
            ) -> Result<V::Value, D::Error>
            where
                V: Visitor<'de>,
            {
                self.unwrap().deserialize_enum(name, variants, $wrapper::$(<$context, _>::)*from(visitor))
            }

            fn is_human_readable(&self) -> bool {
                $(
                    let _ = stringify!($context);
                    return self.as_ref().is_human_readable();
                )*

                #[allow(unreachable_code)]
                $is_human_readable
            }

            #[allow(bare_trait_objects)] // to support rustc < 1.27
            fn get_context<T: ?Sized + Any>(&self) -> &Any {
                $(return $context::context();)*

                #[allow(unreachable_code)]
                self.as_ref().get_context::<T>()
            }
        }

        impl<'de, $($context: Context,)* D> Visitor<'de> for $wrapper<$($context,)* D>
        where
            D: Visitor<'de>,
        {
            type Value = D::Value;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                self.as_ref().expecting(formatter)
            }
            fn visit_bool<E>(self, v: bool) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_bool(v)
            }
            fn visit_i8<E>(self, v: i8) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_i8(v)
            }
            fn visit_i16<E>(self, v: i16) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_i16(v)
            }
            fn visit_i32<E>(self, v: i32) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_i32(v)
            }
            fn visit_i64<E>(self, v: i64) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_i64(v)
            }
            fn visit_u8<E>(self, v: u8) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_u8(v)
            }
            fn visit_u16<E>(self, v: u16) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_u16(v)
            }
            fn visit_u32<E>(self, v: u32) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_u32(v)
            }
            fn visit_u64<E>(self, v: u64) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_u64(v)
            }
            fn visit_f32<E>(self, v: f32) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_f32(v)
            }
            fn visit_f64<E>(self, v: f64) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_f64(v)
            }
            fn visit_char<E>(self, v: char) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_char(v)
            }
            fn visit_str<E>(self, v: &str) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_str(v)
            }
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_borrowed_str(v)
            }
            fn visit_string<E>(self, v: String) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_string(v)
            }
            fn visit_bytes<E>(self, v: &[u8]) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_bytes(v)
            }
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_borrowed_bytes(v)
            }
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_byte_buf(v)
            }
            fn visit_none<E>(self) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_none()
            }
            fn visit_some<D2>(self, deserializer: D2) -> Result<Self::Value, D2::Error>
            where
                D2: Deserializer<'de>,
            {
                self.unwrap().visit_some($wrapper::$(<$context, _>::)*from(deserializer))
            }
            fn visit_unit<E>(self) -> Result<D::Value, E>
            where
                E: Error,
            {
                self.unwrap().visit_unit()
            }
            fn visit_newtype_struct<D2>(self, deserializer: D2) -> Result<Self::Value, D2::Error>
            where
                D2: Deserializer<'de>,
            {
                self.unwrap().visit_newtype_struct($wrapper::$(<$context, _>::)*from(deserializer))
            }
            fn visit_seq<V>(self, seq: V) -> Result<D::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                self.unwrap().visit_seq($wrapper::$(<$context, _>::)*from(seq))
            }
            fn visit_map<V>(self, map: V) -> Result<D::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                self.unwrap().visit_map($wrapper::$(<$context, _>::)*from(map))
            }
            fn visit_enum<V>(self, data: V) -> Result<D::Value, V::Error>
            where
                V: EnumAccess<'de>,
            {
                self.unwrap().visit_enum($wrapper::$(<$context, _>::)*from(data))
            }
        }

        impl<'de, $($context: Context,)* D> SeqAccess<'de> for $wrapper<$($context,)* D>
        where
            D: SeqAccess<'de>,
        {
            type Error = D::Error;
            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, D::Error>
            where
                T: DeserializeSeed<'de>,
            {
                self.as_mut().next_element_seed($wrapper::$(<$context, _>::)*from(seed))
            }
            fn size_hint(&self) -> Option<usize> {
                self.as_ref().size_hint()
            }
        }

        impl<'de, $($context: Context,)* D> MapAccess<'de> for $wrapper<$($context,)* D>
        where
            D: MapAccess<'de>,
        {
            type Error = D::Error;
            fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, D::Error>
            where
                K: DeserializeSeed<'de>,
            {
                self.as_mut().next_key_seed($wrapper::$(<$context, _>::)*from(seed))
            }
            fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, D::Error>
            where
                V: DeserializeSeed<'de>,
            {
                self.as_mut().next_value_seed($wrapper::$(<$context, _>::)*from(seed))
            }
            fn next_entry_seed<K, V>(
                &mut self,
                kseed: K,
                vseed: V,
            ) -> Result<Option<(K::Value, V::Value)>, D::Error>
            where
                K: DeserializeSeed<'de>,
                V: DeserializeSeed<'de>,
            {
                self.as_mut().next_entry_seed($wrapper::$(<$context, _>::)*from(kseed), $wrapper::$(<$context, _>::)*from(vseed))
            }
            fn size_hint(&self) -> Option<usize> {
                self.as_ref().size_hint()
            }
        }

        impl<'de, $($context: Context,)* D> EnumAccess<'de> for $wrapper<$($context,)* D>
        where
            D: EnumAccess<'de>,
        {
            type Error = D::Error;
            type Variant = $wrapper<$($context,)* D::Variant>;
            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
            where
                V: DeserializeSeed<'de>,
            {
                self.unwrap()
                    .variant_seed($wrapper::$(<$context, _>::)*from(seed))
                    .map(|(value, variant)| (value, $wrapper::$(<$context, _>::)*from(variant)))
            }
        }

        impl<'de, $($context: Context,)* D> VariantAccess<'de> for $wrapper<$($context,)* D>
        where
            D: VariantAccess<'de>,
        {
            type Error = D::Error;
            fn unit_variant(self) -> Result<(), D::Error> {
                self.unwrap().unit_variant()
            }
            fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, D::Error>
            where
                T: DeserializeSeed<'de>,
            {
                self.unwrap().newtype_variant_seed($wrapper::$(<$context, _>::)*from(seed))
            }
            fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, D::Error>
            where
                V: Visitor<'de>,
            {
                self.unwrap().tuple_variant(len, $wrapper::$(<$context, _>::)*from(visitor))
            }
            fn struct_variant<V>(
                self,
                fields: &'static [&'static str],
                visitor: V,
            ) -> Result<V::Value, D::Error>
            where
                V: Visitor<'de>,
            {
                self.unwrap().struct_variant(fields, $wrapper::$(<$context, _>::)*from(visitor))
            }
        }
    };
}

impl_deserializer!(Readable, true);
impl_deserializer!(Compact, false);
impl_deserializer!(WithContext<C>, true);
