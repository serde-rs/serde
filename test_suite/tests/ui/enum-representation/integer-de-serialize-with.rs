use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(integer)]
enum E {
    #[serde(serialize_with = "serialize_some_variant")]
    A,
    B
}

#[derive(Serialize, Deserialize)]
#[serde(integer)]
enum F {
    A,
    #[serde(deserialize_with = "deserialize_some_variant")]
    B
}

fn main() {}
