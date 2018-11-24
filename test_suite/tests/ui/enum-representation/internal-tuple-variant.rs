#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(tag = "type")]
enum E {
    Tuple(u8, u8),
}

fn main() {}
