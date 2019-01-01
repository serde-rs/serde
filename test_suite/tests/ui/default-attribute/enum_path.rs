use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default = "default_e")]
enum E {
    S { f: u8 },
}

fn main() {}
