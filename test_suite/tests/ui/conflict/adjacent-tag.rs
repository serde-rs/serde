#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(tag = "conflict", content = "conflict")]
enum E {
    A,
    B,
}

fn main() {}
