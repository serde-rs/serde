use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(transparent)]
struct S {
    #[serde(skip)]
    a: u8,
    #[serde(default)]
    b: u8,
}

fn main() {}
