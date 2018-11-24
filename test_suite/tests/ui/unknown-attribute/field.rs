#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct C {
    #[serde(abc = "xyz")]
    x: u32,
}

fn main() {}
