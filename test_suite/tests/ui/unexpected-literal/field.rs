use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde("literal")]
    x: (),
}

fn main() {}
