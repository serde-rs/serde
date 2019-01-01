use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
struct T(u8, u8);

fn main() {}
