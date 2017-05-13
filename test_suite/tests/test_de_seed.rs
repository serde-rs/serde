#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_test;

use std::cell::Cell;
use std::rc::Rc;

use serde::de::{Deserialize, Deserializer, DeserializeSeed};

use serde_test::{Token, assert_de_seed_tokens};

#[derive(Clone)]
struct Seed(Rc<Cell<i32>>);

#[derive(Deserialize, Debug, PartialEq)]
struct Inner;

struct InnerSeed(Rc<Cell<i32>>);

impl From<Seed> for InnerSeed {
    fn from(value: Seed) -> Self {
        InnerSeed(value.0.clone())
    }
}

impl<'de> DeserializeSeed<'de> for InnerSeed {
    type Value = Inner;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer<'de>
    {
        self.0.set(self.0.get() + 1);
        Inner::deserialize(deserializer)
    }
}

#[derive(DeserializeSeed, Debug, PartialEq)]
#[serde(deserialize_seed = "Seed")]
struct SeedStruct {
    #[serde(deserialize_seed_with = "InnerSeed::from")]
    value: Inner,
    #[serde(deserialize_seed_with = "InnerSeed::from")]
    value2: Inner,
    value3: Inner,
}

#[test]
fn test_deserialize_seed() {
    let value = SeedStruct { value: Inner, value2: Inner, value3: Inner };
    let seed = Seed(Rc::new(Cell::new(0)));
    assert_de_seed_tokens(
        seed.clone(),
        &value,
        &[
            Token::Struct {
                name: "SeedStruct",
                len: 1,
            },

            Token::Str("value"),
            Token::UnitStruct { name: "Inner" },

            Token::Str("value2"),
            Token::UnitStruct { name: "Inner" },

            Token::Str("value3"),
            Token::UnitStruct { name: "Inner" },

            Token::StructEnd,
        ],
    );

    assert_eq!(seed.0.get(), 2);
}
