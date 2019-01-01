use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(bound(unknown))]
    x: (),
}

fn main() {}
