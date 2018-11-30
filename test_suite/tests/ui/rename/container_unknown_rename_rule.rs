#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(rename_all = "abc")]
struct S {
    name: u8,
    long_name: u8,
    very_long_name: u8,
}

fn main() {}
