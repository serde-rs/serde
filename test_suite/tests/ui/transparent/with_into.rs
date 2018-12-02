#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(transparent, into = "u64")]
struct S {
    a: u8,
}

fn main() {}
