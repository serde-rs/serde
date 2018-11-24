#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(rename = "x")]
    #[serde(rename(deserialize = "y"))]
    x: (),
}

fn main() {}
