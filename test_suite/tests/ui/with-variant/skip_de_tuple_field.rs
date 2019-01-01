use serde_derive::Deserialize;

#[derive(Deserialize)]
enum Enum {
    #[serde(deserialize_with = "deserialize_some_other_variant")]
    Tuple(#[serde(skip_deserializing)] String, u8),
}

fn main() {}
