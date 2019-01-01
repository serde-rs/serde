use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename = 3.14)]
    float: (),
}

fn main() {}
