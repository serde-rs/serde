#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
union Union {
    x: u8,
    y: (),
}

fn main() {}
