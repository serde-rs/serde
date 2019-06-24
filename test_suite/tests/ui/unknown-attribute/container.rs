use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(abc = "xyz")]
struct A {
    x: u32,
}

fn main() {}
