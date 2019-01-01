use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Test<'a> {
    #[serde(borrow = "zzz")]
    s: &'a str,
}

fn main() {}
