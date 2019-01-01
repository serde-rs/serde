use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename(serialize = "x"))]
    #[serde(rename(serialize = "y"))]
    x: (),
}

fn main() {}
