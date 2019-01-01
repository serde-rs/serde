use serde_derive::Serialize;

#[derive(Serialize)]
struct Foo {
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    other: Option<Other>,
}

#[derive(Serialize)]
struct Other {
    x: u32,
}

fn main() {}
