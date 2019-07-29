use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(transparent, try_from = "u64")]
struct S {
    a: u8,
}

fn main() {}
