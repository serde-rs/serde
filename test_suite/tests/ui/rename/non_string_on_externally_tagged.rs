use serde_derive::Serialize;

#[derive(Serialize)]
enum S {
    #[serde(rename = 1)]
    A,
}

fn main() {}
