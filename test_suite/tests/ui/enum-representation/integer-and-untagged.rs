use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(integer)]
#[serde(untagged)]
enum E {
    A(u8),
    B(String),
}

fn main() {}
