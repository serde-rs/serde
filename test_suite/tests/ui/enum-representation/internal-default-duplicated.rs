use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "tag")]
enum E {
    #[serde(default)]
    V1 { f: u8 },
    #[serde(default)]
    V2 { f: u8 },
}

fn main() {}
