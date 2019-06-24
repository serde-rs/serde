use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(tag = "conflict", content = "conflict")]
enum E {
    A,
    B,
}

fn main() {}
