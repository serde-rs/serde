use serde_derive::Deserialize;

#[derive(Deserialize)]
struct S {
    string: String,
    slice: [u8],
}

fn main() {}
