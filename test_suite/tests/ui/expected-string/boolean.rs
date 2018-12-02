#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(rename = true)]
    boolean: (),
}

fn main() {}
