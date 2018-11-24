#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(transparent)]
struct S {
    a: u8,
    b: u8,
}

fn main() {}
