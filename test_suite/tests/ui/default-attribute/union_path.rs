use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default = "default_u")]
union Union {
    f: u8,
}

fn main() {}
