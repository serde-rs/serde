use serde_derive::Deserialize;

#[derive(Deserialize)]
struct T {
    #[serde(skip_serializing_if = "always", skip_serializing_if_default)]
    a: u8,
}

fn main() {}
