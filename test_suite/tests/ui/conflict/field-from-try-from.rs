use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(from = "u64", try_from = "u64", deserialize_with = "de_unit")]
    a: u8,
}

fn main() {}
