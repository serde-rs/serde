use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename = "x")]
    #[serde(rename(deserialize = "y"))]
    x: (),
}

fn main() {}
