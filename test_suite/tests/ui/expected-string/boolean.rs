use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename = true)]
    boolean: (),
}

fn main() {}
