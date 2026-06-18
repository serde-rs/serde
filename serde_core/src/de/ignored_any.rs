use crate::lib::*;

use crate::de::{
    Deserialize, DeserializeSeed, Deserializer, EnumAccess, Error, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};

/// An efficient way of discarding data from a deserializer.
///
/// Think of this like `serde_json::Value` in that it can be deserialized from
/// any type, except that it does not store any information about the data that
/// gets deserialized.
///
/// ```edition2021
/// use serde::de::{
///     self, Deserialize, DeserializeSeed, Deserializer, IgnoredAny, SeqAccess, Visitor,
/// };
/// use std::fmt;
/// use std::marker::PhantomData;
///
/// /// A seed that can be used to deserialize only the `n`th element of a sequence
/// /// while efficiently discarding elements of any type before or after index `n`.
/// ///
/// /// For example to deserialize only the element at index 3:
/// ///
/// /// ```
/// /// NthElement::new(3).deserialize(deserializer)
/// /// ```
/// pub struct NthElement<T> {
///     n: usize,
///     marker: PhantomData<T>,
/// }
///
/// impl<T> NthElement<T> {
///     pub fn new(n: usize) -> Self {
///         NthElement {
///             n: n,
///             marker: PhantomData,
///         }
///     }
/// }
///
/// impl<'de, T> Visitor<'de> for NthElement<T>
/// where
///     T: Deserialize<'de>,
/// {
///     type Value = T;
///
///     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
///         write!(
///             formatter,
///             "a sequence in which we care about element {}",
///             self.n
///         )
///     }
///
///     fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
///     where
///         A: SeqAccess<'de>,
///     {
///         // Skip over the first `n` elements.
///         for i in 0..self.n {
///             // It is an error if the sequence ends before we get to element `n`.
///             if seq.next_element::<IgnoredAny>()?.is_none() {
///                 return Err(de::Error::invalid_length(i, &self));
///             }
///         }
///
///         // Deserialize the one we care about.
///         let nth = match seq.next_element()? {
///             Some(nth) => nth,
///             None => {
///                 return Err(de::Error::invalid_length(self.n, &self));
///             }
///         };
///
///         // Skip over any remaining elements in the sequence after `n`.
///         while let Some(IgnoredAny) = seq.next_element()? {
///             // ignore
///         }
///
///         Ok(nth)
///     }
/// }
///
/// impl<'de, T> DeserializeSeed<'de> for NthElement<T>
/// where
///     T: Deserialize<'de>,
/// {
///     type Value = T;
///
///     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
///     where
///         D: Deserializer<'de>,
///     {
///         deserializer.deserialize_seq(self)
///     }
/// }
///
/// # fn example<'de, D>(deserializer: D) -> Result<(), D::Error>
/// # where
/// #     D: Deserializer<'de>,
/// # {
/// // Deserialize only the sequence element at index 3 from this deserializer.
/// // The element at index 3 is required to be a string. Elements before and
/// // after index 3 are allowed to be of any type.
/// let s: String = NthElement::new(3).deserialize(deserializer)?;
/// #     Ok(())
/// # }
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct IgnoredAny;

// Bound the recursion so that ignoring an already constructed, deeply nested
// structure errors out rather than exhausting the stack and aborting.
const RECURSION_LIMIT: u32 = 128;

// The scalar visits are the same whether or not recursion depth is being
// tracked, so both `IgnoredAny` and `IgnoredAnyDepth` share them from here.
macro_rules! ignored_scalars {
    () => {
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
        fn visit_i128<E>(self, x: i128) -> Result<Self::Value, E> {
            let _ = x;
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_u64<E>(self, x: u64) -> Result<Self::Value, E> {
            let _ = x;
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_u128<E>(self, x: u128) -> Result<Self::Value, E> {
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
        fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            let _ = bytes;
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(IgnoredAny)
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(IgnoredAny)
        }
    };
}

#[derive(Copy, Clone)]
struct IgnoredAnyDepth(u32);

impl IgnoredAnyDepth {
    #[inline]
    fn descend<E>(&self) -> Result<u32, E>
    where
        E: Error,
    {
        match self.0.checked_sub(1) {
            Some(remaining) => Ok(remaining),
            None => Err(E::custom("recursion limit exceeded")),
        }
    }
}

impl<'de> DeserializeSeed<'de> for IgnoredAnyDepth {
    type Value = IgnoredAny;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_ignored_any(self)
    }
}

impl<'de> Visitor<'de> for IgnoredAnyDepth {
    type Value = IgnoredAny;

    ignored_scalars!();

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_ignored_any(self)
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_ignored_any(self)
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let seed = IgnoredAnyDepth(tri!(self.descend()));
        while tri!(seq.next_element_seed(seed)).is_some() {
            // Gobble
        }
        Ok(IgnoredAny)
    }

    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let depth = tri!(self.descend());
        while tri!(map.next_key::<IgnoredAny>()).is_some() {
            tri!(map.next_value_seed(IgnoredAnyDepth(depth)));
        }
        Ok(IgnoredAny)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let depth = tri!(self.descend());
        tri!(data.variant::<IgnoredAny>())
            .1
            .newtype_variant_seed(IgnoredAnyDepth(depth))
    }
}

impl<'de> Visitor<'de> for IgnoredAny {
    type Value = IgnoredAny;

    ignored_scalars!();

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_ignored_any(IgnoredAnyDepth(RECURSION_LIMIT))
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_ignored_any(IgnoredAnyDepth(RECURSION_LIMIT))
    }

    #[inline]
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        IgnoredAnyDepth(RECURSION_LIMIT).visit_seq(seq)
    }

    #[inline]
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        IgnoredAnyDepth(RECURSION_LIMIT).visit_map(map)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        IgnoredAnyDepth(RECURSION_LIMIT).visit_enum(data)
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
