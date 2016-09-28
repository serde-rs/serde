#![feature(rustc_macro)]

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: custom derive attribute panicked
struct S {
    #[serde(rename="x")]
    #[serde(rename(deserialize="y"))] // ERROR: duplicate serde attribute `rename`
    x: (),
}

fn main() {}
