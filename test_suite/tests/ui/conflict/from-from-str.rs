use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(from = "u64", from_str)]
struct S {
    a: u8,
}

fn main() {}
