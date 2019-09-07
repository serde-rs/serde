use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(into = "u64", serialize_with = "ser_unit")]
    a: u8,
}

fn main() {}
