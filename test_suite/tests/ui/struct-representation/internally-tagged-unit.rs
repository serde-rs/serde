use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
struct U;

fn main() {}
