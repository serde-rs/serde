#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde("literal")]
    x: (),
}

fn main() {}
