#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(field_identifier)]
enum F {
    A,
    B,
}

fn main() {}
