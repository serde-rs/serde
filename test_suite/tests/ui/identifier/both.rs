use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(field_identifier, variant_identifier)]
enum F {
    A,
    B,
}

fn main() {}
