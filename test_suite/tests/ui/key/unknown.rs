use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(key = g)]
    x: (),
}

fn main() {}
