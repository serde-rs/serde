use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Test<'a> {
    #[serde(borrow = "")]
    r: &'a str,
    #[serde(borrow = "  ")]
    s: &'a str,
}

fn main() {}
