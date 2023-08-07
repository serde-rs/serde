use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default = "default_u")]
struct Unit;

fn main() {}
