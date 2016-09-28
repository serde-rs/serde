#![feature(rustc_macro)]

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: custom derive attribute panicked
struct S {
    #[serde(rename(serialize="x"))]
    #[serde(rename(serialize="y"))] // ERROR: duplicate serde attribute `rename`
    x: (),
}

fn main() {}
