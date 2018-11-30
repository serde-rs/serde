#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(rename = 100)]
    integer: (),
}

fn main() {}
