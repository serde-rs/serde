//! Tests for `#[serde(skip)]` behavior

use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_de_tokens, assert_tokens, Token};

/// This helper struct does not implement neither `Deserialize` or `Serialize`.
/// This should not prevent to use it in the skipped fields.
#[derive(Debug, PartialEq, Default)]
struct NotSerializableOrDeserializable;

#[test]
fn unit() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Unit;

    let value = Unit;

    // Uses visit_unit
    assert_tokens(&value, &[Token::UnitStruct { name: "Unit" }]);

    // Uses visit_unit
    assert_de_tokens(&value, &[Token::Unit]);
}

mod tuple_struct {
    use super::*;

    #[test]
    fn tuple0() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Tuple0();

        let value = Tuple0();

        // Uses visit_seq
        assert_tokens(
            &value,
            &[
                Token::TupleStruct {
                    name: "Tuple0",
                    len: 0,
                },
                Token::TupleStructEnd,
            ],
        );
    }

    #[test]
    fn tuple1() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Tuple1(u32);

        let value = Tuple1(42);

        // Uses visit_newtype_struct
        assert_tokens(
            &value,
            &[Token::NewtypeStruct { name: "Tuple1" }, Token::U32(42)],
        );

        // Uses visit_seq
        assert_de_tokens(
            &value,
            &[Token::Seq { len: None }, Token::U32(42), Token::SeqEnd],
        );
    }

    #[test]
    fn tuple2() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Tuple2(u32, u32);

        let value = Tuple2(4, 2);

        // Uses visit_seq
        assert_tokens(
            &value,
            &[
                Token::TupleStruct {
                    name: "Tuple2",
                    len: 2,
                },
                Token::U32(4),
                Token::U32(2),
                Token::TupleStructEnd,
            ],
        );
    }

    /* FIXME: compilation error: https://github.com/serde-rs/serde/issues/2105
    #[test]
    fn tuple1as0() {
        /// This newtype struct in the serialized form the same as `struct Tuple0();`
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Tuple1as0(#[serde(skip)] NotSerializableOrDeserializable);

        let value = Tuple1as0(NotSerializableOrDeserializable);

        // Uses visit_seq
        assert_tokens(
            &value,
            &[
                Token::TupleStruct {
                    name: "Tuple1as0",
                    len: 0,
                },
                Token::TupleStructEnd,
            ],
        );
    }*/

    #[test]
    fn tuple2as0() {
        /// This tuple struct in the serialized form the same as `struct Tuple0();`
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Tuple2as0(
            #[serde(skip)] NotSerializableOrDeserializable,
            #[serde(skip)] NotSerializableOrDeserializable,
        );

        let value = Tuple2as0(
            NotSerializableOrDeserializable,
            NotSerializableOrDeserializable,
        );

        // Uses visit_seq
        assert_tokens(
            &value,
            &[
                Token::TupleStruct {
                    name: "Tuple2as0",
                    len: 0,
                },
                Token::TupleStructEnd,
            ],
        );
    }

    #[test]
    fn tuple2as1() {
        /// This tuple struct in the serialized form the same as `struct Tuple1(u32);`
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Tuple2as1(#[serde(skip)] NotSerializableOrDeserializable, u32);

        let value = Tuple2as1(NotSerializableOrDeserializable, 20);

        // Uses visit_seq
        assert_tokens(
            &value,
            &[
                Token::TupleStruct {
                    name: "Tuple2as1",
                    len: 1,
                },
                Token::U32(20),
                Token::TupleStructEnd,
            ],
        );

        // Uses visit_newtype_struct
        assert_de_tokens(
            &value,
            &[Token::NewtypeStruct { name: "Tuple2as1" }, Token::U32(20)],
        );
    }

    #[test]
    fn tuple3as2() {
        /// This tuple struct in the serialized form the same as `struct Tuple2(u32, u32);`
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Tuple3as2(#[serde(skip)] NotSerializableOrDeserializable, u32, u32);

        let value = Tuple3as2(NotSerializableOrDeserializable, 20, 30);

        // Uses visit_seq
        assert_tokens(
            &value,
            &[
                Token::TupleStruct {
                    name: "Tuple3as2",
                    len: 2,
                },
                Token::U32(20),
                Token::U32(30),
                Token::TupleStructEnd,
            ],
        );
    }
}

mod struct_ {
    use super::*;

    #[test]
    fn struct0() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Struct0 {}

        let value = Struct0 {};

        // Uses visit_map
        assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Struct0",
                    len: 0,
                },
                Token::StructEnd,
            ],
        );

        // Uses visit_seq
        assert_de_tokens(&value, &[Token::Seq { len: None }, Token::SeqEnd]);
    }

    #[test]
    fn struct1() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Struct1 {
            a: u32,
        }

        let value = Struct1 { a: 42 };

        // Uses visit_map
        assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Struct1",
                    len: 1,
                },
                Token::Str("a"),
                Token::U32(42),
                Token::StructEnd,
            ],
        );

        // Uses visit_seq
        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: None },
                Token::U32(42), // a
                Token::SeqEnd,
            ],
        );
    }

    #[test]
    fn struct2as0() {
        /// This struct in the serialized form the same as `struct Struct0 {}`
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Struct2as0 {
            #[serde(skip)]
            a: NotSerializableOrDeserializable,
            #[serde(skip)]
            b: NotSerializableOrDeserializable,
        }

        let value = Struct2as0 {
            a: NotSerializableOrDeserializable,
            b: NotSerializableOrDeserializable,
        };

        // Uses visit_map
        assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Struct2as0",
                    len: 0,
                },
                Token::StructEnd,
            ],
        );

        // Uses visit_seq
        assert_de_tokens(&value, &[Token::Seq { len: None }, Token::SeqEnd]);
    }

    #[test]
    fn struct2as1() {
        /// This struct in the serialized form the same as `struct Struct1 { b: u32 }`
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Struct2as1 {
            #[serde(skip)]
            a: NotSerializableOrDeserializable,
            b: u32,
        }

        let value = Struct2as1 {
            a: NotSerializableOrDeserializable,
            b: 20,
        };

        // Uses visit_map
        assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Struct2as1",
                    len: 1,
                },
                Token::Str("b"),
                Token::U32(20),
                Token::StructEnd,
            ],
        );

        // Uses visit_seq
        assert_de_tokens(
            &value,
            &[
                Token::Seq { len: None },
                Token::U32(20), // b
                Token::SeqEnd,
            ],
        );
    }
}
