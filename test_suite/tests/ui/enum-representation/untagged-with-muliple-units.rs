use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(untagged)]
enum E1 {
    Unit1,
    Tuple1(usize),
    Unit2,
    Struct {},
    #[serde(skip_serializing)]
    Unit3,
    #[serde(skip_deserializing)]
    Unit4,
}

#[derive(Serialize)]
#[serde(untagged)]
enum E2 {
    Unit1,
    Tuple1(usize),
    Unit2,
    Struct {},
    #[serde(skip_serializing)]
    Unit3,
    #[serde(skip_deserializing)]
    Unit4,
}

fn main() {}
