#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
struct S {
    string: String,
    slice: [u8],
}

fn main() {}
