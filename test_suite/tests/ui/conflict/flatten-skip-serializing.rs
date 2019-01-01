use serde_derive::Serialize;

#[derive(Serialize)]
struct Foo {
    #[serde(flatten, skip_serializing)]
    other: Other,
}

#[derive(Serialize)]
struct Other {
    x: u32,
}

fn main() {}
