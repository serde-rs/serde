#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
enum Enum {
    #[serde(serialize_with = "serialize_some_newtype_variant")]
    Newtype(#[serde(skip_serializing)] String),
}

fn main() {}
