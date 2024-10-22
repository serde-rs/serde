// Tests that type error points to the path in attribute

use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default = "main")]
struct Newtype(#[serde(default = "main")] u8);

fn main() {}
