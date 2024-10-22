//! Ensures that error message points to the path in attribute
use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default = "main")]
struct Tuple(u8, #[serde(default = "main")] i8);

fn main() {}
