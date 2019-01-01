use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(field_identifier)]
enum F {
    A,
    #[serde(other)]
    Other,
    B,
}

fn main() {}
