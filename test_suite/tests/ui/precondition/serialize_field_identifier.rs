use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(field_identifier)]
enum F {
    A,
    B,
}

fn main() {}
