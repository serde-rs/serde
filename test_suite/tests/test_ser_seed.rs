#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_test;

use std::cell::Cell;

use serde::Serialize;
use serde::ser::Seeded;

use serde_test::{Token, assert_ser_tokens};

#[derive(Serialize)]
struct Inner;

impl serde::ser::SerializeSeed for Inner {
    type Seed = Cell<i32>;

    fn serialize_seed<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        seed.set(seed.get() + 1);
        self.serialize(serializer)
    }
}

#[derive(SerializeSeed)]
#[serde(seed = "Cell<i32>")]
struct SeedStruct {
    #[serde(serialize_seed)]
    value: Inner,
}

#[test]
fn test_serialize_seed() {
    let value = SeedStruct { value: Inner };
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value),
        &[
            Token::Struct {
                name: "SeedStruct",
                len: 1,
            },

            Token::Str("value"),
            Token::UnitStruct { name: "Inner" },

            Token::StructEnd,
        ],
    );

    assert_eq!(seed.get(), 1);
}

#[test]
fn test_serialize_vec_seed() {
    let value = [SeedStruct { value: Inner }, SeedStruct { value: Inner }];
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value[..]),
        &[
            Token::Seq { len: Some(2) },

            Token::Struct {
                name: "SeedStruct",
                len: 1,
            },

            Token::Str("value"),
            Token::UnitStruct { name: "Inner" },

            Token::StructEnd,

            Token::Struct {
                name: "SeedStruct",
                len: 1,
            },

            Token::Str("value"),
            Token::UnitStruct { name: "Inner" },

            Token::StructEnd,

            Token::SeqEnd,
        ],
    );

    assert_eq!(seed.get(), 2);
}

#[test]
fn test_serialize_option_some_seed() {
    let value = Some(SeedStruct { value: Inner });
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value),
        &[
            Token::Some,

            Token::Struct {
                name: "SeedStruct",
                len: 1,
            },

            Token::Str("value"),
            Token::UnitStruct { name: "Inner" },

            Token::StructEnd,
        ],
    );

    assert_eq!(seed.get(), 1);
}

#[test]
fn test_serialize_option_none_seed() {
    let value: Option<SeedStruct> = None;
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value),
        &[
            Token::None,
        ],
    );

    assert_eq!(seed.get(), 0);
}
