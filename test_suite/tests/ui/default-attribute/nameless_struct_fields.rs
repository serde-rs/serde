#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(default)]
struct T(u8, u8);

fn main() {}
