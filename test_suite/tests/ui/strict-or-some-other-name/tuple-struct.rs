use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(strict_or_some_other_name)]
struct S(u8, u8);

fn main() {}