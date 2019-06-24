use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
struct S(u8, u8);

fn main() {}
