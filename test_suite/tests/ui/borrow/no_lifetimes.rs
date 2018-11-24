#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
struct Test {
    #[serde(borrow)]
    s: String,
}

fn main() {}
