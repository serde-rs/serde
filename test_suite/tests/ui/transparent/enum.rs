use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(transparent)]
enum E {}

fn main() {}
