use serde_derive::Deserialize;

#[derive(Deserialize)]
enum Enum {
    #[serde(deserialize_with = "deserialize_some_unit_variant")]
    #[serde(skip_deserializing)]
    Unit,
}

fn main() {}
