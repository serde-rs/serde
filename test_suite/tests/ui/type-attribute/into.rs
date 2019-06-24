use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(into = "Option<T")]
enum TestOne {
    Testing,
    One,
    Two,
    Three,
}

fn main() {}
