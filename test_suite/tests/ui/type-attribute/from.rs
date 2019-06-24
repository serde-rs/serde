use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(from = "Option<T")]
enum TestOne {
    Testing,
    One,
    Two,
    Three,
}

fn main() {}
