#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(untagged)]
struct S;

fn main() {}
