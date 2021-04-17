use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(transparent, to_string)]
struct S {
    a: u8,
}

fn main() {}
