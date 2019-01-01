use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename = 'a')]
    character: (),
}

fn main() {}
