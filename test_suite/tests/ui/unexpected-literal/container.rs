#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde("literal")]
struct S;

fn main() {}
