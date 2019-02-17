use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(integer)]
enum E {
    A(u8),
    B(String),
}

fn main() {}
