#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(field_identifier)]
enum F {
    A,
    B(u8, u8),
}

fn main() {}
