use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(tag = "conflict")]
struct S {
    #[serde(rename = "conflict")]
    x: (),
}

fn main() {}
