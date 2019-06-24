use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(field_identifier)]
enum F {
    A,
    #[serde(other)]
    Other(u8, u8),
}

fn main() {}
