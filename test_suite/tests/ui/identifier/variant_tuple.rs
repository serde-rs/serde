use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(variant_identifier)]
enum F {
    A,
    B(u8, u8),
}

fn main() {}
