#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
enum Enum {
    #[serde(deserialize_with = "deserialize_some_newtype_variant")]
    Newtype(#[serde(skip_deserializing)] String),
}

fn main() {}
