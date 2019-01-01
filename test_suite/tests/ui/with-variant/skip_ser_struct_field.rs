use serde_derive::Serialize;

#[derive(Serialize)]
enum Enum {
    #[serde(serialize_with = "serialize_some_other_variant")]
    Struct {
        #[serde(skip_serializing)]
        f1: String,
        f2: u8,
    },
}

fn main() {}
