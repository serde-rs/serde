use serde_derive::Serialize;

#[derive(Serialize)]
struct C {
    #[serde(abc = "xyz")]
    x: u32,
}

fn main() {}
