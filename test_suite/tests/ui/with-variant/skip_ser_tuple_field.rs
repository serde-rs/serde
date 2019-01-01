use serde_derive::Serialize;

#[derive(Serialize)]
enum Enum {
    #[serde(serialize_with = "serialize_some_other_variant")]
    Tuple(#[serde(skip_serializing)] String, u8),
}

fn main() {}
