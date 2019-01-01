use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(with = "w", serialize_with = "s")]
    x: (),
}

fn main() {}
