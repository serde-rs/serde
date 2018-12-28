#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(tag = "type")]
struct U;

fn main() {}
