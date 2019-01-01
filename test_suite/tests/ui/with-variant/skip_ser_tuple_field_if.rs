use serde_derive::Serialize;

#[derive(Serialize)]
enum Enum {
    #[serde(serialize_with = "serialize_some_other_variant")]
    Tuple(#[serde(skip_serializing_if = "always")] String, u8),
}

fn main() {}
