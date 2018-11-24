#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(rename(serialize = "x", serialize = "y"))]
    x: (),
}

fn main() {}
