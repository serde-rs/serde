use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
enum E {
    Tuple(u8, u8),
}

fn main() {}
