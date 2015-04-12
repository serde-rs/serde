//! Helper module to enable serializing bytes more efficiently

use std::ops;

use ser;
use de;

///////////////////////////////////////////////////////////////////////////////

/// `Bytes` wraps a `&[u8]` in order to serialize into a byte array.
pub struct Bytes<'a> {
    bytes: &'a [u8],
}

impl<'a> From<&'a [u8]> for Bytes<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Bytes {
            bytes: bytes,
        }
    }
}

impl<'a> From<&'a Vec<u8>> for Bytes<'a> {
    fn from(bytes: &'a Vec<u8>) -> Self {
        Bytes {
            bytes: &bytes,
        }
    }
}

impl<'a> ops::Deref for Bytes<'a> {
    type Target = [u8];

    fn deref(&self) -> &[u8] { self.bytes }
}

impl<'a> ser::Serialize for Bytes<'a> {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer
    {
        serializer.visit_bytes(self.bytes)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// `ByteBuf` wraps a `Vec<u8>` in order to hook into serialize and from deserialize a byte array.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ByteBuf {
    bytes: Vec<u8>,
}

impl ByteBuf {
    pub fn new() -> Self {
        ByteBuf {
            bytes: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        ByteBuf {
            bytes: Vec::with_capacity(cap)
        }
    }
}

impl<T> From<T> for ByteBuf where T: Into<Vec<u8>> {
    fn from(bytes: T) -> Self {
        ByteBuf {
            bytes: bytes.into(),
        }
    }
}

impl AsRef<Vec<u8>> for ByteBuf {
    fn as_ref(&self) -> &Vec<u8> {
        &self.bytes
    }
}

impl AsRef<[u8]> for ByteBuf {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl AsMut<Vec<u8>> for ByteBuf {
    fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }
}

impl AsMut<[u8]> for ByteBuf {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl ops::Deref for ByteBuf {
    type Target = [u8];

    fn deref(&self) -> &[u8] { &self.bytes[..] }
}

impl ops::DerefMut for ByteBuf {
    fn deref_mut(&mut self) -> &mut [u8] { &mut self.bytes[..] }
}

impl ser::Serialize for ByteBuf {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer
    {
        serializer.visit_bytes(&self)
    }
}

pub struct ByteBufVisitor;

impl de::Visitor for ByteBufVisitor {
    type Value = ByteBuf;

    #[inline]
    fn visit_unit<E>(&mut self) -> Result<ByteBuf, E>
        where E: de::Error,
    {
        Ok(ByteBuf {
            bytes: Vec::new(),
        })
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<ByteBuf, V::Error>
        where V: de::SeqVisitor,
    {
        let (len, _) = visitor.size_hint();
        let mut values = Vec::with_capacity(len);

        while let Some(value) = try!(visitor.visit()) {
            values.push(value);
        }

        try!(visitor.end());

        Ok(ByteBuf {
            bytes: values,
        })
    }

    #[inline]
    fn visit_bytes<E>(&mut self, v: &[u8]) -> Result<ByteBuf, E>
        where E: de::Error,
    {
        self.visit_byte_buf(v.to_vec())
    }

    #[inline]
    fn visit_byte_buf<E>(&mut self, v: Vec<u8>) -> Result<ByteBuf, E>
        where E: de::Error,
    {
        Ok(ByteBuf {
            bytes: v,
        })
    }
}

impl de::Deserialize for ByteBuf {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<ByteBuf, D::Error>
        where D: de::Deserializer
    {
        deserializer.visit_bytes(ByteBufVisitor)
    }
}
