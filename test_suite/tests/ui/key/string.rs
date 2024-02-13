use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(key = "a")]
    x: (),
}

fn main() {}
