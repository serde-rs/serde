use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename = 100i128)]
    signed128: (),
    #[serde(rename = 101u128)]
    unsigned128: (),
    #[serde(rename = 500u8)]
    overflow: (),
}

fn main() {}
