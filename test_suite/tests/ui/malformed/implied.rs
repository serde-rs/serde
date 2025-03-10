use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(implied(unknown))]
struct S {
    x: (),
}

fn main() {}
