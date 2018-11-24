#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
struct Test<'a> {
    #[serde(borrow = "'a + 'a")]
    s: &'a str,
}

fn main() {}
