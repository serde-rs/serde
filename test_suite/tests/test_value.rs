#![allow(clippy::derive_partial_eq_without_eq, clippy::similar_names)]

use serde::de::value;
use serde::de::{Deserialize, Deserializer, IntoDeserializer, Visitor};
use serde_derive::Deserialize;
use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};
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
        Unit,
        Newtype(Airebo),
        Tuple(String, f64),
        Struct { lj_sigma: f64 },
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
        fn unit() {
            assert_de_tokens(
                &UseAccess(Enum::Unit),
                &[
                    Token::Map { len: Some(1) },
                    Token::Str("Unit"),
                    Token::Unit,
                    Token::MapEnd,
                ],
            );
        }

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

        #[test]
        fn tuple() {
            assert_de_tokens(
                &UseAccess(Enum::Tuple("lj_sigma".to_string(), 14.0)),
                &[
                    Token::Map { len: Some(1) },
                    Token::Str("Tuple"),
                    Token::Seq { len: Some(2) },
                    Token::Str("lj_sigma"),
                    Token::F64(14.0),
                    Token::SeqEnd,
                    Token::MapEnd,
                ],
            );
        }

        #[test]
        fn struct_() {
            assert_de_tokens(
                &UseAccess(Enum::Struct { lj_sigma: 14.0 }),
                &[
                    Token::Map { len: Some(1) },
                    Token::Str("Struct"),
                    Token::Map { len: Some(1) },
                    Token::Str("lj_sigma"),
                    Token::F64(14.0),
                    Token::MapEnd,
                    Token::MapEnd,
                ],
            );
        }

        #[test]
        fn wrong_tag() {
            assert_de_tokens_error::<UseAccess>(
                &[
                    Token::Map { len: Some(1) },
                    Token::Str("AnotherTag"),
                    Token::Map { len: Some(1) },
                    // Tokens that could follow, but assert_de_tokens_error do not want them
                    // Token::Str("lj_sigma"),
                    // Token::F64(14.0),
                    // Token::MapEnd,
                    // Token::MapEnd,
                ],
                "unknown variant `AnotherTag`, expected one of `Unit`, `Newtype`, `Tuple`, `Struct`",
            );
        }

        #[test]
        fn empty_map() {
            assert_de_tokens_error::<UseAccess>(
                &[Token::Map { len: Some(0) }, Token::MapEnd],
                "invalid type: map, expected enum",
            );
        }
    }
}
