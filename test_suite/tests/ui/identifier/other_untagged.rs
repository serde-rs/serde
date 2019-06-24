use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
enum F {
    #[serde(other)]
    Other,
}

fn main() {}
