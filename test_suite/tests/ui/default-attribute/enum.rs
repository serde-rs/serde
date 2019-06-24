use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
enum E {
    S { f: u8 },
}

fn main() {}
