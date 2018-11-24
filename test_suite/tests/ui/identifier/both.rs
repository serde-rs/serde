#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(field_identifier, variant_identifier)]
enum F {
    A,
    B,
}

fn main() {}
