#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(transparent)]
enum E {}

fn main() {}
