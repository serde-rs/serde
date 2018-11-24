#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(with = "w", serialize_with = "s")]
    x: (),
}

fn main() {}
