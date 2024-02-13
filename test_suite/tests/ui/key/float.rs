use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(key = 1.1)]
    x: (),
}

fn main() {}
