#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(bound(unknown))]
    x: (),
}

fn main() {}
