#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(transparent)]
struct S {
    #[serde(skip)]
    a: u8,
    #[serde(default)]
    b: u8,
}

fn main() {}
