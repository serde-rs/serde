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
    }

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

mod enum_ {
    use super::*;

    mod externally_tagged {
        use super::*;

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        enum Enum {
            Unit,

            Tuple0(),
            Tuple1(u32),
            Tuple2(u32, u32),
            Tuple1as0(#[serde(skip)] NotSerializableOrDeserializable),
            Tuple2as0(
                #[serde(skip)] NotSerializableOrDeserializable,
                #[serde(skip)] NotSerializableOrDeserializable,
            ),
            Tuple2as1(#[serde(skip)] NotSerializableOrDeserializable, u32),
            Tuple3as2(#[serde(skip)] NotSerializableOrDeserializable, u32, u32),

            Struct0 {},
            Struct1 {
                a: u32,
            },
            Struct2as0 {
                #[serde(skip)]
                a: NotSerializableOrDeserializable,
                #[serde(skip)]
                b: NotSerializableOrDeserializable,
            },
            Struct2as1 {
                #[serde(skip)]
                a: NotSerializableOrDeserializable,
                b: u32,
            },
        }

        #[test]
        fn unit() {
            let value = Enum::Unit;

            // Uses unit_variant
            // Visitor does not used
            assert_tokens(
                &value,
                &[Token::UnitVariant {
                    name: "Enum",
                    variant: "Unit",
                }],
            );
        }

        mod tuple_struct {
            use super::*;

            #[test]
            fn tuple0() {
                let value = Enum::Tuple0();

                // Uses unit_variant
                // Visitor does not used
                assert_tokens(
                    &value,
                    &[Token::UnitVariant {
                        name: "Enum",
                        variant: "Tuple0",
                    }],
                );

                // Uses tuple_variant(0) + visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::TupleVariant {
                            name: "Enum",
                            variant: "Tuple0",
                            len: 0,
                        },
                        Token::TupleVariantEnd,
                    ],
                );
            }

            #[test]
            fn tuple1() {
                let value = Enum::Tuple1(42);

                // Uses newtype_variant
                // Visitor does not used
                assert_tokens(
                    &value,
                    &[
                        Token::NewtypeVariant {
                            name: "Enum",
                            variant: "Tuple1",
                        },
                        Token::U32(42),
                    ],
                );

                // Uses tuple_variant(1) + visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::TupleVariant {
                            name: "Enum",
                            variant: "Tuple1",
                            len: 1,
                        },
                        Token::U32(42),
                        Token::TupleVariantEnd,
                    ],
                );
            }

            #[test]
            fn tuple2() {
                let value = Enum::Tuple2(4, 2);

                // Uses tuple_variant(2) + visit_seq
                assert_tokens(
                    &value,
                    &[
                        Token::TupleVariant {
                            name: "Enum",
                            variant: "Tuple2",
                            len: 2,
                        },
                        Token::U32(4),
                        Token::U32(2),
                        Token::TupleVariantEnd,
                    ],
                );
            }

            #[test]
            fn tuple1as0() {
                let value = Enum::Tuple1as0(NotSerializableOrDeserializable);

                // Uses unit_variant
                // Visitor does not used
                assert_tokens(
                    &value,
                    &[Token::UnitVariant {
                        name: "Enum",
                        variant: "Tuple1as0",
                    }],
                );

                // Uses tuple_variant(0) + visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::TupleVariant {
                            name: "Enum",
                            variant: "Tuple1as0",
                            len: 0,
                        },
                        Token::TupleVariantEnd,
                    ],
                );
            }

            #[test]
            fn tuple2as0() {
                let value = Enum::Tuple2as0(
                    NotSerializableOrDeserializable,
                    NotSerializableOrDeserializable,
                );

                // Uses unit_variant
                // Visitor does not used
                assert_tokens(
                    &value,
                    &[Token::UnitVariant {
                        name: "Enum",
                        variant: "Tuple2as0",
                    }],
                );

                // Uses tuple_variant(0) + visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::TupleVariant {
                            name: "Enum",
                            variant: "Tuple2as0",
                            len: 0,
                        },
                        Token::TupleVariantEnd,
                    ],
                );
            }

            #[test]
            fn tuple2as1() {
                let value = Enum::Tuple2as1(NotSerializableOrDeserializable, 20);

                // Uses newtype_variant
                // Visitor does not used
                assert_tokens(
                    &value,
                    &[
                        Token::NewtypeVariant {
                            name: "Enum",
                            variant: "Tuple2as1",
                        },
                        Token::U32(20),
                    ],
                );

                // Uses tuple_variant(1) + visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::TupleVariant {
                            name: "Enum",
                            variant: "Tuple2as1",
                            len: 1,
                        },
                        Token::U32(20),
                        Token::TupleVariantEnd,
                    ],
                );
            }

            #[test]
            fn tuple3as2() {
                let value = Enum::Tuple3as2(NotSerializableOrDeserializable, 20, 30);

                // Uses tuple_variant(2) + visit_seq
                assert_tokens(
                    &value,
                    &[
                        Token::TupleVariant {
                            name: "Enum",
                            variant: "Tuple3as2",
                            len: 2,
                        },
                        Token::U32(20),
                        Token::U32(30),
                        Token::TupleVariantEnd,
                    ],
                );
            }
        }

        mod struct_ {
            use super::*;

            #[test]
            fn struct0() {
                let value = Enum::Struct0 {};

                // Uses struct_variant + visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::StructVariant {
                            name: "Enum",
                            variant: "Struct0",
                            len: 0,
                        },
                        Token::StructVariantEnd,
                    ],
                );

                // Uses struct_variant + visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Enum { name: "Enum" },
                        Token::Str("Struct0"),
                        Token::Seq { len: None },
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn struct1() {
                let value = Enum::Struct1 { a: 42 };

                // Uses struct_variant + visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::StructVariant {
                            name: "Enum",
                            variant: "Struct1",
                            len: 1,
                        },
                        Token::Str("a"),
                        Token::U32(42),
                        Token::StructVariantEnd,
                    ],
                );

                // Uses struct_variant + visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Enum { name: "Enum" },
                        Token::Str("Struct1"),
                        Token::Seq { len: None },
                        Token::U32(42), // a
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn struct2as0() {
                let value = Enum::Struct2as0 {
                    a: NotSerializableOrDeserializable,
                    b: NotSerializableOrDeserializable,
                };

                // Uses struct_variant + visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::StructVariant {
                            name: "Enum",
                            variant: "Struct2as0",
                            len: 0,
                        },
                        Token::StructVariantEnd,
                    ],
                );

                // Uses struct_variant + visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Enum { name: "Enum" },
                        Token::Str("Struct2as0"),
                        Token::Seq { len: None },
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn struct2as1() {
                let value = Enum::Struct2as1 {
                    a: NotSerializableOrDeserializable,
                    b: 20,
                };

                // Uses struct_variant + visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::StructVariant {
                            name: "Enum",
                            variant: "Struct2as1",
                            len: 1,
                        },
                        Token::Str("b"),
                        Token::U32(20),
                        Token::StructVariantEnd,
                    ],
                );

                // Uses struct_variant + visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Enum { name: "Enum" },
                        Token::Str("Struct2as1"),
                        Token::Seq { len: None },
                        Token::U32(20), // b
                        Token::SeqEnd,
                    ],
                );
            }
        }
    }

    mod internally_tagged {
        use super::*;

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Nested {}

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        #[serde(tag = "tag")]
        enum Enum {
            Unit,

            // Tuple0(), // FIXME; compilation error
            Tuple1(Nested),
            // Tuple2(u32, u32), - not supported in internally tagged enums
            Tuple1as0(#[serde(skip)] NotSerializableOrDeserializable),
            // Tuple2as0( // FIXME; compilation error
            //     #[serde(skip)] NotSerializableOrDeserializable,
            //     #[serde(skip)] NotSerializableOrDeserializable,
            // ),
            // Tuple2as1(#[serde(skip)] NotSerializableOrDeserializable, Nested), // FIXME; compilation error
            // Tuple3as2(#[serde(skip)] NotSerializableOrDeserializable, u32, u32), - not supported in internally tagged enums
            Struct0 {},
            Struct1 {
                a: u32,
            },
            Struct2as0 {
                #[serde(skip)]
                a: NotSerializableOrDeserializable,
                #[serde(skip)]
                b: NotSerializableOrDeserializable,
            },
            Struct2as1 {
                #[serde(skip)]
                a: NotSerializableOrDeserializable,
                b: u32,
            },
        }

        #[test]
        fn unit() {
            let value = Enum::Unit;

            // Uses visit_map
            assert_tokens(
                &value,
                &[
                    Token::Struct {
                        name: "Enum",
                        len: 1,
                    },
                    Token::Str("tag"),
                    Token::Str("Unit"),
                    Token::StructEnd,
                ],
            );

            // Uses visit_seq
            assert_de_tokens(
                &value,
                &[
                    Token::Seq { len: None },
                    Token::Str("Unit"), // tag
                    Token::SeqEnd,
                ],
            );
        }

        mod tuple_struct {
            use super::*;

            /* FIXME: compilation error
            #[test]
            fn tuple0() {
                let value = Enum::Tuple0();

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 1,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple0"),
                        Token::StructEnd,
                    ],
                );

                // Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple0"), // tag
                        Token::SeqEnd,
                    ],
                );
            }*/

            #[test]
            fn tuple1() {
                let value = Enum::Tuple1(Nested {});

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 1,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple1"),
                        // Nested fields...
                        Token::StructEnd,
                    ],
                );

                // Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple1"), // tag
                        // Nested fields...
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn tuple1as0() {
                let value = Enum::Tuple1as0(NotSerializableOrDeserializable);

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 1,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple1as0"),
                        Token::StructEnd,
                    ],
                );

                // Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple1as0"), // tag
                        Token::SeqEnd,
                    ],
                );
            }

            /* FIXME: compilation error
            #[test]
            fn tuple2as0() {
                let value = Enum::Tuple2as0(
                    NotSerializableOrDeserializable,
                    NotSerializableOrDeserializable,
                );

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 1,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple2as0"),
                        Token::StructEnd,
                    ],
                );

                // Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple2as0"), // tag
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn tuple2as1() {
                let value = Enum::Tuple2as1(NotSerializableOrDeserializable, Nested {});

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 1,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple2as1"),
                        // Nested fields...
                        Token::StructEnd,
                    ],
                );

                // Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple2as1"), // tag
                        // Nested fields...
                        Token::SeqEnd,
                    ],
                );
            }*/
        }

        mod struct_ {
            use super::*;

            #[test]
            fn struct0() {
                let value = Enum::Struct0 {};

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 1,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct0"),
                        Token::StructEnd,
                    ],
                );

                // Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Struct0"), // tag
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn struct1() {
                let value = Enum::Struct1 { a: 42 };

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct1"),
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
                        Token::Str("Struct1"), // tag
                        Token::U32(42),        // a
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn struct2as0() {
                let value = Enum::Struct2as0 {
                    a: NotSerializableOrDeserializable,
                    b: NotSerializableOrDeserializable,
                };

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 1,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct2as0"),
                        Token::StructEnd,
                    ],
                );

                // Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Struct2as0"), // tag
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn struct2as1() {
                let value = Enum::Struct2as1 {
                    a: NotSerializableOrDeserializable,
                    b: 20,
                };

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct2as1"),
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
                        Token::Str("Struct2as1"), // tag
                        Token::U32(20),           // b
                        Token::SeqEnd,
                    ],
                );
            }
        }
    }

    mod adjacently_tagged {
        use super::*;

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        #[serde(tag = "tag", content = "content")]
        enum Enum {
            Unit,

            Tuple0(),
            Tuple1(u32),
            Tuple2(u32, u32),
            // Tuple1as0(#[serde(skip)] NotSerializableOrDeserializable), // FIXME; compilation error
            Tuple2as0(
                #[serde(skip)] NotSerializableOrDeserializable,
                #[serde(skip)] NotSerializableOrDeserializable,
            ),
            Tuple2as1(#[serde(skip)] NotSerializableOrDeserializable, u32),
            Tuple3as2(#[serde(skip)] NotSerializableOrDeserializable, u32, u32),

            Struct0 {},
            Struct1 {
                a: u32,
            },
            Struct2as0 {
                #[serde(skip)]
                a: NotSerializableOrDeserializable,
                #[serde(skip)]
                b: NotSerializableOrDeserializable,
            },
            Struct2as1 {
                #[serde(skip)]
                a: NotSerializableOrDeserializable,
                b: u32,
            },
        }

        #[test]
        fn unit() {
            let value = Enum::Unit;

            // Map: No `content` field
            assert_tokens(
                &value,
                &[
                    Token::Struct {
                        name: "Enum",
                        len: 1,
                    },
                    Token::Str("tag"),
                    Token::Str("Unit"),
                    Token::StructEnd,
                ],
            );
            // Map: Uses visit_unit
            assert_de_tokens(
                &value,
                &[
                    Token::Struct {
                        name: "Enum",
                        len: 2,
                    },
                    Token::Str("tag"),
                    Token::Str("Unit"),
                    Token::Str("content"),
                    Token::Unit,
                    Token::StructEnd,
                ],
            );

            // Seq: No `content` field
            assert_de_tokens(
                &value,
                &[
                    Token::Seq { len: None },
                    Token::Str("Unit"), // tag
                    Token::SeqEnd,
                ],
            );
            // Seq: Uses visit_unit
            assert_de_tokens(
                &value,
                &[
                    Token::Seq { len: None },
                    Token::Str("Unit"), // tag
                    Token::Unit,        // content
                    Token::SeqEnd,
                ],
            );
        }

        mod tuple_struct {
            use super::*;

            #[test]
            fn tuple0() {
                let value = Enum::Tuple0();

                // Map: No `content` field
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 1,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple0"),
                        Token::StructEnd,
                    ],
                );
                // Map: Uses visit_unit
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple0"),
                        Token::Str("content"),
                        Token::Unit,
                        Token::StructEnd,
                    ],
                );
                // Map: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple0"),
                        Token::Str("content"),
                        Token::Tuple { len: 0 },
                        Token::TupleEnd,
                        Token::StructEnd,
                    ],
                );

                // Seq: No `content` field
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple0"), // tag
                        Token::SeqEnd,
                    ],
                );
                // Seq: Uses visit_unit
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple0"), // tag
                        Token::Unit,          // content
                        Token::SeqEnd,
                    ],
                );
                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple0"),    // tag
                        Token::Tuple { len: 0 }, // content
                        Token::TupleEnd,
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn tuple1() {
                let value = Enum::Tuple1(42);

                // Map: delegates to the inner type
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple1"),
                        Token::Str("content"),
                        Token::U32(42),
                        Token::StructEnd,
                    ],
                );
                // Map: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple1"),
                        Token::Str("content"),
                        Token::Tuple { len: 1 },
                        Token::U32(42),
                        Token::TupleEnd,
                        Token::StructEnd,
                    ],
                );

                // Seq: delegates to the inner type
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple1"), // tag
                        Token::U32(42),       // content
                        Token::SeqEnd,
                    ],
                );
                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple1"),    // tag
                        Token::Tuple { len: 1 }, // content
                        Token::U32(42),
                        Token::TupleEnd,
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn tuple2() {
                let value = Enum::Tuple2(4, 2);

                // Map: Uses visit_seq
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::UnitVariant {
                            name: "Enum",
                            variant: "Tuple2",
                        },
                        Token::Str("content"),
                        Token::Tuple { len: 2 },
                        Token::U32(4),
                        Token::U32(2),
                        Token::TupleEnd,
                        Token::StructEnd,
                    ],
                );
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple2"),
                        Token::Str("content"),
                        Token::Tuple { len: 2 },
                        Token::U32(4),
                        Token::U32(2),
                        Token::TupleEnd,
                        Token::StructEnd,
                    ],
                );

                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple2"),    // tag
                        Token::Tuple { len: 2 }, // content
                        Token::U32(4),
                        Token::U32(2),
                        Token::TupleEnd,
                        Token::SeqEnd,
                    ],
                );
            }

            /* FIXME: compilation error
            #[test]
            fn tuple1as0() {
                let value = Enum::Tuple1as0(NotSerializableOrDeserializable);

                // Map: No `content` field
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 1,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple1as0"),
                        Token::StructEnd,
                    ],
                );
                // Map: Uses visit_unit
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple1as0"),
                        Token::Str("content"),
                        Token::Unit,
                        Token::StructEnd,
                    ],
                );
                // Map: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple1as0"),
                        Token::Str("content"),
                        Token::Tuple { len: 0 },
                        Token::TupleEnd,
                        Token::StructEnd,
                    ],
                );

                // Seq: No `content` field
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple1as0"), // tag
                        Token::SeqEnd,
                    ],
                );
                // Seq: Uses visit_unit
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple1as0"), // tag
                        Token::Unit,             // content
                        Token::SeqEnd,
                    ],
                );
                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple1as0"), // tag
                        Token::Tuple { len: 0 }, // content
                        Token::TupleEnd,
                        Token::SeqEnd,
                    ],
                );
            }*/

            #[test]
            fn tuple2as0() {
                let value = Enum::Tuple2as0(
                    NotSerializableOrDeserializable,
                    NotSerializableOrDeserializable,
                );

                // Map: No `content` field
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 1,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple2as0"),
                        Token::StructEnd,
                    ],
                );
                // Map: Uses visit_unit
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple2as0"),
                        Token::Str("content"),
                        Token::Unit,
                        Token::StructEnd,
                    ],
                );
                // Map: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple2as0"),
                        Token::Str("content"),
                        Token::Tuple { len: 0 },
                        Token::TupleEnd,
                        Token::StructEnd,
                    ],
                );

                // Seq: No `content` field
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple2as0"), // tag
                        Token::SeqEnd,
                    ],
                );
                // Seq: Uses visit_unit
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple2as0"), // tag
                        Token::Unit,             // content
                        Token::SeqEnd,
                    ],
                );
                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple2as0"), // tag
                        Token::Tuple { len: 0 }, // content
                        Token::TupleEnd,
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn tuple2as1() {
                let value = Enum::Tuple2as1(NotSerializableOrDeserializable, 20);

                // Map: delegates to the inner type
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple2as1"),
                        Token::Str("content"),
                        Token::U32(20),
                        Token::StructEnd,
                    ],
                );
                // Map: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple2as1"),
                        Token::Str("content"),
                        Token::Tuple { len: 1 },
                        Token::U32(20),
                        Token::TupleEnd,
                        Token::StructEnd,
                    ],
                );

                // Seq: delegates to the inner type
                assert_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple2as1"), // tag
                        Token::U32(20),          // content
                        Token::SeqEnd,
                    ],
                );
                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple2as1"), // tag
                        Token::Tuple { len: 1 }, // content
                        Token::U32(20),
                        Token::TupleEnd,
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn tuple3as2() {
                let value = Enum::Tuple3as2(NotSerializableOrDeserializable, 20, 30);

                // Map: Uses visit_seq
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::UnitVariant {
                            name: "Enum",
                            variant: "Tuple3as2",
                        },
                        Token::Str("content"),
                        Token::Tuple { len: 2 },
                        Token::U32(20),
                        Token::U32(30),
                        Token::TupleEnd,
                        Token::StructEnd,
                    ],
                );
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Tuple3as2"),
                        Token::Str("content"),
                        Token::Tuple { len: 2 },
                        Token::U32(20),
                        Token::U32(30),
                        Token::TupleEnd,
                        Token::StructEnd,
                    ],
                );

                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Tuple3as2"), // tag
                        Token::Tuple { len: 2 }, // content
                        Token::U32(20),
                        Token::U32(30),
                        Token::TupleEnd,
                        Token::SeqEnd,
                    ],
                );
            }
        }

        mod struct_ {
            use super::*;

            #[test]
            fn struct0() {
                let value = Enum::Struct0 {};

                // Map: Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::UnitVariant {
                            name: "Enum",
                            variant: "Struct0",
                        },
                        Token::Str("content"),
                        Token::Struct {
                            name: "Struct0",
                            len: 0,
                        },
                        Token::StructEnd,
                        Token::StructEnd,
                    ],
                );
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct0"),
                        Token::Str("content"),
                        Token::Struct {
                            name: "Struct0",
                            len: 0,
                        },
                        Token::StructEnd,
                        Token::StructEnd,
                    ],
                );
                // Seq: Uses visit_map
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Struct0"), // tag
                        // content
                        Token::Struct {
                            name: "Struct0",
                            len: 0,
                        },
                        Token::StructEnd,
                        Token::SeqEnd,
                    ],
                );

                // Map: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct0"),
                        Token::Str("content"),
                        Token::Seq { len: None },
                        Token::SeqEnd,
                        Token::StructEnd,
                    ],
                );
                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Struct0"),    // tag
                        Token::Seq { len: None }, // content
                        Token::SeqEnd,
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn struct1() {
                let value = Enum::Struct1 { a: 42 };

                // Map: Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::UnitVariant {
                            name: "Enum",
                            variant: "Struct1",
                        },
                        Token::Str("content"),
                        Token::Struct {
                            name: "Struct1",
                            len: 1,
                        },
                        Token::Str("a"),
                        Token::U32(42),
                        Token::StructEnd,
                        Token::StructEnd,
                    ],
                );
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct1"),
                        Token::Str("content"),
                        Token::Struct {
                            name: "Struct1",
                            len: 1,
                        },
                        Token::Str("a"),
                        Token::U32(42),
                        Token::StructEnd,
                        Token::StructEnd,
                    ],
                );
                // Seq: Uses visit_map
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Struct1"), // tag
                        // content
                        Token::Struct {
                            name: "Struct1",
                            len: 1,
                        },
                        Token::Str("a"),
                        Token::U32(42),
                        Token::StructEnd,
                        Token::SeqEnd,
                    ],
                );

                // Map: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct1"),
                        Token::Str("content"),
                        Token::Seq { len: None },
                        Token::U32(42), // a
                        Token::SeqEnd,
                        Token::StructEnd,
                    ],
                );
                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Struct1"),    // tag
                        Token::Seq { len: None }, // content
                        Token::U32(42),           // a
                        Token::SeqEnd,
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn struct2as0() {
                let value = Enum::Struct2as0 {
                    a: NotSerializableOrDeserializable,
                    b: NotSerializableOrDeserializable,
                };

                // Map: Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::UnitVariant {
                            name: "Enum",
                            variant: "Struct2as0",
                        },
                        Token::Str("content"),
                        Token::Struct {
                            name: "Struct2as0",
                            len: 0,
                        },
                        Token::StructEnd,
                        Token::StructEnd,
                    ],
                );
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct2as0"),
                        Token::Str("content"),
                        Token::Struct {
                            name: "Struct2as0",
                            len: 0,
                        },
                        Token::StructEnd,
                        Token::StructEnd,
                    ],
                );
                // Seq: Uses visit_map
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Struct2as0"), // tag
                        // content
                        Token::Struct {
                            name: "Struct2as0",
                            len: 0,
                        },
                        Token::StructEnd,
                        Token::SeqEnd,
                    ],
                );

                // Map: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct2as0"),
                        Token::Str("content"),
                        Token::Seq { len: None },
                        Token::SeqEnd,
                        Token::StructEnd,
                    ],
                );
                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Struct2as0"), // tag
                        Token::Seq { len: None }, // content
                        Token::SeqEnd,
                        Token::SeqEnd,
                    ],
                );
            }

            #[test]
            fn struct2as1() {
                let value = Enum::Struct2as1 {
                    a: NotSerializableOrDeserializable,
                    b: 20,
                };

                // Map: Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::UnitVariant {
                            name: "Enum",
                            variant: "Struct2as1",
                        },
                        Token::Str("content"),
                        Token::Struct {
                            name: "Struct2as1",
                            len: 1,
                        },
                        Token::Str("b"),
                        Token::U32(20),
                        Token::StructEnd,
                        Token::StructEnd,
                    ],
                );
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct2as1"),
                        Token::Str("content"),
                        Token::Struct {
                            name: "Struct2as1",
                            len: 1,
                        },
                        Token::Str("b"),
                        Token::U32(20),
                        Token::StructEnd,
                        Token::StructEnd,
                    ],
                );
                // Seq: Uses visit_map
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Struct2as1"), // tag
                        // content
                        Token::Struct {
                            name: "Struct2as1",
                            len: 1,
                        },
                        Token::Str("b"),
                        Token::U32(20),
                        Token::StructEnd,
                        Token::SeqEnd,
                    ],
                );

                // Map: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
                            len: 2,
                        },
                        Token::Str("tag"),
                        Token::Str("Struct2as1"),
                        Token::Str("content"),
                        Token::Seq { len: None },
                        Token::U32(20), // b
                        Token::SeqEnd,
                        Token::StructEnd,
                    ],
                );
                // Seq: Uses visit_seq
                assert_de_tokens(
                    &value,
                    &[
                        Token::Seq { len: None },
                        Token::Str("Struct2as1"), // tag
                        Token::Seq { len: None }, // content
                        Token::U32(20),           // b
                        Token::SeqEnd,
                        Token::SeqEnd,
                    ],
                );
            }
        }
    }

    mod untagged {
        use super::*;

        #[test]
        fn unit() {
            #[derive(Debug, PartialEq, Deserialize, Serialize)]
            #[serde(untagged)]
            enum Enum {
                Unit,
            }

            let value = Enum::Unit;

            // Uses visit_unit
            assert_tokens(&value, &[Token::Unit]);
            // Uses visit_unit_struct
            assert_de_tokens(&value, &[Token::UnitStruct { name: "Unit" }]);
        }

        mod tuple_struct {
            use super::*;

            #[derive(Debug, PartialEq, Deserialize, Serialize)]
            #[serde(untagged)]
            enum Enum {
                Tuple2(u32, u32),
                Tuple1(u32),
                Tuple0(),
            }

            /// Because enum variant is determined by the serialized form, that
            /// should be identical for variants with and without skipped fields,
            /// we should have two different enums to test deserialization
            #[derive(Debug, PartialEq, Deserialize, Serialize)]
            #[serde(untagged)]
            enum WithSkipped {
                Tuple3as2(#[serde(skip)] NotSerializableOrDeserializable, u32, u32),
                Tuple2as1(#[serde(skip)] NotSerializableOrDeserializable, u32),
                Tuple1as0(#[serde(skip)] NotSerializableOrDeserializable),
            }

            #[test]
            fn tuple0() {
                let value = Enum::Tuple0();

                // Uses visit_unit
                assert_tokens(&value, &[Token::Unit]);
                // Uses visit_unit_struct
                assert_de_tokens(&value, &[Token::UnitStruct { name: "Unit" }]);
                // Uses visit_seq
                assert_de_tokens(&value, &[Token::Tuple { len: 0 }, Token::TupleEnd]);
            }

            #[test]
            fn tuple1() {
                let value = Enum::Tuple1(42);

                // Delegates to the inner field
                assert_tokens(&value, &[Token::U32(42)]);
            }

            #[test]
            fn tuple2() {
                let value = Enum::Tuple2(4, 2);

                // Uses visit_seq
                assert_tokens(
                    &value,
                    &[
                        Token::Tuple { len: 2 },
                        Token::U32(4),
                        Token::U32(2),
                        Token::TupleEnd,
                    ],
                );
            }

            #[test]
            fn tuple1as0() {
                let value = WithSkipped::Tuple1as0(NotSerializableOrDeserializable);

                // Uses visit_unit
                assert_tokens(&value, &[Token::Unit]);
                // Uses visit_unit_struct
                assert_de_tokens(&value, &[Token::UnitStruct { name: "Unit" }]);
                // Uses visit_seq
                assert_de_tokens(&value, &[Token::Tuple { len: 0 }, Token::TupleEnd]);
            }

            #[test]
            fn tuple2as0() {
                #[derive(Debug, PartialEq, Deserialize, Serialize)]
                #[serde(untagged)]
                enum Enum {
                    Tuple2as0(
                        #[serde(skip)] NotSerializableOrDeserializable,
                        #[serde(skip)] NotSerializableOrDeserializable,
                    ),
                }

                let value = Enum::Tuple2as0(
                    NotSerializableOrDeserializable,
                    NotSerializableOrDeserializable,
                );

                // Uses visit_unit
                assert_tokens(&value, &[Token::Unit]);
                // Uses visit_unit_struct
                assert_de_tokens(&value, &[Token::UnitStruct { name: "Unit" }]);
                // Uses visit_seq
                assert_de_tokens(&value, &[Token::Tuple { len: 0 }, Token::TupleEnd]);
            }

            #[test]
            fn tuple2as1() {
                let value = WithSkipped::Tuple2as1(NotSerializableOrDeserializable, 20);

                // Delegates to the second field
                assert_tokens(&value, &[Token::U32(20)]);
            }

            #[test]
            fn tuple3as2() {
                let value = WithSkipped::Tuple3as2(NotSerializableOrDeserializable, 20, 30);

                // Uses visit_seq
                assert_tokens(
                    &value,
                    &[
                        Token::Tuple { len: 2 },
                        Token::U32(20),
                        Token::U32(30),
                        Token::TupleEnd,
                    ],
                );
            }
        }

        mod struct_ {
            use super::*;

            #[derive(Debug, PartialEq, Deserialize, Serialize)]
            #[serde(untagged)]
            enum Enum {
                Struct1 { a: u32 },
                Struct0 {},
            }

            /// Because enum variant is determined by the serialized form, that
            /// should be identical for variants with and without skipped fields,
            /// we should have two different enums to test deserialization
            #[derive(Debug, PartialEq, Deserialize, Serialize)]
            #[serde(untagged)]
            enum WithSkipped {
                Struct2as1 {
                    #[serde(skip)]
                    a: NotSerializableOrDeserializable,
                    b: u32,
                },
                Struct2as0 {
                    #[serde(skip)]
                    a: NotSerializableOrDeserializable,
                    #[serde(skip)]
                    b: NotSerializableOrDeserializable,
                },
            }

            #[test]
            fn struct0() {
                let value = Enum::Struct0 {};

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
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
                let value = Enum::Struct1 { a: 42 };

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "Enum",
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
                let value = WithSkipped::Struct2as0 {
                    a: NotSerializableOrDeserializable,
                    b: NotSerializableOrDeserializable,
                };

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "WithSkipped",
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
                let value = WithSkipped::Struct2as1 {
                    a: NotSerializableOrDeserializable,
                    b: 20,
                };

                // Uses visit_map
                assert_tokens(
                    &value,
                    &[
                        Token::Struct {
                            name: "WithSkipped",
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
                        Token::U32(20), // a
                        Token::SeqEnd,
                    ],
                );
            }
        }
    }
}
