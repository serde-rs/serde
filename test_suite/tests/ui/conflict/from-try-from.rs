use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(from = "u64", try_from = "u64")]
struct S {
    a: u8,
}

fn main() {}
