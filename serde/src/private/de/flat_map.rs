use crate::de::{DeserializeSeed, Deserializer, IgnoredAny, MapAccess, Visitor};
use crate::lib::*;

use super::{Content, ContentDeserializer, ContentVisitor, FlatMapDeserializer};

// The callback consumes named fields and returns the next flattened key.
// Values remain in the original MapAccess, so a flattened map's value seed
// sees the format's deserializer instead of a lossy ContentDeserializer.
pub struct FlatMapAccessAdapter<A, F> {
    map: A,
    next_key: F,
    pending_value: bool,
    ended: bool,
}

impl<A, F> FlatMapAccessAdapter<A, F> {
    pub fn new(map: A, next_key: F) -> Self {
        FlatMapAccessAdapter {
            map,
            next_key,
            pending_value: false,
            ended: false,
        }
    }
}

impl<'de, A, F> FlatMapAccessAdapter<A, F>
where
    A: MapAccess<'de>,
    F: FnMut(&mut A) -> Result<Option<Content<'de>>, A::Error>,
{
    // A custom flattened field may stop without reading the whole map. Still
    // consume the remaining entries, including any named fields after it.
    pub fn finish(&mut self) -> Result<(), A::Error> {
        if self.pending_value {
            tri!(self.next_value::<IgnoredAny>());
        }
        while tri!(self.next_key::<IgnoredAny>()).is_some() {
            tri!(self.next_value::<IgnoredAny>());
        }
        Ok(())
    }
}

impl<'de, A, F> MapAccess<'de> for FlatMapAccessAdapter<A, F>
where
    A: MapAccess<'de>,
    F: FnMut(&mut A) -> Result<Option<Content<'de>>, A::Error>,
{
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.ended {
            return Ok(None);
        }
        match tri!((self.next_key)(&mut self.map)) {
            Some(key) => {
                self.pending_value = true;
                seed.deserialize(ContentDeserializer::new(key)).map(Some)
            }
            None => {
                self.ended = true;
                Ok(None)
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.pending_value = false;
        self.map.next_value_seed(seed)
    }
}

pub struct FlatMapAccessDeserializer<A>(pub A);

impl<'de, A> FlatMapAccessDeserializer<A>
where
    A: MapAccess<'de>,
{
    // Structs and enums select particular keys, and options may suppress a
    // failed attempt. Preserve their existing buffered behavior.
    fn buffer(mut self) -> Result<Vec<Option<(Content<'de>, Content<'de>)>>, A::Error> {
        let mut entries = Vec::new();
        while let Some(entry) = tri!(self
            .0
            .next_entry_seed(ContentVisitor::new(), ContentVisitor::new()))
        {
            entries.push(Some(entry));
        }
        Ok(entries)
    }
}

macro_rules! forward_to_buffered {
    ($($func:ident($($arg:ident: $ty:ty),*))* ) => {
        $(
            fn $func<V>(self, $($arg: $ty,)* visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                FlatMapDeserializer(&mut tri!(self.buffer()), PhantomData)
                    .$func($($arg,)* visitor)
            }
        )*
    };
}

impl<'de, A> Deserializer<'de> for FlatMapAccessDeserializer<A>
where
    A: MapAccess<'de>,
{
    type Error = A::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self.0)
    }

    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_buffered! {
        deserialize_bool()
        deserialize_i8()
        deserialize_i16()
        deserialize_i32()
        deserialize_i64()
        deserialize_i128()
        deserialize_u8()
        deserialize_u16()
        deserialize_u32()
        deserialize_u64()
        deserialize_u128()
        deserialize_f32()
        deserialize_f64()
        deserialize_char()
        deserialize_str()
        deserialize_string()
        deserialize_bytes()
        deserialize_byte_buf()
        deserialize_seq()
        deserialize_tuple(len: usize)
        deserialize_tuple_struct(name: &'static str, len: usize)
        deserialize_identifier()
        deserialize_unit()
        deserialize_unit_struct(name: &'static str)
        deserialize_ignored_any()
        deserialize_option()
        deserialize_struct(name: &'static str, fields: &'static [&'static str])
        deserialize_enum(name: &'static str, variants: &'static [&'static str])
    }
}
