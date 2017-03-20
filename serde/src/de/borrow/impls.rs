#[cfg(feature = "std")]
use std::borrow::Cow;
#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::borrow::Cow;

#[cfg(feature = "collections")]
use collections::borrow::ToOwned;

use core::fmt;
use core::marker::PhantomData;

use de::{Visitor, Error};
use de::borrow::{DeserializeBorrow, DeserializerBorrow, VisitorBorrow};

impl<'a> DeserializeBorrow<'a> for &'a str {
    fn deserialize_borrow<D>(deserializer: D) -> Result<Self, D::Error>
        where D: DeserializerBorrow<'a>
    {
        struct StrVisitor<'a>(PhantomData<&'a str>);

        impl<'a> Visitor for StrVisitor<'a> {
            type Value = &'a str;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a borrowed string")
            }
        }

        impl<'a> VisitorBorrow<'a> for StrVisitor<'a> {
            fn visit_borrow_str<E>(self, v: &'a str) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(v)
            }
        }

        deserializer.deserialize_borrow_str(StrVisitor(PhantomData))
    }
}

impl<'a> DeserializeBorrow<'a> for &'a [u8] {
    fn deserialize_borrow<D>(deserializer: D) -> Result<Self, D::Error>
        where D: DeserializerBorrow<'a>
    {
        struct BytesVisitor<'a>(PhantomData<&'a [u8]>);

        impl<'a> Visitor for BytesVisitor<'a> {
            type Value = &'a [u8];

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("borrowed bytes")
            }
        }

        impl<'a> VisitorBorrow<'a> for BytesVisitor<'a> {
            fn visit_borrow_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(v)
            }
        }

        deserializer.deserialize_borrow_bytes(BytesVisitor(PhantomData))
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'a, T> DeserializeBorrow<'a> for Cow<'a, T>
    where T: ToOwned,
          &'a T: DeserializeBorrow<'a>
{
    fn deserialize_borrow<D>(deserializer: D) -> Result<Self, D::Error>
        where D: DeserializerBorrow<'a>
    {
        DeserializeBorrow::deserialize_borrow(deserializer).map(Cow::Borrowed)
    }
}
