#![allow(clippy::derive_partial_eq_without_eq)]

use serde::de::value::{Error, MapAccessDeserializer, MapDeserializer};
use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer, IntoDeserializer, MapAccess, Visitor,
};
use serde_derive::Deserialize;
use serde_test::{assert_de_tokens, Token};
use std::collections::BTreeMap;
use std::fmt;

// A format-specific newtype, like serde_json::value::RawValue, needs the
// original deserializer. deserialize_any deliberately loses its source text.
struct NumberText<'a>(&'a str);

impl<'de> IntoDeserializer<'de, Error> for NumberText<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

impl<'de> Deserializer<'de> for NumberText<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let number = self.0.parse::<f64>().map_err(de::Error::custom)?;
        visitor.visit_f64(number)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if name == "Raw" {
            visitor.visit_borrowed_str(self.0)
        } else {
            visitor.visit_newtype_struct(self)
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple tuple_struct map struct
        enum identifier ignored_any
    }
}

#[derive(Debug, PartialEq)]
struct Raw<'a>(&'a str);

impl<'de> Deserialize<'de> for Raw<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RawVisitor;

        impl<'de> Visitor<'de> for RawVisitor {
            type Value = Raw<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("original number text")
            }

            fn visit_borrowed_str<E>(self, text: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Raw(text))
            }
        }

        deserializer.deserialize_newtype_struct("Raw", RawVisitor)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct Outer<T> {
    known: f64,
    #[serde(flatten)]
    extra: T,
}

#[test]
fn values_use_the_original_deserializer() {
    let text = String::from("1.2300e+02");
    let input = [
        ("first", NumberText(&text)),
        ("known", NumberText("7")),
        ("last", NumberText("-0")),
    ];

    let value = Outer::<BTreeMap<String, Raw>>::deserialize(MapDeserializer::<_, Error>::new(
        input.into_iter(),
    ))
    .unwrap();

    assert_eq!(value.known.to_bits(), 7.0_f64.to_bits());
    assert_eq!(value.extra["first"].0, text);
    assert_eq!(value.extra["first"].0.as_ptr(), text.as_ptr());
    assert_eq!(value.extra["last"].0, "-0");
}

#[test]
fn newtype_map_uses_the_original_deserializer() {
    #[derive(Deserialize)]
    struct Newtype<T>(BTreeMap<String, T>);

    let input = [("raw", NumberText("1.00e+2")), ("known", NumberText("7"))];
    let value =
        Outer::<Newtype<Raw>>::deserialize(MapDeserializer::<_, Error>::new(input.into_iter()))
            .unwrap();

    assert_eq!(value.extra.0["raw"].0, "1.00e+2");
}

#[test]
fn map_without_named_fields_uses_the_original_deserializer() {
    #[derive(Deserialize)]
    struct OnlyMap<T> {
        #[serde(flatten)]
        extra: BTreeMap<String, T>,
    }

    let input = [("raw", NumberText("1.00e+2"))];
    let value =
        OnlyMap::<Raw>::deserialize(MapDeserializer::<_, Error>::new(input.into_iter())).unwrap();
    assert_eq!(value.extra["raw"].0, "1.00e+2");
}

fn first_entry<'de, D>(deserializer: D) -> Result<Option<(String, u32)>, D::Error>
where
    D: Deserializer<'de>,
{
    struct FirstEntry;

    impl<'de> Visitor<'de> for FirstEntry {
        type Value = Option<(String, u32)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            map.next_entry()
        }
    }

    deserializer.deserialize_map(FirstEntry)
}

#[test]
fn finish_consumes_remaining_entries_and_named_fields() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Partial {
        after: u32,
        #[serde(flatten, deserialize_with = "first_entry")]
        first: Option<(String, u32)>,
    }

    assert_de_tokens(
        &Partial {
            after: 3,
            first: Some(("first".to_owned(), 1)),
        },
        &[
            Token::Map { len: Some(3) },
            Token::Str("first"),
            Token::U32(1),
            Token::Str("ignored"),
            Token::Seq { len: Some(1) },
            Token::U32(2),
            Token::SeqEnd,
            Token::Str("after"),
            Token::U32(3),
            Token::MapEnd,
        ],
    );
}

fn first_key<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct FirstKey;

    impl<'de> Visitor<'de> for FirstKey {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            map.next_key()
        }
    }

    deserializer.deserialize_any(FirstKey)
}

#[test]
fn finish_consumes_a_pending_value() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Partial {
        after: u32,
        #[serde(flatten, deserialize_with = "first_key")]
        first: Option<String>,
    }

    assert_de_tokens(
        &Partial {
            after: 2,
            first: Some("first".to_owned()),
        },
        &[
            Token::Map { len: Some(2) },
            Token::Str("first"),
            Token::Map { len: Some(1) },
            Token::Str("nested"),
            Token::U32(1),
            Token::MapEnd,
            Token::Str("after"),
            Token::U32(2),
            Token::MapEnd,
        ],
    );
}

#[test]
fn does_not_read_past_the_end_of_the_map() {
    struct Once<A> {
        inner: A,
        ended: bool,
    }

    impl<'de, A: MapAccess<'de>> MapAccess<'de> for Once<A> {
        type Error = A::Error;

        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where
            K: DeserializeSeed<'de>,
        {
            assert!(!self.ended, "read after end of map");
            let key = self.inner.next_key_seed(seed)?;
            self.ended = key.is_none();
            Ok(key)
        }

        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where
            V: DeserializeSeed<'de>,
        {
            self.inner.next_value_seed(seed)
        }
    }

    let input = [("extra", NumberText("3")), ("known", NumberText("7"))];
    let deserializer = MapAccessDeserializer::new(Once {
        inner: MapDeserializer::<_, Error>::new(input.into_iter()),
        ended: false,
    });
    let value = Outer::<BTreeMap<String, f64>>::deserialize(deserializer).unwrap();
    assert_eq!(value.extra["extra"].to_bits(), 3.0_f64.to_bits());
}

#[test]
fn aliases_defaults_and_skipped_fields() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Attributes {
        #[serde(rename = "wire", alias = "alias")]
        renamed: u32,
        #[serde(default)]
        defaulted: u32,
        #[serde(skip)]
        skipped: u32,
        #[serde(flatten)]
        extra: BTreeMap<String, u32>,
    }

    assert_de_tokens(
        &Attributes {
            renamed: 2,
            defaulted: 0,
            skipped: 0,
            extra: BTreeMap::from([("first".to_owned(), 1), ("last".to_owned(), 3)]),
        },
        &[
            Token::Map { len: Some(3) },
            Token::Str("first"),
            Token::U32(1),
            Token::Str("alias"),
            Token::U32(2),
            Token::Str("last"),
            Token::U32(3),
            Token::MapEnd,
        ],
    );
}

#[test]
fn duplicate_unknown_keys_keep_map_semantics() {
    let input = [
        ("extra", NumberText("1.00")),
        ("known", NumberText("7")),
        ("extra", NumberText("2.00")),
    ];
    let value = Outer::<BTreeMap<String, Raw>>::deserialize(MapDeserializer::<_, Error>::new(
        input.into_iter(),
    ))
    .unwrap();
    assert_eq!(value.extra.len(), 1);
    assert_eq!(value.extra["extra"].0, "2.00");
}
