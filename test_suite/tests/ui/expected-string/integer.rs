use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename = 100)]
    integer: (),
}

fn main() {}
