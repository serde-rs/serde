#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
enum Enum {
    #[serde(serialize_with = "serialize_some_unit_variant")]
    #[serde(skip_serializing)]
    Unit,
}

fn main() {}
