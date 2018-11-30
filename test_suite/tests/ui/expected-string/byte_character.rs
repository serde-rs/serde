#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(rename = b'a')]
    byte_character: (),
}

fn main() {}
