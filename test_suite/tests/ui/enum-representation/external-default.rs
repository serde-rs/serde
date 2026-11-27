use serde_derive::Deserialize;

#[derive(Deserialize)]
enum E {
    V1 { f: u8 },
    #[serde(default)]
    V2 { f: u8 },
}

fn main() {}
