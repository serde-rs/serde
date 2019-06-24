use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename = b'a')]
    byte_character: (),
}

fn main() {}
