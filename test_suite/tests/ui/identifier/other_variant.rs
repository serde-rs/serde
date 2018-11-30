#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(variant_identifier)]
enum F {
    #[serde(other)]
    Other,
}

fn main() {}
