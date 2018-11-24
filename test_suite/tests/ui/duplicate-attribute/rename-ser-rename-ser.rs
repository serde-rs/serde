#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(rename(serialize = "x"), rename(serialize = "y"))]
    x: (),
}

fn main() {}
