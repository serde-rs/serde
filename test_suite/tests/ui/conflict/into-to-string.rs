use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(into = "u64", to_string)]
struct S {
    a: u8,
}

fn main() {}
