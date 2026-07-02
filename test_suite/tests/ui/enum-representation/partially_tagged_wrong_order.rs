use serde_derive::Serialize;

#[derive(Serialize)]
enum E {
    #[serde(untagged)]
    A(u8),
    B(String),
}

fn main() {}
