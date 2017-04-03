#[cfg(any(feature = "std", feature = "collections"))]
use core::{fmt, str};

use core::marker::PhantomData;

#[cfg(feature = "collections")]
use collections::borrow::ToOwned;

#[cfg(feature = "std")]
use std::borrow::Cow;
#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::borrow::Cow;

#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::{String, Vec};

use de::{Deserialize, Deserializer, Error, Visitor};

#[cfg(any(feature = "std", feature = "collections"))]
use de::Unexpected;

#[cfg(any(feature = "std", feature = "collections"))]
pub use de::content::{Content, ContentRefDeserializer, ContentDeserializer, TaggedContentVisitor,
                      TagOrContentField, TagOrContentFieldVisitor, InternallyTaggedUnitVisitor,
                      UntaggedUnitVisitor};

/// If the missing field is of type `Option<T>` then treat is as `None`,
/// otherwise it is an error.
pub fn missing_field<'de, V, E>(field: &'static str) -> Result<V, E>
    where V: Deserialize<'de>,
          E: Error
{
    struct MissingFieldDeserializer<E>(&'static str, PhantomData<E>);

    impl<'de, E> Deserializer<'de> for MissingFieldDeserializer<E>
        where E: Error
    {
        type Error = E;

        fn deserialize<V>(self, _visitor: V) -> Result<V::Value, E>
            where V: Visitor<'de>
        {
            Err(Error::missing_field(self.0))
        }

        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, E>
            where V: Visitor<'de>
        {
            visitor.visit_none()
        }

        forward_to_deserialize! {
            bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
            seq_fixed_size bytes byte_buf map unit_struct newtype_struct
            tuple_struct struct struct_field tuple enum ignored_any
        }
    }

    let deserializer = MissingFieldDeserializer(field, PhantomData);
    Deserialize::deserialize(deserializer)
}

#[cfg(any(feature = "std", feature = "collections"))]
pub fn borrow_cow_str<'de, D>(deserializer: D) -> Result<Cow<'de, str>, D::Error>
    where D: Deserializer<'de>
{
    struct CowStrVisitor;

    impl<'a> Visitor<'a> for CowStrVisitor {
        type Value = Cow<'a, str>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(Cow::Owned(v.to_owned()))
        }

        fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(Cow::Borrowed(v))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(Cow::Owned(v))
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where E: Error
        {
            match str::from_utf8(v) {
                Ok(s) => Ok(Cow::Owned(s.to_owned())),
                Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
            }
        }

        fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
            where E: Error
        {
            match str::from_utf8(v) {
                Ok(s) => Ok(Cow::Borrowed(s)),
                Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
            }
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where E: Error
        {
            match String::from_utf8(v) {
                Ok(s) => Ok(Cow::Owned(s)),
                Err(e) => Err(Error::invalid_value(Unexpected::Bytes(&e.into_bytes()), &self)),
            }
        }
    }

    deserializer.deserialize_str(CowStrVisitor)
}

#[cfg(any(feature = "std", feature = "collections"))]
pub fn borrow_cow_bytes<'de, D>(deserializer: D) -> Result<Cow<'de, [u8]>, D::Error>
    where D: Deserializer<'de>
{
    struct CowBytesVisitor;

    impl<'a> Visitor<'a> for CowBytesVisitor {
        type Value = Cow<'a, [u8]>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a byte array")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(Cow::Owned(v.as_bytes().to_vec()))
        }

        fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(Cow::Borrowed(v.as_bytes()))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(Cow::Owned(v.into_bytes()))
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(Cow::Owned(v.to_vec()))
        }

        fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(Cow::Borrowed(v))
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(Cow::Owned(v))
        }
    }

    deserializer.deserialize_str(CowBytesVisitor)
}
