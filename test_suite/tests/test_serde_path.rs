#![allow(
    clippy::extra_unused_type_parameters,
    clippy::type_repetition_in_bounds
)]

#[test]
fn test_gen_custom_serde() {
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(crate = "fake_serde")]
    struct Foo;

    // Would be overlapping if serde::Serialize were implemented
    impl AssertNotSerdeSerialize for Foo {}
    // Would be overlapping if serde::Deserialize were implemented
    impl<'a> AssertNotSerdeDeserialize<'a> for Foo {}

    fake_serde::assert::<Foo>();
}

mod fake_serde {
    pub use serde::*;

    pub fn assert<T>()
    where
        T: Serialize,
        T: for<'a> Deserialize<'a>,
    {
    }

    pub trait Serialize {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
    }

    pub trait Deserialize<'a>: Sized {
        fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error>;
    }
}

trait AssertNotSerdeSerialize {}

impl<T: serde::Serialize> AssertNotSerdeSerialize for T {}

trait AssertNotSerdeDeserialize<'a> {}

impl<'a, T: serde::Deserialize<'a>> AssertNotSerdeDeserialize<'a> for T {}
