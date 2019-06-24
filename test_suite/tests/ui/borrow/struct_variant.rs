use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Str<'a>(&'a str);

#[derive(Deserialize)]
enum Test<'a> {
    #[serde(borrow)]
    S { s: Str<'a> },
}

fn main() {}
