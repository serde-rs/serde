use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Test {
    #[serde(borrow)]
    s: String,
}

fn main() {}
