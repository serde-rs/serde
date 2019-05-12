use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(try_from = "Option<T")]
enum TestOne {
    Testing,
    One,
    Two,
    Three,
}

fn main() {}
