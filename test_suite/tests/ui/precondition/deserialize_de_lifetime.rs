use serde_derive::Deserialize;

#[derive(Deserialize)]
struct S<'de> {
    s: &'de str,
}

fn main() {}
