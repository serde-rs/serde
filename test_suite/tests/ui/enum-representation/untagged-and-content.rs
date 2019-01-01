use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(untagged)]
#[serde(content = "c")]
enum E {
    A(u8),
    B(String),
}

fn main() {}
