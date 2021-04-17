use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(try_from = "u64", from_str)]
struct S {
    a: u8,
}

fn main() {}
