use serde_derive::Deserialize;

#[derive(Deserialize)]
union Union {
    x: u8,
    y: (),
}

fn main() {}
