#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
struct Foo {
    #[serde(flatten, skip_deserializing)]
    other: Other,
}

#[derive(Deserialize)]
struct Other {
    x: u32,
}

fn main() {}
