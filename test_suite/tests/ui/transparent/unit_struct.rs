#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(transparent)]
struct S;

fn main() {}
