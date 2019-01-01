use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename(unknown))]
    x: (),
}

fn main() {}
