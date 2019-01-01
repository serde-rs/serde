use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(transparent)]
struct S {
    a: u8,
    b: u8,
}

fn main() {}
