use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
union Union {
    f: u8,
}

fn main() {}
