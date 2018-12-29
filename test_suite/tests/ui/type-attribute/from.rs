#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(from = "Option<T")]
enum TestOne {
    Testing,
    One,
    Two,
    Three,
}

fn main() {}
