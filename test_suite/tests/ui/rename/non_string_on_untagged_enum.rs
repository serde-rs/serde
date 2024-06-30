use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(untagged)]
enum S {
    #[serde(rename = 1)]
    A,
}

fn main() {}
