use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Test<'a> {
    #[serde(borrow = "")]
    s: &'a str,
}

fn main() {}
