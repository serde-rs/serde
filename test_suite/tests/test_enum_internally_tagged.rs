#![deny(trivial_numeric_casts)]
#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::enum_variant_names,
    clippy::redundant_field_names,
    clippy::too_many_lines
)]

mod bytes;

use self::bytes::{ByteBuf, Bytes};
use serde_derive::{Deserialize, Serialize};
use serde_test::{
    assert_de_tokens, assert_de_tokens_error, assert_ser_tokens_error, assert_tokens, Token,
};
use std::collections::BTreeMap;
use std::iter::FromIterator;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Unit;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Newtype<T>(T);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Tuple(u8, u8);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Struct {
    i128_: i128,
    u128_: u128,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Enum {
    Unit,
    Newtype(u128),
    Tuple(i128, u128),
    Struct { i128_: i128, u128_: u128 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tag")]
enum InternallyTagged<'a> {
    Unit,

    NewtypeI8(i8),
    NewtypeI16(i16),
    NewtypeI32(i32),
    NewtypeI64(i64),
    NewtypeI128(i128),
    NewtypeIsize(isize),

    NewtypeU8(u8),
    NewtypeU16(u16),
    NewtypeU32(u32),
    NewtypeU64(u64),
    NewtypeU128(u128),
    NewtypeUsize(usize),

    NewtypeF32(f32),
    NewtypeF64(f64),

    NewtypeBool(bool),
    NewtypeChar(char),

    NewtypeStr(&'a str),
    NewtypeString(String),

    NewtypeBytes(Bytes<'a>),
    NewtypeByteBuf(ByteBuf),

    NewtypeUnit(()),
    NewtypeUnitStruct(Unit),

    NewtypeNewtypeU8(Newtype<u8>),
    NewtypeNewtypeStruct(Newtype<Struct>),

    NewtypeTuple((u8, u8)),
    NewtypeTupleStruct(Tuple),

    NewtypeMap(BTreeMap<String, String>),
    NewtypeStruct(Struct),
    NewtypeEnum(Enum),

    Struct { i128_: i128, u128_: u128 },
    StructEnum { enum_: Enum },
}

#[test]
fn unit() {
    assert_tokens(
        &InternallyTagged::Unit,
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 1,
            },
            Token::Str("tag"),
            Token::Str("Unit"),
            Token::StructEnd,
        ],
    );
    assert_de_tokens(
        &InternallyTagged::Unit,
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 1,
            },
            Token::BorrowedStr("tag"),
            Token::BorrowedStr("Unit"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::Unit,
        &[
            Token::Map { len: Some(1) },
            Token::Str("tag"),
            Token::Str("Unit"),
            Token::MapEnd,
        ],
    );
    assert_de_tokens(
        &InternallyTagged::Unit,
        &[
            Token::Map { len: Some(1) },
            Token::BorrowedStr("tag"),
            Token::BorrowedStr("Unit"),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::Unit,
        &[
            Token::Seq { len: Some(1) },
            Token::Str("Unit"), // tag
            Token::SeqEnd,
        ],
    );
    assert_de_tokens(
        &InternallyTagged::Unit,
        &[
            Token::Seq { len: Some(1) },
            Token::BorrowedStr("Unit"), // tag
            Token::SeqEnd,
        ],
    );
}

mod newtype {
    use super::*;

    macro_rules! failed {
        ($test:ident, $name:ident $init:tt, $message:literal) => {
            #[test]
            fn $test() {
                assert_ser_tokens_error(
                    &InternallyTagged::$name $init,
                    &[],
                    $message
                );
            }
        };
    }

    failed!(
        i8,
        NewtypeI8(42),
        "cannot serialize tagged newtype variant InternallyTagged::NewtypeI8 containing an integer"
    );
    failed!(i16, NewtypeI16(42), "cannot serialize tagged newtype variant InternallyTagged::NewtypeI16 containing an integer");
    failed!(i32, NewtypeI32(42), "cannot serialize tagged newtype variant InternallyTagged::NewtypeI32 containing an integer");
    failed!(i64, NewtypeI64(42), "cannot serialize tagged newtype variant InternallyTagged::NewtypeI64 containing an integer");
    failed!(i128, NewtypeI128(42), "cannot serialize tagged newtype variant InternallyTagged::NewtypeI128 containing an integer");
    failed!(isize, NewtypeIsize(42), "cannot serialize tagged newtype variant InternallyTagged::NewtypeIsize containing an integer");

    failed!(
        u8,
        NewtypeU8(42),
        "cannot serialize tagged newtype variant InternallyTagged::NewtypeU8 containing an integer"
    );
    failed!(u16, NewtypeU16(42), "cannot serialize tagged newtype variant InternallyTagged::NewtypeU16 containing an integer");
    failed!(u32, NewtypeU32(42), "cannot serialize tagged newtype variant InternallyTagged::NewtypeU32 containing an integer");
    failed!(u64, NewtypeU64(42), "cannot serialize tagged newtype variant InternallyTagged::NewtypeU64 containing an integer");
    failed!(u128, NewtypeU128(42), "cannot serialize tagged newtype variant InternallyTagged::NewtypeU128 containing an integer");
    failed!(usize, NewtypeUsize(42), "cannot serialize tagged newtype variant InternallyTagged::NewtypeUsize containing an integer");

    failed!(
        f32,
        NewtypeF32(4.2),
        "cannot serialize tagged newtype variant InternallyTagged::NewtypeF32 containing a float"
    );
    failed!(
        f64,
        NewtypeF64(4.2),
        "cannot serialize tagged newtype variant InternallyTagged::NewtypeF64 containing a float"
    );

    failed!(bool_, NewtypeBool(true), "cannot serialize tagged newtype variant InternallyTagged::NewtypeBool containing a boolean");
    failed!(
        char_,
        NewtypeChar('x'),
        "cannot serialize tagged newtype variant InternallyTagged::NewtypeChar containing a char"
    );

    failed!(
        str,
        NewtypeStr("string"),
        "cannot serialize tagged newtype variant InternallyTagged::NewtypeStr containing a string"
    );
    failed!(string, NewtypeString("string".to_owned()), "cannot serialize tagged newtype variant InternallyTagged::NewtypeString containing a string");

    failed!(bytes, NewtypeBytes(Bytes(b"string")), "cannot serialize tagged newtype variant InternallyTagged::NewtypeBytes containing a byte array");
    failed!(byte_buf, NewtypeByteBuf(ByteBuf(b"string".to_vec())), "cannot serialize tagged newtype variant InternallyTagged::NewtypeByteBuf containing a byte array");

    #[test]
    fn unit() {
        let value = InternallyTagged::NewtypeUnit(());

        assert_tokens(
            &value,
            &[
                Token::Map { len: Some(1) },
                Token::Str("tag"),
                Token::Str("NewtypeUnit"),
                Token::MapEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Map { len: Some(1) },
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("NewtypeUnit"),
                Token::MapEnd,
            ],
        );

        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "InternallyTagged",
                    len: 1,
                },
                Token::Str("tag"),
                Token::Str("NewtypeUnit"),
                Token::StructEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "InternallyTagged",
                    len: 1,
                },
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("NewtypeUnit"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn unit_struct() {
        let value = InternallyTagged::NewtypeUnitStruct(Unit);

        assert_tokens(
            &value,
            &[
                Token::Map { len: Some(1) },
                Token::Str("tag"),
                Token::Str("NewtypeUnitStruct"),
                Token::MapEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Map { len: Some(1) },
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("NewtypeUnitStruct"),
                Token::MapEnd,
            ],
        );

        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "InternallyTagged",
                    len: 1,
                },
                Token::Str("tag"),
                Token::Str("NewtypeUnitStruct"),
                Token::StructEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "InternallyTagged",
                    len: 1,
                },
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("NewtypeUnitStruct"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: Some(1) },
                Token::Str("NewtypeUnitStruct"), // tag
                Token::SeqEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: Some(1) },
                Token::BorrowedStr("NewtypeUnitStruct"), // tag
                Token::SeqEnd,
            ],
        );
    }

    failed!(newtype_u8, NewtypeNewtypeU8(Newtype(42)), "cannot serialize tagged newtype variant InternallyTagged::NewtypeNewtypeU8 containing an integer");

    #[test]
    fn newtype_struct() {
        assert_tokens(
            &InternallyTagged::NewtypeNewtypeStruct(Newtype(Struct { i128_: 4, u128_: 2 })),
            &[
                Token::Struct {
                    name: "Struct",
                    len: 3,
                },
                Token::Str("tag"),
                Token::Str("NewtypeNewtypeStruct"),
                Token::Str("i128_"),
                Token::I128(4),
                Token::Str("u128_"),
                Token::U128(2),
                Token::StructEnd,
            ],
        );
    }

    failed!(
        tuple,
        NewtypeTuple((4, 2)),
        "cannot serialize tagged newtype variant InternallyTagged::NewtypeTuple containing a tuple"
    );
    failed!(tuple_struct, NewtypeTupleStruct(Tuple(4, 2)), "cannot serialize tagged newtype variant InternallyTagged::NewtypeTupleStruct containing a tuple struct");

    #[test]
    fn map() {
        let value = InternallyTagged::NewtypeMap(BTreeMap::new());

        // Special case: empty map
        assert_tokens(
            &value,
            &[
                Token::Map { len: Some(1) },
                Token::Str("tag"),
                Token::Str("NewtypeMap"),
                Token::MapEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Map { len: Some(1) },
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("NewtypeMap"),
                Token::MapEnd,
            ],
        );

        let value = InternallyTagged::NewtypeMap(BTreeMap::from_iter([(
            "field".to_string(),
            "value".to_string(),
        )]));

        // Special case: tag field ("tag") is the first field
        assert_tokens(
            &value,
            &[
                Token::Map { len: Some(2) },
                Token::Str("tag"),
                Token::Str("NewtypeMap"),
                Token::Str("field"),
                Token::Str("value"),
                Token::MapEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Map { len: Some(2) },
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("NewtypeMap"),
                Token::BorrowedStr("field"),
                Token::BorrowedStr("value"),
                Token::MapEnd,
            ],
        );

        // General case: tag field ("tag") is not the first field
        assert_de_tokens(
            &value,
            &[
                Token::Map { len: Some(2) },
                Token::Str("field"),
                Token::Str("value"),
                Token::Str("tag"),
                Token::Str("NewtypeMap"),
                Token::MapEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Map { len: Some(2) },
                Token::BorrowedStr("field"),
                Token::BorrowedStr("value"),
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("NewtypeMap"),
                Token::MapEnd,
            ],
        );

        assert_de_tokens_error::<InternallyTagged>(
            &[
                Token::Seq { len: Some(2) },
                Token::Str("NewtypeMap"), // tag
                Token::Map { len: Some(0) },
                Token::MapEnd,
                Token::SeqEnd,
            ],
            "invalid type: sequence, expected a map",
        );
    }

    #[test]
    fn struct_() {
        let value = InternallyTagged::NewtypeStruct(Struct { i128_: 4, u128_: 2 });

        // Special case: tag field ("tag") is the first field
        assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Struct",
                    len: 3,
                },
                Token::Str("tag"),
                Token::Str("NewtypeStruct"),
                Token::Str("i128_"),
                Token::I128(4),
                Token::Str("u128_"),
                Token::U128(2),
                Token::StructEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Struct",
                    len: 3,
                },
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("NewtypeStruct"),
                Token::BorrowedStr("i128_"),
                Token::I128(4),
                Token::BorrowedStr("u128_"),
                Token::U128(2),
                Token::StructEnd,
            ],
        );

        // General case: tag field ("tag") is not the first field
        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Struct",
                    len: 3,
                },
                Token::Str("u128_"),
                Token::U128(2),
                Token::Str("tag"),
                Token::Str("NewtypeStruct"),
                Token::Str("i128_"),
                Token::I128(4),
                Token::StructEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Struct",
                    len: 3,
                },
                Token::BorrowedStr("u128_"),
                Token::U128(2),
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("NewtypeStruct"),
                Token::BorrowedStr("i128_"),
                Token::I128(4),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: Some(2) },
                Token::Str("NewtypeStruct"), // tag
                Token::I128(4),
                Token::U128(2),
                Token::SeqEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: Some(2) },
                Token::BorrowedStr("NewtypeStruct"), // tag
                Token::I128(4),
                Token::U128(2),
                Token::SeqEnd,
            ],
        );
    }

    mod enum_ {
        use super::*;

        #[test]
        fn unit() {
            let value = InternallyTagged::NewtypeEnum(Enum::Unit);

            // Special case: tag field ("tag") is the first field
            assert_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::Str("tag"),
                    Token::Str("NewtypeEnum"),
                    Token::Str("Unit"),
                    Token::Unit,
                    Token::MapEnd,
                ],
            );
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::BorrowedStr("tag"),
                    Token::BorrowedStr("NewtypeEnum"),
                    Token::BorrowedStr("Unit"),
                    Token::Unit,
                    Token::MapEnd,
                ],
            );

            // General case: tag field ("tag") is not the first field
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::Str("Unit"),
                    Token::Unit,
                    Token::Str("tag"),
                    Token::Str("NewtypeEnum"),
                    Token::MapEnd,
                ],
            );
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::BorrowedStr("Unit"),
                    Token::Unit,
                    Token::BorrowedStr("tag"),
                    Token::BorrowedStr("NewtypeEnum"),
                    Token::MapEnd,
                ],
            );
        }

        #[test]
        fn newtype() {
            let value = InternallyTagged::NewtypeEnum(Enum::Newtype(1));

            // Special case: tag field ("tag") is the first field
            assert_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::Str("tag"),
                    Token::Str("NewtypeEnum"),
                    Token::Str("Newtype"),
                    Token::U128(1),
                    Token::MapEnd,
                ],
            );
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::BorrowedStr("tag"),
                    Token::BorrowedStr("NewtypeEnum"),
                    Token::BorrowedStr("Newtype"),
                    Token::U128(1),
                    Token::MapEnd,
                ],
            );

            // General case: tag field ("tag") is not the first field
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::Str("Newtype"),
                    Token::U128(1),
                    Token::Str("tag"),
                    Token::Str("NewtypeEnum"),
                    Token::MapEnd,
                ],
            );
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::BorrowedStr("Newtype"),
                    Token::U128(1),
                    Token::BorrowedStr("tag"),
                    Token::BorrowedStr("NewtypeEnum"),
                    Token::MapEnd,
                ],
            );
        }

        #[test]
        fn tuple() {
            let value = InternallyTagged::NewtypeEnum(Enum::Tuple(4, 2));

            // Special case: tag field ("tag") is the first field
            assert_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::Str("tag"),
                    Token::Str("NewtypeEnum"),
                    Token::Str("Tuple"),
                    Token::TupleStruct {
                        name: "Tuple",
                        len: 2,
                    },
                    Token::I128(4),
                    Token::U128(2),
                    Token::TupleStructEnd,
                    Token::MapEnd,
                ],
            );
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::BorrowedStr("tag"),
                    Token::BorrowedStr("NewtypeEnum"),
                    Token::BorrowedStr("Tuple"),
                    Token::TupleStruct {
                        name: "Tuple",
                        len: 2,
                    },
                    Token::I128(4),
                    Token::U128(2),
                    Token::TupleStructEnd,
                    Token::MapEnd,
                ],
            );

            // Special case: tag field ("tag") is not the first field
            // Reaches crate::private::de::content::VariantDeserializer::tuple_variant
            // Content::Seq case
            // via ContentDeserializer::deserialize_enum
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::Str("Tuple"),
                    Token::TupleStruct {
                        name: "Tuple",
                        len: 2,
                    },
                    Token::I128(4),
                    Token::U128(2),
                    Token::TupleStructEnd,
                    Token::Str("tag"),
                    Token::Str("NewtypeEnum"),
                    Token::MapEnd,
                ],
            );
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::BorrowedStr("Tuple"),
                    Token::TupleStruct {
                        name: "Tuple",
                        len: 2,
                    },
                    Token::I128(4),
                    Token::U128(2),
                    Token::TupleStructEnd,
                    Token::BorrowedStr("tag"),
                    Token::BorrowedStr("NewtypeEnum"),
                    Token::MapEnd,
                ],
            );
        }

        #[test]
        fn struct_() {
            let value = InternallyTagged::NewtypeEnum(Enum::Struct { i128_: 4, u128_: 2 });

            // Special case: tag field ("tag") is the first field
            assert_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::Str("tag"),
                    Token::Str("NewtypeEnum"),
                    Token::Str("Struct"),
                    Token::Struct {
                        name: "Struct",
                        len: 2,
                    },
                    Token::Str("i128_"),
                    Token::I128(4),
                    Token::Str("u128_"),
                    Token::U128(2),
                    Token::StructEnd,
                    Token::MapEnd,
                ],
            );
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::BorrowedStr("tag"),
                    Token::BorrowedStr("NewtypeEnum"),
                    Token::BorrowedStr("Struct"),
                    Token::Struct {
                        name: "Struct",
                        len: 2,
                    },
                    Token::BorrowedStr("i128_"),
                    Token::I128(4),
                    Token::BorrowedStr("u128_"),
                    Token::U128(2),
                    Token::StructEnd,
                    Token::MapEnd,
                ],
            );

            // General case: tag field ("tag") is not the first field
            // Reaches crate::private::de::content::VariantDeserializer::struct_variant
            // Content::Map case
            // via ContentDeserializer::deserialize_enum
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::Str("Struct"),
                    Token::Struct {
                        name: "Struct",
                        len: 2,
                    },
                    Token::Str("i128_"),
                    Token::I128(4),
                    Token::Str("u128_"),
                    Token::U128(2),
                    Token::StructEnd,
                    Token::Str("tag"),
                    Token::Str("NewtypeEnum"),
                    Token::MapEnd,
                ],
            );
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::BorrowedStr("Struct"),
                    Token::Struct {
                        name: "Struct",
                        len: 2,
                    },
                    Token::BorrowedStr("i128_"),
                    Token::I128(4),
                    Token::BorrowedStr("u128_"),
                    Token::U128(2),
                    Token::StructEnd,
                    Token::BorrowedStr("tag"),
                    Token::BorrowedStr("NewtypeEnum"),
                    Token::MapEnd,
                ],
            );

            // Special case: tag field ("tag") is the first field
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::Str("tag"),
                    Token::Str("NewtypeEnum"),
                    Token::Str("Struct"),
                    Token::Seq { len: Some(2) },
                    Token::I128(4), // i128_
                    Token::U128(2), // u128_
                    Token::SeqEnd,
                    Token::MapEnd,
                ],
            );
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::BorrowedStr("tag"),
                    Token::BorrowedStr("NewtypeEnum"),
                    Token::BorrowedStr("Struct"),
                    Token::Seq { len: Some(2) },
                    Token::I128(4), // i128_
                    Token::U128(2), // u128_
                    Token::SeqEnd,
                    Token::MapEnd,
                ],
            );

            // General case: tag field ("tag") is not the first field
            // Reaches crate::private::de::content::VariantDeserializer::struct_variant
            // Content::Seq case
            // via ContentDeserializer::deserialize_enum
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::Str("Struct"),
                    Token::Seq { len: Some(2) },
                    Token::I128(4), // i128_
                    Token::U128(2), // u128_
                    Token::SeqEnd,
                    Token::Str("tag"),
                    Token::Str("NewtypeEnum"),
                    Token::MapEnd,
                ],
            );
            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(2) },
                    Token::BorrowedStr("Struct"),
                    Token::Seq { len: Some(2) },
                    Token::I128(4), // i128_
                    Token::U128(2), // u128_
                    Token::SeqEnd,
                    Token::BorrowedStr("tag"),
                    Token::BorrowedStr("NewtypeEnum"),
                    Token::MapEnd,
                ],
            );
        }
    }
}

#[test]
fn struct_() {
    let value = InternallyTagged::Struct { i128_: 4, u128_: 2 };

    // Special case: tag field ("tag") is the first field
    assert_tokens(
        &value,
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 3,
            },
            Token::Str("tag"),
            Token::Str("Struct"),
            Token::Str("i128_"),
            Token::I128(4),
            Token::Str("u128_"),
            Token::U128(2),
            Token::StructEnd,
        ],
    );
    assert_de_tokens(
        &value,
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 3,
            },
            Token::BorrowedStr("tag"),
            Token::BorrowedStr("Struct"),
            Token::BorrowedStr("i128_"),
            Token::I128(4),
            Token::BorrowedStr("u128_"),
            Token::U128(2),
            Token::StructEnd,
        ],
    );

    // General case: tag field ("tag") is not the first field, struct fields not in order
    assert_de_tokens(
        &value,
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 3,
            },
            Token::Str("u128_"),
            Token::U128(2),
            Token::Str("i128_"),
            Token::I128(4),
            Token::Str("tag"),
            Token::Str("Struct"),
            Token::StructEnd,
        ],
    );
    assert_de_tokens(
        &value,
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 3,
            },
            Token::BorrowedStr("u128_"),
            Token::U128(2),
            Token::BorrowedStr("tag"),
            Token::BorrowedStr("Struct"),
            Token::BorrowedStr("i128_"),
            Token::I128(4),
            Token::StructEnd,
        ],
    );

    // Special case: tag field ("tag") is the first field
    assert_de_tokens(
        &value,
        &[
            Token::Map { len: Some(2) },
            Token::Str("tag"),
            Token::Str("Struct"),
            Token::Str("i128_"),
            Token::I128(4),
            Token::Str("u128_"),
            Token::U128(2),
            Token::MapEnd,
        ],
    );
    assert_de_tokens(
        &value,
        &[
            Token::Map { len: Some(2) },
            Token::BorrowedStr("tag"),
            Token::BorrowedStr("Struct"),
            Token::BorrowedStr("i128_"),
            Token::I128(4),
            Token::BorrowedStr("u128_"),
            Token::U128(2),
            Token::MapEnd,
        ],
    );

    // General case: tag field ("tag") is not the first field
    assert_de_tokens(
        &value,
        &[
            Token::Map { len: Some(3) },
            Token::Str("u128_"),
            Token::U128(2),
            Token::Str("tag"),
            Token::Str("Struct"),
            Token::Str("i128_"),
            Token::I128(4),
            Token::MapEnd,
        ],
    );
    assert_de_tokens(
        &value,
        &[
            Token::Map { len: Some(2) },
            Token::BorrowedStr("u128_"),
            Token::U128(2),
            Token::BorrowedStr("tag"),
            Token::BorrowedStr("Struct"),
            Token::BorrowedStr("i128_"),
            Token::I128(4),
            Token::MapEnd,
        ],
    );

    assert_de_tokens(
        &value,
        &[
            Token::Seq { len: Some(3) },
            Token::Str("Struct"), // tag
            Token::I128(4),
            Token::U128(2),
            Token::SeqEnd,
        ],
    );
    assert_de_tokens(
        &value,
        &[
            Token::Seq { len: Some(3) },
            Token::BorrowedStr("Struct"), // tag
            Token::I128(4),
            Token::U128(2),
            Token::SeqEnd,
        ],
    );
}

mod struct_enum {
    use super::*;

    #[test]
    fn unit() {
        assert_de_tokens(
            &Enum::Unit,
            &[
                Token::Enum { name: "Enum" },
                Token::BorrowedStr("Unit"),
                Token::Unit,
            ],
        );

        let value = InternallyTagged::StructEnum { enum_: Enum::Unit };

        // Special case: tag field ("tag") is the first field
        assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "InternallyTagged",
                    len: 2,
                },
                Token::Str("tag"),
                Token::Str("StructEnum"),
                Token::Str("enum_"),
                Token::Enum { name: "Enum" },
                Token::Str("Unit"),
                Token::Unit,
                Token::StructEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "InternallyTagged",
                    len: 2,
                },
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("StructEnum"),
                Token::BorrowedStr("enum_"),
                Token::Enum { name: "Enum" },
                Token::BorrowedStr("Unit"),
                Token::Unit,
                Token::StructEnd,
            ],
        );

        // General case: tag field ("tag") is not the first field
        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "InternallyTagged",
                    len: 2,
                },
                Token::Str("enum_"),
                Token::Enum { name: "Enum" },
                Token::Str("Unit"),
                Token::Unit,
                Token::Str("tag"),
                Token::Str("StructEnum"),
                Token::StructEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "InternallyTagged",
                    len: 2,
                },
                Token::BorrowedStr("enum_"),
                Token::Enum { name: "Enum" },
                Token::BorrowedStr("Unit"),
                Token::Unit,
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("StructEnum"),
                Token::StructEnd,
            ],
        );

        // Special case: tag field ("tag") is the first field
        assert_de_tokens(
            &value,
            &[
                Token::Map { len: Some(2) },
                Token::Str("tag"),
                Token::Str("StructEnum"),
                Token::Str("enum_"),
                Token::Enum { name: "Enum" },
                Token::Str("Unit"),
                Token::Unit,
                Token::MapEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Map { len: Some(2) },
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("StructEnum"),
                Token::BorrowedStr("enum_"),
                Token::Enum { name: "Enum" },
                Token::BorrowedStr("Unit"),
                Token::Unit,
                Token::MapEnd,
            ],
        );

        // General case: tag field ("tag") is not the first field
        assert_de_tokens(
            &value,
            &[
                Token::Map { len: Some(2) },
                Token::Str("enum_"),
                Token::Enum { name: "Enum" },
                Token::Str("Unit"),
                Token::Unit,
                Token::Str("tag"),
                Token::Str("StructEnum"),
                Token::MapEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Map { len: Some(2) },
                Token::BorrowedStr("enum_"),
                Token::Enum { name: "Enum" },
                Token::BorrowedStr("Unit"),
                Token::Unit,
                Token::BorrowedStr("tag"),
                Token::BorrowedStr("StructEnum"),
                Token::MapEnd,
            ],
        );

        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: Some(2) },
                Token::Str("StructEnum"),     // tag
                Token::Enum { name: "Enum" }, // enum_
                Token::Str("Unit"),
                Token::Unit,
                Token::SeqEnd,
            ],
        );
        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: Some(2) },
                Token::BorrowedStr("StructEnum"), // tag
                Token::Enum { name: "Enum" },     // enum_
                Token::BorrowedStr("Unit"),
                Token::Unit,
                Token::SeqEnd,
            ],
        );
    }
}

#[test]
fn wrong_tag() {
    assert_de_tokens_error::<InternallyTagged>(
        &[Token::Map { len: Some(0) }, Token::MapEnd],
        "missing field `tag`",
    );

    assert_de_tokens_error::<InternallyTagged>(
        &[
            Token::Map { len: Some(1) },
            Token::Str("tag"),
            Token::Str("Z"),
            Token::MapEnd,
        ],
        "unknown variant `Z`, expected one of \
        `Unit`, \
        `NewtypeI8`, \
        `NewtypeI16`, \
        `NewtypeI32`, \
        `NewtypeI64`, \
        `NewtypeI128`, \
        `NewtypeIsize`, \
        `NewtypeU8`, \
        `NewtypeU16`, \
        `NewtypeU32`, \
        `NewtypeU64`, \
        `NewtypeU128`, \
        `NewtypeUsize`, \
        `NewtypeF32`, \
        `NewtypeF64`, \
        `NewtypeBool`, \
        `NewtypeChar`, \
        `NewtypeStr`, \
        `NewtypeString`, \
        `NewtypeBytes`, \
        `NewtypeByteBuf`, \
        `NewtypeUnit`, \
        `NewtypeUnitStruct`, \
        `NewtypeNewtypeU8`, \
        `NewtypeNewtypeStruct`, \
        `NewtypeTuple`, \
        `NewtypeTupleStruct`, \
        `NewtypeMap`, \
        `NewtypeStruct`, \
        `NewtypeEnum`, \
        `Struct`, \
        `StructEnum`",
    );
}

#[test]
fn untagged_variant() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "tag")]
    enum InternallyTagged {
        Tagged {
            a: u8,
        },
        #[serde(untagged)]
        Untagged {
            tag: String,
            b: u8,
        },
    }

    assert_de_tokens(
        &InternallyTagged::Tagged { a: 1 },
        &[
            Token::Map { len: Some(2) },
            Token::Str("tag"),
            Token::Str("Tagged"),
            Token::Str("a"),
            Token::U8(1),
            Token::MapEnd,
        ],
    );

    assert_tokens(
        &InternallyTagged::Tagged { a: 1 },
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 2,
            },
            Token::Str("tag"),
            Token::Str("Tagged"),
            Token::Str("a"),
            Token::U8(1),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &InternallyTagged::Untagged {
            tag: "Foo".to_owned(),
            b: 2,
        },
        &[
            Token::Map { len: Some(2) },
            Token::Str("tag"),
            Token::Str("Foo"),
            Token::Str("b"),
            Token::U8(2),
            Token::MapEnd,
        ],
    );

    assert_tokens(
        &InternallyTagged::Untagged {
            tag: "Foo".to_owned(),
            b: 2,
        },
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 2,
            },
            Token::Str("tag"),
            Token::Str("Foo"),
            Token::Str("b"),
            Token::U8(2),
            Token::StructEnd,
        ],
    );

    assert_tokens(
        &InternallyTagged::Untagged {
            tag: "Tagged".to_owned(),
            b: 2,
        },
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 2,
            },
            Token::Str("tag"),
            Token::Str("Tagged"),
            Token::Str("b"),
            Token::U8(2),
            Token::StructEnd,
        ],
    );
}

mod string_and_bytes {
    use super::*;

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(tag = "tag")]
    enum InternallyTagged {
        String {
            string: String,
        },
        Bytes {
            #[serde(with = "bytes")]
            bytes: Vec<u8>,
        },
    }

    #[test]
    fn string_from_string() {
        assert_de_tokens(
            &InternallyTagged::String {
                string: "\0".to_owned(),
            },
            &[
                Token::Struct {
                    name: "String",
                    len: 2,
                },
                Token::Str("tag"),
                Token::Str("String"),
                Token::Str("string"),
                Token::Str("\0"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &InternallyTagged::String {
                string: "\0".to_owned(),
            },
            &[
                Token::Struct {
                    name: "String",
                    len: 2,
                },
                Token::Str("tag"),
                Token::Str("String"),
                Token::Str("string"),
                Token::String("\0"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn string_from_bytes() {
        assert_de_tokens(
            &InternallyTagged::String {
                string: "\0".to_owned(),
            },
            &[
                Token::Struct {
                    name: "String",
                    len: 2,
                },
                Token::Str("tag"),
                Token::Str("String"),
                Token::Str("string"),
                Token::Bytes(b"\0"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &InternallyTagged::String {
                string: "\0".to_owned(),
            },
            &[
                Token::Struct {
                    name: "String",
                    len: 2,
                },
                Token::Str("tag"),
                Token::Str("String"),
                Token::Str("string"),
                Token::ByteBuf(b"\0"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn bytes_from_string() {
        assert_de_tokens(
            &InternallyTagged::Bytes { bytes: vec![0] },
            &[
                Token::Struct {
                    name: "Bytes",
                    len: 2,
                },
                Token::Str("tag"),
                Token::Str("Bytes"),
                Token::Str("bytes"),
                Token::Str("\0"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &InternallyTagged::Bytes { bytes: vec![0] },
            &[
                Token::Struct {
                    name: "Bytes",
                    len: 2,
                },
                Token::Str("tag"),
                Token::Str("Bytes"),
                Token::Str("bytes"),
                Token::String("\0"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn bytes_from_bytes() {
        assert_de_tokens(
            &InternallyTagged::Bytes { bytes: vec![0] },
            &[
                Token::Struct {
                    name: "Bytes",
                    len: 2,
                },
                Token::Str("tag"),
                Token::Str("Bytes"),
                Token::Str("bytes"),
                Token::Bytes(b"\0"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &InternallyTagged::Bytes { bytes: vec![0] },
            &[
                Token::Struct {
                    name: "Bytes",
                    len: 2,
                },
                Token::Str("tag"),
                Token::Str("Bytes"),
                Token::Str("bytes"),
                Token::ByteBuf(b"\0"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn bytes_from_seq() {
        assert_de_tokens(
            &InternallyTagged::Bytes { bytes: vec![0] },
            &[
                Token::Struct {
                    name: "Bytes",
                    len: 2,
                },
                Token::Str("tag"),
                Token::Str("Bytes"),
                Token::Str("bytes"),
                Token::Seq { len: Some(1) },
                Token::U8(0),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }
}

#[test]
fn borrow() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "tag")]
    enum Input<'a> {
        Package { name: &'a str },
    }

    assert_tokens(
        &Input::Package { name: "borrowed" },
        &[
            Token::Struct {
                name: "Input",
                len: 2,
            },
            Token::BorrowedStr("tag"),
            Token::BorrowedStr("Package"),
            Token::BorrowedStr("name"),
            Token::BorrowedStr("borrowed"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn with_skipped_conflict() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "tag")]
    enum Data {
        A,
        #[serde(skip)]
        #[allow(dead_code)]
        B {
            t: String,
        },
        C {
            #[serde(default, skip)]
            t: String,
        },
    }

    let data = Data::C { t: String::new() };

    assert_tokens(
        &data,
        &[
            Token::Struct {
                name: "Data",
                len: 1,
            },
            Token::Str("tag"),
            Token::Str("C"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn containing_flatten() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "tag")]
    enum Data {
        A {
            a: i32,
            #[serde(flatten)]
            flat: Flat,
        },
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Flat {
        b: i32,
    }

    let data = Data::A {
        a: 0,
        flat: Flat { b: 0 },
    };

    assert_tokens(
        &data,
        &[
            Token::Map { len: None },
            Token::Str("tag"),
            Token::Str("A"),
            Token::Str("a"),
            Token::I32(0),
            Token::Str("b"),
            Token::I32(0),
            Token::MapEnd,
        ],
    );
}

#[test]
fn unit_variant_with_unknown_fields() {
    let value = InternallyTagged::Unit;

    assert_de_tokens(
        &value,
        &[
            Token::Map { len: None },
            Token::Str("tag"),
            Token::Str("Unit"),
            Token::Str("b"),
            Token::I32(0),
            Token::MapEnd,
        ],
    );

    // Unknown elements are not allowed in sequences
    assert_de_tokens_error::<InternallyTagged>(
        &[
            Token::Seq { len: None },
            Token::Str("Unit"), // tag
            Token::I32(0),
            Token::SeqEnd,
        ],
        "invalid length 1, expected 0 elements in sequence",
    );
}

#[test]
fn expecting_message() {
    #[derive(Deserialize)]
    #[serde(tag = "tag")]
    #[serde(expecting = "something strange...")]
    enum Enum {
        InternallyTagged,
    }

    assert_de_tokens_error::<Enum>(
        &[Token::Str("InternallyTagged")],
        r#"invalid type: string "InternallyTagged", expected something strange..."#,
    );

    // Check that #[serde(expecting = "...")] doesn't affect variant identifier error message
    assert_de_tokens_error::<Enum>(
        &[Token::Map { len: None }, Token::Str("tag"), Token::Unit],
        "invalid type: unit value, expected variant identifier",
    );
}
