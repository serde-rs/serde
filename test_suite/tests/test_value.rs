#![allow(clippy::derive_partial_eq_without_eq, clippy::similar_names)]

use serde::de::value;
use serde::de::{Deserialize, Deserializer, IntoDeserializer, Visitor};
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

mod access_to_enum {
    use super::*;
    use serde::de::value::MapAccessDeserializer;
    use serde::de::MapAccess;

    #[derive(PartialEq, Debug)]
    struct UseAccess(Enum);

    #[derive(PartialEq, Debug, Deserialize)]
    enum Enum {
        Newtype(Airebo),
    }

    #[derive(PartialEq, Debug, Deserialize)]
    struct Airebo {
        lj_sigma: f64,
    }

    impl<'de> Deserialize<'de> for UseAccess {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct UseAccessVisitor;

            impl<'de> Visitor<'de> for UseAccessVisitor {
                type Value = UseAccess;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a map")
                }

                fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    Deserialize::deserialize(MapAccessDeserializer::new(map)).map(UseAccess)
                }
            }

            deserializer.deserialize_any(UseAccessVisitor)
        }
    }

    /// Because [`serde_test::de::Deserializer`] handles both tokens [`Token::Map`]
    /// and [`Token::Struct`] the same, we test only `Token::Map` tokens here.
    mod map {
        use super::*;

        #[test]
        fn newtype() {
            assert_de_tokens(
                &UseAccess(Enum::Newtype(Airebo { lj_sigma: 14.0 })),
                &[
                    Token::Map { len: Some(1) },
                    Token::Str("Newtype"),
                    Token::Map { len: Some(1) },
                    Token::Str("lj_sigma"),
                    Token::F64(14.0),
                    Token::MapEnd,
                    Token::MapEnd,
                ],
            );
        }
    }
}
