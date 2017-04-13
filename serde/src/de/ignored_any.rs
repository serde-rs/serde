use lib::*;

use de::{Deserialize, Deserializer, Visitor, SeqVisitor, MapVisitor, Error};

/// A target for deserializers that want to ignore data. Implements Deserialize
/// and silently eats data given to it.
#[derive(Copy, Clone, Debug, Default)]
pub struct IgnoredAny;

impl<'de> Visitor<'de> for IgnoredAny {
    type Value = IgnoredAny;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("anything at all")
    }

    #[inline]
    fn visit_bool<E>(self, x: bool) -> Result<Self::Value, E> {
        let _ = x;
        Ok(IgnoredAny)
    }

    #[inline]
    fn visit_i64<E>(self, x: i64) -> Result<Self::Value, E> {
        let _ = x;
        Ok(IgnoredAny)
    }

    #[inline]
    fn visit_u64<E>(self, x: u64) -> Result<Self::Value, E> {
        let _ = x;
        Ok(IgnoredAny)
    }

    #[inline]
    fn visit_f64<E>(self, x: f64) -> Result<Self::Value, E> {
        let _ = x;
        Ok(IgnoredAny)
    }

    #[inline]
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let _ = s;
        Ok(IgnoredAny)
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(IgnoredAny)
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        IgnoredAny::deserialize(deserializer)
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        IgnoredAny::deserialize(deserializer)
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(IgnoredAny)
    }

    #[inline]
    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: SeqVisitor<'de>,
    {
        while let Some(IgnoredAny) = try!(visitor.visit()) {
            // Gobble
        }
        Ok(IgnoredAny)
    }

    #[inline]
    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: MapVisitor<'de>,
    {
        while let Some((IgnoredAny, IgnoredAny)) = try!(visitor.visit()) {
            // Gobble
        }
        Ok(IgnoredAny)
    }

    #[inline]
    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let _ = bytes;
        Ok(IgnoredAny)
    }
}

impl<'de> Deserialize<'de> for IgnoredAny {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<IgnoredAny, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_ignored_any(IgnoredAny)
    }
}
