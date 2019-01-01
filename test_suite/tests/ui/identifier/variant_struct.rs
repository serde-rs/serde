use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(variant_identifier)]
struct S;

fn main() {}
