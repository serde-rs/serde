#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(tag = "type")]
struct S(u8, u8);

fn main() {}
