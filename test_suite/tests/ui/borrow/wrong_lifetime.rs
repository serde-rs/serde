#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
struct Test<'a> {
    #[serde(borrow = "'b")]
    s: &'a str,
}

fn main() {}
