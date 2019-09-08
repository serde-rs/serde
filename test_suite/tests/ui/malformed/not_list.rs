use serde_derive::Serialize;

#[derive(Serialize)]
#[serde]
#[serde = "?"]
struct S;

fn main() {}
