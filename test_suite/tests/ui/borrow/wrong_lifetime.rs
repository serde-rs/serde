use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Test<'a> {
    #[serde(borrow = "'b")]
    s: &'a str,
}

fn main() {}
