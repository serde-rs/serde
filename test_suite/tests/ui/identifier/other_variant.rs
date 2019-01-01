use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(variant_identifier)]
enum F {
    #[serde(other)]
    Other,
}

fn main() {}
