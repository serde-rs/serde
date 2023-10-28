use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(strict_or_some_other_name)]
enum E {
    S { a: u8 },
}

fn main() {}
