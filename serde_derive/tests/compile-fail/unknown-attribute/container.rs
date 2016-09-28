#![feature(rustc_macro)]

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: custom derive attribute panicked
#[serde(abc="xyz")] //~^ HELP: unknown serde container attribute `abc`
struct A {
    x: u32,
}

fn main() { }
