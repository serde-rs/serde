use self::bytes::{ByteBuf, Bytes};
use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_ser_tokens_error, assert_tokens, Token};

mod bytes;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
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

    NewtypeStruct(Struct),
    NewtypeEnum(Enum),

    Struct { i128_: i128, u128_: u128 },
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Unit;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Newtype<T>(T);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Tuple(u8, u8);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Struct {
    i128_: i128,
    u128_: u128,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum Enum {
    Unit,
    Newtype(u128),
    Tuple(i128, u128),
    Struct { i128_: i128, u128_: u128 },
}

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
}

mod newtype {
    use super::*;

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
        assert_tokens(
            &InternallyTagged::NewtypeUnit(()),
            &[
                Token::Map { len: Some(1) },
                Token::Str("tag"),
                Token::Str("NewtypeUnit"),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn unit_struct() {
        assert_tokens(
            &InternallyTagged::NewtypeUnitStruct(Unit),
            &[
                Token::Map { len: Some(1) },
                Token::Str("tag"),
                Token::Str("NewtypeUnitStruct"),
                Token::MapEnd,
            ],
        );
    }

    failed!(
        tuple,
        NewtypeTuple((4, 2)),
        "cannot serialize tagged newtype variant InternallyTagged::NewtypeTuple containing a tuple"
    );
    failed!(tuple_struct, NewtypeTupleStruct(Tuple(4, 2)), "cannot serialize tagged newtype variant InternallyTagged::NewtypeTupleStruct containing a tuple struct");

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

    #[test]
    fn struct_() {
        assert_tokens(
            &InternallyTagged::NewtypeStruct(Struct { i128_: 4, u128_: 2 }),
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
    }

    #[test]
    fn enum_unit() {
        assert_tokens(
            &InternallyTagged::NewtypeEnum(Enum::Unit),
            &[
                Token::Map { len: Some(2) },
                Token::Str("tag"),
                Token::Str("NewtypeEnum"),
                Token::Str("Unit"),
                Token::Unit,
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn enum_newtype() {
        assert_tokens(
            &InternallyTagged::NewtypeEnum(Enum::Newtype(42)),
            &[
                Token::Map { len: Some(2) },
                Token::Str("tag"),
                Token::Str("NewtypeEnum"),
                Token::Str("Newtype"),
                Token::U128(42),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn enum_tuple() {
        assert_tokens(
            &InternallyTagged::NewtypeEnum(Enum::Tuple(4, 2)),
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
    }

    #[test]
    fn enum_struct() {
        assert_tokens(
            &InternallyTagged::NewtypeEnum(Enum::Struct { i128_: 4, u128_: 2 }),
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
    }
}

#[test]
fn struct_() {
    assert_tokens(
        &InternallyTagged::Struct { i128_: 4, u128_: 2 },
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
}
