use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(tag = "conflict")]
struct S {
    #[serde(alias = "conflict")]
    x: (),
}

fn main() {}
