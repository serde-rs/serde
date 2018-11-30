#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(default = "default_t")]
struct T(u8, u8);

fn main() {}
