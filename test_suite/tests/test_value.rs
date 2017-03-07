#[macro_use]
extern crate serde_derive;

extern crate serde;
use serde::Deserialize;
use serde::de::value::{self, ValueDeserializer};

#[test]
fn test_u32_to_enum() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum E {
        A,
        B,
    }

    let deserializer = ValueDeserializer::<value::Error>::into_deserializer(1u32);
    let e: E = E::deserialize(deserializer).unwrap();
    assert_eq!(E::B, e);
}
