#![feature(test, custom_attribute, custom_derive, plugin, specialization)]
#![plugin(serde_macros)]

extern crate test;

include!("../../testing/tests/test.rs.in");

mod compile_tests;

extern crate serde_test;
use serde_test::{
    Token,
    Deserializer,
    assert_de_tokens,
    Serializer,
    assert_ser_tokens,
};

#[test]
fn tagging_serialization() {
    struct TaggedValue(String);

    trait SerializeTag<S: serde::Serialize>: serde::Serializer {
        fn serialize_tag(&mut self, value: &S) -> Result<(), <Self as serde::Serializer>::Error>;
    }

    impl<S: serde::Serializer> SerializeTag<TaggedValue> for S {
        default fn serialize_tag(&mut self, _value: &TaggedValue) -> Result<(), S::Error> {
            unimplemented!()
        }
    }

    impl<'a, I: Iterator<Item=&'a Token<'a>>> SerializeTag<TaggedValue> for Serializer<'a, I> {
        fn serialize_tag(&mut self, value: &TaggedValue) -> Result<(), Self::Error> {
            use serde::Serializer;
            self.serialize_tagged_value(42, &value.0)
        }
    }

    impl serde::Serialize for TaggedValue {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            serializer.serialize_tag(self)
        }
    }
    assert_ser_tokens(
        &TaggedValue("cake".to_string()),
        &[
            Token::Tag,
            Token::I32(42),
            Token::Str("cake"),
        ],
    );
}

#[test]
fn tagging_deserialization() {
    #[derive(Debug, PartialEq, Eq)]
    struct TaggedValue(String);

    impl serde::Deserialize for TaggedValue {
        fn deserialize<D>(deserializer: &mut D) -> Result<TaggedValue, D::Error>
            where D: serde::Deserializer,
        {
            deserializer.deserialize_tag()
        }
    }
    assert_de_tokens(
        &TaggedValue("cake".to_string()),
        &[
            Token::Tag,
            Token::I32(42),
            Token::String("cake".to_owned()),
        ],
    );

    trait DeserializeTag<S: serde::Deserialize>: serde::Deserializer {
        fn deserialize_tag(&mut self) -> Result<S, <Self as serde::Deserializer>::Error>;
    }

    impl<I: Iterator<Item=Token<'static>>> DeserializeTag<TaggedValue> for Deserializer<I> {
        fn deserialize_tag(&mut self) -> Result<TaggedValue, Self::Error> {
            use serde::Deserializer;
            Ok(TaggedValue(try!(self.deserialize_tagged_value())))
        }
    }

    impl<S: serde::Deserializer> DeserializeTag<TaggedValue> for S {
        default fn deserialize_tag(&mut self) -> Result<TaggedValue, S::Error> {
            unimplemented!()
        }
    }
}
