use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(field_identifier)]
enum F {
    A,
    Other(String),
    B,
}

fn main() {}
