use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(transparent)]
struct S {
    #[serde(skip)]
    a: u8,
}

fn main() {}
