// Tests that type error points to the path in attribute

use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default = "main")]
struct Struct {
    #[serde(default = "main")]
    f1: u8,
    f2: u8,
    #[serde(default = "main")]
    f3: i8,
}

fn main() {}
