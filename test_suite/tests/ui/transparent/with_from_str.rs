use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(transparent, from_str)]
struct S {
    a: u8,
}

fn main() {}
