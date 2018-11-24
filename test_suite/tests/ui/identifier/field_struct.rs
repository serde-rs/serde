#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(field_identifier)]
struct S;

fn main() {}
