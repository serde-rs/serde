#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: custom derive attribute panicked
struct S {
    #[serde(rename="x")]
    #[serde(rename(deserialize="y"))] //~^^^ HELP: duplicate serde attribute `rename`
    x: (),
}

fn main() {}
