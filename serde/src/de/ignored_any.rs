use lib::*;

use de::{Deserialize, Deserializer, Visitor, SeqVisitor, MapVisitor, Error};

/// A target for deserializers that want to ignore data. Implements Deserialize
/// and silently eats data given to it.
#[derive(Copy, Clone, Debug, Default)]
pub struct IgnoredAny;

impl<'de> Deserialize<'de> for IgnoredAny {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<IgnoredAny, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IgnoredAnyVisitor;

        impl<'de> Visitor<'de> for IgnoredAnyVisitor {
            type Value = IgnoredAny;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("anything at all")
            }

            #[inline]
            fn visit_bool<E>(self, _: bool) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_i64<E>(self, _: i64) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_u64<E>(self, _: u64) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_f64<E>(self, _: f64) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_str<E>(self, _: &str) -> Result<IgnoredAny, E>
            where
                E: Error,
            {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<IgnoredAny, D::Error>
            where
                D: Deserializer<'de>,
            {
                IgnoredAny::deserialize(deserializer)
            }

            #[inline]
            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<IgnoredAny, D::Error>
            where
                D: Deserializer<'de>,
            {
                IgnoredAny::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<IgnoredAny, V::Error>
            where
                V: SeqVisitor<'de>,
            {
                while let Some(_) = try!(visitor.visit::<IgnoredAny>()) {
                    // Gobble
                }
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<IgnoredAny, V::Error>
            where
                V: MapVisitor<'de>,
            {
                while let Some((_, _)) = try!(visitor.visit::<IgnoredAny, IgnoredAny>()) {
                    // Gobble
                }
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_bytes<E>(self, _: &[u8]) -> Result<IgnoredAny, E>
            where
                E: Error,
            {
                Ok(IgnoredAny)
            }
        }

        deserializer.deserialize_ignored_any(IgnoredAnyVisitor)
    }
}
