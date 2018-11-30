#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(untagged)]
enum F {
    #[serde(other)]
    Other,
}

fn main() {}
