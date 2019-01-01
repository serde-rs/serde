use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(variant_identifier)]
enum F {
    A,
    B,
}

fn main() {}
