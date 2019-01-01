use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename = "x", serialize = "y")]
    x: (),
}

fn main() {}
