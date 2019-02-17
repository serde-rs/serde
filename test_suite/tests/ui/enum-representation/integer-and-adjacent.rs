use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(integer)]
#[serde(tag = "t", content = "c")]
enum E {
    A(u8),
    B(String),
}

fn main() {}
