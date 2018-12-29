#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
struct S<'de> {
    s: &'de str,
}

fn main() {}
