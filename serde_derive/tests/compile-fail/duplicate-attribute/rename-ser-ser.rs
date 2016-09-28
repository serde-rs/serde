#![feature(rustc_macro)]

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: custom derive attribute panicked
struct S {
    #[serde(rename(serialize="x", serialize="y"))] //~^^ HELP: duplicate serde attribute `rename`
    x: (),
}

fn main() {}
