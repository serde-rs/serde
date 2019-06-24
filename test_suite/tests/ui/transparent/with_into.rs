use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(transparent, into = "u64")]
struct S {
    a: u8,
}

fn main() {}
