#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(field_identifier)]
enum F {
    A,
    Other(String),
    B,
}

fn main() {}
