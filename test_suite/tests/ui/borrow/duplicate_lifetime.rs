use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Test<'a> {
    #[serde(borrow = "'a + 'a")]
    s: &'a str,
}

fn main() {}
