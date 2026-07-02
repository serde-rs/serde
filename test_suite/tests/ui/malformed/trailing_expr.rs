use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(skip_serializing_if, x.is_empty())]
    x: Vec<()>,
}

fn main() {}
