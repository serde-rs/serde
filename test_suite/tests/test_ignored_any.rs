#![allow(clippy::derive_partial_eq_without_eq)]

use serde::de::value::{Error, MapDeserializer, SeqDeserializer};
use serde::de::{
    Deserialize, DeserializeSeed, Deserializer, EnumAccess, IgnoredAny, IntoDeserializer,
    VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;
use serde_derive::Deserialize;

#[derive(PartialEq, Debug, Deserialize)]
enum Target {
    Unit,
    Newtype(i32),
    Tuple(i32, i32),
    Struct { a: i32 },
}

struct Enum(&'static str);

impl<'de> Deserializer<'de> for Enum {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[cfg(feature = "unstable")]
    forward_to_deserialize_any! {
        f16 f128
    }
}

impl<'de> EnumAccess<'de> for Enum {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let v = seed.deserialize(self.0.into_deserializer())?;
        Ok((v, self))
    }
}

impl<'de> VariantAccess<'de> for Enum {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(10i32.into_deserializer())
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let seq = SeqDeserializer::new(vec![1i32, 2].into_iter());
        visitor.visit_seq(seq)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let map = MapDeserializer::new(vec![("a", 10i32)].into_iter());
        visitor.visit_map(map)
    }
}

#[test]
fn test_deserialize_enum() {
    // First just make sure the Deserializer impl works
    assert_eq!(Target::Unit, Target::deserialize(Enum("Unit")).unwrap());
    assert_eq!(
        Target::Newtype(10),
        Target::deserialize(Enum("Newtype")).unwrap()
    );
    assert_eq!(
        Target::Tuple(1, 2),
        Target::deserialize(Enum("Tuple")).unwrap()
    );
    assert_eq!(
        Target::Struct { a: 10 },
        Target::deserialize(Enum("Struct")).unwrap()
    );

    // Now try IgnoredAny
    IgnoredAny::deserialize(Enum("Unit")).unwrap();
    IgnoredAny::deserialize(Enum("Newtype")).unwrap();
    IgnoredAny::deserialize(Enum("Tuple")).unwrap();
    IgnoredAny::deserialize(Enum("Struct")).unwrap();
}
