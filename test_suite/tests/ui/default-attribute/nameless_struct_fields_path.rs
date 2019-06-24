use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default = "default_t")]
struct T(u8, u8);

fn main() {}
