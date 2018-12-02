#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(variant_identifier)]
enum F {
    A,
    B,
}

fn main() {}
