#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(rename = 3.14)]
    float: (),
}

fn main() {}
