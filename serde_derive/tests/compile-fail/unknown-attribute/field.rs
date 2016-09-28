#![feature(rustc_macro)]

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: custom derive attribute panicked
struct C {
    #[serde(abc="xyz")] // ERROR: unknown serde field attribute `abc`
    x: u32,
}

fn main() { }
