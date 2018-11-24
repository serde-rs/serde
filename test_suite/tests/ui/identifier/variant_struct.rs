#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(variant_identifier)]
struct S;

fn main() {}
