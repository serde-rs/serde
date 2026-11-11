use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "tag")]
enum E1 {
    V1 { f: u8 },
    #[serde(default, skip)]
    V2 { f: u8 },
}

#[derive(Deserialize)]
#[serde(tag = "tag")]
enum E2 {
    V1 { f: u8 },
    #[serde(default, skip_deserializing)]
    V2 { f: u8 },
}

fn main() {}
