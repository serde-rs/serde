use serde_derive::Deserialize;

#[derive(Deserialize)]
enum Enum {
    #[serde(deserialize_with = "deserialize_some_other_variant")]
    Struct {
        #[serde(skip_deserializing)]
        f1: String,
        f2: u8,
    },
}

fn main() {}
