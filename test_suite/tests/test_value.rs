#![allow(clippy::derive_partial_eq_without_eq, clippy::similar_names)]

use serde::de::value::{self, MapAccessDeserializer, MapDeserializer};
use serde::de::{Deserialize, Deserializer, IntoDeserializer, MapAccess, Visitor};
use serde_derive::Deserialize;
use serde_test::{assert_de_tokens, Token};
use std::fmt;

#[test]
fn test_u32_to_enum() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum E {
        A,
        B,
    }

    let deserializer = IntoDeserializer::<value::Error>::into_deserializer(1u32);
    let e: E = E::deserialize(deserializer).unwrap();
    assert_eq!(E::B, e);
}

#[test]
fn test_integer128() {
    let de_u128 = IntoDeserializer::<value::Error>::into_deserializer(1u128);
    let de_i128 = IntoDeserializer::<value::Error>::into_deserializer(1i128);

    // u128 to u128
    assert_eq!(1u128, u128::deserialize(de_u128).unwrap());

    // u128 to i128
    assert_eq!(1i128, i128::deserialize(de_u128).unwrap());

    // i128 to u128
    assert_eq!(1u128, u128::deserialize(de_i128).unwrap());

    // i128 to i128
    assert_eq!(1i128, i128::deserialize(de_i128).unwrap());
}

#[test]
fn test_map_access_to_enum() {
    #[derive(PartialEq, Debug)]
    struct Potential(PotentialKind);

    #[derive(PartialEq, Debug, Deserialize)]
    enum PotentialKind {
        Airebo(Airebo),
    }

    #[derive(PartialEq, Debug, Deserialize)]
    struct Airebo {
        lj_sigma: f64,
    }

    impl<'de> Deserialize<'de> for Potential {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct PotentialVisitor;

            impl<'de> Visitor<'de> for PotentialVisitor {
                type Value = Potential;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a map")
                }

                fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    Deserialize::deserialize(MapAccessDeserializer::new(map)).map(Potential)
                }
            }

            deserializer.deserialize_any(PotentialVisitor)
        }
    }

    let expected = Potential(PotentialKind::Airebo(Airebo { lj_sigma: 14.0 }));

    assert_de_tokens(
        &expected,
        &[
            Token::Map { len: Some(1) },
            Token::Str("Airebo"),
            Token::Map { len: Some(1) },
            Token::Str("lj_sigma"),
            Token::F64(14.0),
            Token::MapEnd,
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_option_from_value_deserializer() {
    // A value deserializer holding a present value must deserialize into
    // `Option<T>` as `Some(value)`, not error with "invalid type ..., expected
    // option".
    let de = IntoDeserializer::<value::Error>::into_deserializer("value");
    let opt = Option::<String>::deserialize(de).unwrap();
    assert_eq!(opt, Some("value".to_owned()));

    let de = IntoDeserializer::<value::Error>::into_deserializer(7u32);
    let opt = Option::<u32>::deserialize(de).unwrap();
    assert_eq!(opt, Some(7));
}

#[test]
fn test_map_deserializer_optional_field() {
    // Regression test for https://github.com/serde-rs/serde/issues/3050
    #[derive(Debug, PartialEq, Deserialize)]
    struct Map {
        optional: Option<String>,
    }

    let iter = [("optional", "value")].into_iter();
    let de = MapDeserializer::<_, value::Error>::new(iter);
    let map = Map::deserialize(de).unwrap();
    assert_eq!(
        map,
        Map {
            optional: Some("value".to_owned()),
        },
    );
}
