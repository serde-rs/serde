#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(abc = "xyz")]
struct A {
    x: u32,
}

fn main() {}
