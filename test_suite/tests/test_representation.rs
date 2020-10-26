use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_test::{assert_tokens, Configure, Token};

/// Readable representation of the test subject
#[derive(Serialize, Deserialize)]
struct Readable {
    mode: String,
}

/// Compact representation of the test subject
#[derive(Serialize, Deserialize)]
struct Compact {
    mode: u32,
}

/// Test subject, have the different representations in readable and compact formats
#[derive(Debug, PartialEq)]
struct Subject;
impl Serialize for Subject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        if serializer.is_human_readable() {
            Readable { mode: "readable".to_string() }.serialize(serializer)
        } else {
            Compact { mode: 42 }.serialize(serializer)
        }
    }
}
impl<'de> Deserialize<'de> for Subject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        if deserializer.is_human_readable() {
            <Readable as Deserialize<'de>>::deserialize(deserializer)?;
        } else {
            <Compact as Deserialize<'de>>::deserialize(deserializer)?;
        }
        Ok(Subject)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Newtype(Subject);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Tuple((), Subject);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Struct {
    subject: Subject,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Flatten {
    dummy: (),
    #[serde(flatten)]
    subject: Subject,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum ExternallyTagged {
    Newtype(Subject),
    Tuple((), Subject),
    Struct { subject: Subject },
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tag")]
enum InternallyTagged {
    Newtype(Subject),
    Struct { subject: Subject },
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tag", content = "content")]
enum AdjacentlyTagged {
    Newtype(Subject),
    Tuple((), Subject),
    Struct { subject: Subject },
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum Untagged {
    Newtype(Subject),
    Tuple((), Subject),
    Struct { subject: Subject },
}

mod readable {
    use super::*;

    #[test]
    fn bare() {
        assert_tokens(&Subject.readable(), &[
            Token::Struct { name: "Readable", len: 1 },
                Token::Str("mode"),
                Token::Str("readable"),
            Token::StructEnd,
        ]);
    }

    #[test]
    fn tuple() {
        assert_tokens(&((), Subject).readable(), &[
            Token::Tuple { len: 2 },
                Token::Unit,
                Token::Struct { name: "Readable", len: 1 },
                    Token::Str("mode"),
                    Token::Str("readable"),
                Token::StructEnd,
            Token::TupleEnd,
        ]);
    }

    #[test]
    fn newtype() {
        assert_tokens(&Newtype(Subject).readable(), &[
            Token::NewtypeStruct { name: "Newtype" },
            Token::Struct { name: "Readable", len: 1 },
                Token::Str("mode"),
                Token::Str("readable"),
            Token::StructEnd,
        ]);
    }

    #[test]
    fn named_tuple() {
        assert_tokens(&Tuple((), Subject).readable(), &[
            Token::TupleStruct { name: "Tuple", len: 2 },
                Token::Unit,
                Token::Struct { name: "Readable", len: 1 },
                    Token::Str("mode"),
                    Token::Str("readable"),
                Token::StructEnd,
            Token::TupleStructEnd,
        ]);
    }

    #[test]
    fn struct_() {
        assert_tokens(&Struct { subject: Subject }.readable(), &[
            Token::Struct { name: "Struct", len: 1 },
                Token::Str("subject"),
                Token::Struct { name: "Readable", len: 1 },
                    Token::Str("mode"),
                    Token::Str("readable"),
                Token::StructEnd,
            Token::StructEnd,
        ]);
    }

    mod externally_tagged_enum {
        use super::*;

        #[test]
        fn newtype() {
            assert_tokens(&ExternallyTagged::Newtype(Subject).readable(), &[
                Token::NewtypeVariant { name: "ExternallyTagged", variant: "Newtype" },
                Token::Struct { name: "Readable", len: 1 },
                    Token::Str("mode"),
                    Token::Str("readable"),
                Token::StructEnd,
            ]);
        }

        #[test]
        fn tuple() {
            assert_tokens(&ExternallyTagged::Tuple((), Subject).readable(), &[
                Token::TupleVariant { name: "ExternallyTagged", variant: "Tuple", len: 2 },
                    Token::Unit,
                    Token::Struct { name: "Readable", len: 1 },
                        Token::Str("mode"),
                        Token::Str("readable"),
                    Token::StructEnd,
                Token::TupleVariantEnd,
            ]);
        }

        #[test]
        fn struct_() {
            assert_tokens(&ExternallyTagged::Struct { subject: Subject }.readable(), &[
                Token::StructVariant { name: "ExternallyTagged", variant: "Struct", len: 1 },
                    Token::Str("subject"),
                    Token::Struct { name: "Readable", len: 1 },
                        Token::Str("mode"),
                        Token::Str("readable"),
                    Token::StructEnd,
                Token::StructVariantEnd,
            ]);
        }
    }

    mod internally_tagged_enum {
        use super::*;

        #[test]
        fn newtype() {
            assert_tokens(&InternallyTagged::Newtype(Subject).readable(), &[
                Token::Struct { name: "Readable", len: 2 },
                    Token::Str("tag"),
                    Token::Str("Newtype"),
                    Token::Str("mode"),
                    Token::Str("readable"),
                Token::StructEnd,
            ]);
        }

        #[test]
        fn struct_() {
            assert_tokens(&InternallyTagged::Struct { subject: Subject }.readable(), &[
                Token::Struct { name: "InternallyTagged", len: 2 },
                    Token::Str("tag"),
                    Token::Str("Struct"),
                    Token::Str("subject"),
                    Token::Struct { name: "Readable", len: 1 },
                        Token::Str("mode"),
                        Token::Str("readable"),
                    Token::StructEnd,
                Token::StructEnd,
            ]);
        }
    }

    mod adjacently_tagged_enum {
        use super::*;

        #[test]
        fn newtype() {
            assert_tokens(&AdjacentlyTagged::Newtype(Subject).readable(), &[
                Token::Struct { name: "AdjacentlyTagged", len: 2 },
                    Token::Str("tag"),
                    Token::Str("Newtype"),
                    Token::Str("content"),
                    Token::Struct { name: "Readable", len: 1 },
                        Token::Str("mode"),
                        Token::Str("readable"),
                    Token::StructEnd,
                Token::StructEnd,
            ]);
        }

        #[test]
        fn tuple() {
            assert_tokens(&AdjacentlyTagged::Tuple((), Subject).readable(), &[
                Token::Struct { name: "AdjacentlyTagged", len: 2 },
                    Token::Str("tag"),
                    Token::Str("Tuple"),
                    Token::Str("content"),
                    Token::Tuple { len: 2 },
                        Token::Unit,
                        Token::Struct { name: "Readable", len: 1 },
                            Token::Str("mode"),
                            Token::Str("readable"),
                        Token::StructEnd,
                    Token::TupleEnd,
                Token::StructEnd,
            ]);
        }

        #[test]
        fn struct_() {
            assert_tokens(&AdjacentlyTagged::Struct { subject: Subject }.readable(), &[
                Token::Struct { name: "AdjacentlyTagged", len: 2 },
                    Token::Str("tag"),
                    Token::Str("Struct"),
                    Token::Str("content"),
                    Token::Struct { name: "Struct", len: 1 },
                        Token::Str("subject"),
                        Token::Struct { name: "Readable", len: 1 },
                            Token::Str("mode"),
                            Token::Str("readable"),
                        Token::StructEnd,
                    Token::StructEnd,
                Token::StructEnd,
            ]);
        }
    }

    mod untagged_enum {
        use super::*;

        #[test]
        fn newtype() {
            assert_tokens(&Untagged::Newtype(Subject).readable(), &[
                Token::Struct { name: "Readable", len: 1 },
                    Token::Str("mode"),
                    Token::Str("readable"),
                Token::StructEnd,
            ]);
        }

        #[test]
        fn tuple() {
            assert_tokens(&Untagged::Tuple((), Subject).readable(), &[
                Token::Tuple { len: 2 },
                    Token::Unit,
                    Token::Struct { name: "Readable", len: 1 },
                        Token::Str("mode"),
                        Token::Str("readable"),
                    Token::StructEnd,
                Token::TupleEnd,
            ]);
        }

        #[test]
        fn struct_() {
            assert_tokens(&Untagged::Struct { subject: Subject }.readable(), &[
                Token::Struct { name: "Untagged", len: 1 },
                    Token::Str("subject"),
                    Token::Struct { name: "Readable", len: 1 },
                        Token::Str("mode"),
                        Token::Str("readable"),
                    Token::StructEnd,
                Token::StructEnd,
            ]);
        }
    }
}

mod compact {
    use super::*;

    #[test]
    fn bare() {
        assert_tokens(&Subject.compact(), &[
            Token::Struct { name: "Compact", len: 1 },
                Token::Str("mode"),
                Token::U32(42),
            Token::StructEnd,
        ]);
    }

    #[test]
    fn tuple() {
        assert_tokens(&((), Subject).compact(), &[
            Token::Tuple { len: 2 },
                Token::Unit,
                Token::Struct { name: "Compact", len: 1 },
                    Token::Str("mode"),
                    Token::U32(42),
                Token::StructEnd,
            Token::TupleEnd,
        ]);
    }

    #[test]
    fn newtype() {
        assert_tokens(&Newtype(Subject).compact(), &[
            Token::NewtypeStruct { name: "Newtype" },
            Token::Struct { name: "Compact", len: 1 },
                Token::Str("mode"),
                Token::U32(42),
            Token::StructEnd,
        ]);
    }

    #[test]
    fn named_tuple() {
        assert_tokens(&Tuple((), Subject).compact(), &[
            Token::TupleStruct { name: "Tuple", len: 2 },
                Token::Unit,
                Token::Struct { name: "Compact", len: 1 },
                    Token::Str("mode"),
                    Token::U32(42),
                Token::StructEnd,
            Token::TupleStructEnd,
        ]);
    }

    #[test]
    fn struct_() {
        assert_tokens(&Struct { subject: Subject }.compact(), &[
            Token::Struct { name: "Struct", len: 1 },
                Token::Str("subject"),
                Token::Struct { name: "Compact", len: 1 },
                    Token::Str("mode"),
                    Token::U32(42),
                Token::StructEnd,
            Token::StructEnd,
        ]);
    }

    mod externally_tagged_enum {
        use super::*;

        #[test]
        fn newtype() {
            assert_tokens(&ExternallyTagged::Newtype(Subject).compact(), &[
                Token::NewtypeVariant { name: "ExternallyTagged", variant: "Newtype" },
                Token::Struct { name: "Compact", len: 1 },
                    Token::Str("mode"),
                    Token::U32(42),
                Token::StructEnd,
            ]);
        }

        #[test]
        fn tuple() {
            assert_tokens(&ExternallyTagged::Tuple((), Subject).compact(), &[
                Token::TupleVariant { name: "ExternallyTagged", variant: "Tuple", len: 2 },
                    Token::Unit,
                    Token::Struct { name: "Compact", len: 1 },
                        Token::Str("mode"),
                        Token::U32(42),
                    Token::StructEnd,
                Token::TupleVariantEnd,
            ]);
        }

        #[test]
        fn struct_() {
            assert_tokens(&ExternallyTagged::Struct { subject: Subject }.compact(), &[
                Token::StructVariant { name: "ExternallyTagged", variant: "Struct", len: 1 },
                    Token::Str("subject"),
                    Token::Struct { name: "Compact", len: 1 },
                        Token::Str("mode"),
                        Token::U32(42),
                    Token::StructEnd,
                Token::StructVariantEnd,
            ]);
        }
    }

    mod internally_tagged_enum {
        use super::*;

        #[test]
        fn newtype() {
            assert_tokens(&InternallyTagged::Newtype(Subject).compact(), &[
                Token::Struct { name: "Compact", len: 2 },
                    Token::Str("tag"),
                    Token::Str("Newtype"),
                    Token::Str("mode"),
                    Token::U32(42),
                Token::StructEnd,
            ]);
        }

        #[test]
        fn struct_() {
            assert_tokens(&InternallyTagged::Struct { subject: Subject }.compact(), &[
                Token::Struct { name: "InternallyTagged", len: 2 },
                    Token::Str("tag"),
                    Token::Str("Struct"),
                    Token::Str("subject"),
                    Token::Struct { name: "Compact", len: 1 },
                        Token::Str("mode"),
                        Token::U32(42),
                    Token::StructEnd,
                Token::StructEnd,
            ]);
        }
    }

    mod adjacently_tagged_enum {
        use super::*;

        #[test]
        fn newtype() {
            assert_tokens(&AdjacentlyTagged::Newtype(Subject).compact(), &[
                Token::Struct { name: "AdjacentlyTagged", len: 2 },
                    Token::Str("tag"),
                    Token::Str("Newtype"),
                    Token::Str("content"),
                    Token::Struct { name: "Compact", len: 1 },
                        Token::Str("mode"),
                        Token::U32(42),
                    Token::StructEnd,
                Token::StructEnd,
            ]);
        }

        #[test]
        fn tuple() {
            assert_tokens(&AdjacentlyTagged::Tuple((), Subject).compact(), &[
                Token::Struct { name: "AdjacentlyTagged", len: 2 },
                    Token::Str("tag"),
                    Token::Str("Tuple"),
                    Token::Str("content"),
                    Token::Tuple { len: 2 },
                        Token::Unit,
                        Token::Struct { name: "Compact", len: 1 },
                            Token::Str("mode"),
                            Token::U32(42),
                        Token::StructEnd,
                    Token::TupleEnd,
                Token::StructEnd,
            ]);
        }

        #[test]
        fn struct_() {
            assert_tokens(&AdjacentlyTagged::Struct { subject: Subject }.compact(), &[
                Token::Struct { name: "AdjacentlyTagged", len: 2 },
                    Token::Str("tag"),
                    Token::Str("Struct"),
                    Token::Str("content"),
                    Token::Struct { name: "Struct", len: 1 },
                        Token::Str("subject"),
                        Token::Struct { name: "Compact", len: 1 },
                            Token::Str("mode"),
                            Token::U32(42),
                        Token::StructEnd,
                    Token::StructEnd,
                Token::StructEnd,
            ]);
        }
    }

    mod untagged_enum {
        use super::*;

        #[test]
        fn newtype() {
            assert_tokens(&Untagged::Newtype(Subject).compact(), &[
                Token::Struct { name: "Compact", len: 1 },
                    Token::Str("mode"),
                    Token::U32(42),
                Token::StructEnd,
            ]);
        }

        #[test]
        fn tuple() {
            assert_tokens(&Untagged::Tuple((), Subject).compact(), &[
                Token::Tuple { len: 2 },
                    Token::Unit,
                    Token::Struct { name: "Compact", len: 1 },
                        Token::Str("mode"),
                        Token::U32(42),
                    Token::StructEnd,
                Token::TupleEnd,
            ]);
        }

        #[test]
        fn struct_() {
            assert_tokens(&Untagged::Struct { subject: Subject }.compact(), &[
                Token::Struct { name: "Untagged", len: 1 },
                    Token::Str("subject"),
                    Token::Struct { name: "Compact", len: 1 },
                        Token::Str("mode"),
                        Token::U32(42),
                    Token::StructEnd,
                Token::StructEnd,
            ]);
        }
    }
}
