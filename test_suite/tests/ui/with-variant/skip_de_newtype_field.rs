use serde_derive::Deserialize;

#[derive(Deserialize)]
enum Enum {
    #[serde(deserialize_with = "deserialize_some_newtype_variant")]
    Newtype(#[serde(skip_deserializing)] String),
}

fn main() {}
