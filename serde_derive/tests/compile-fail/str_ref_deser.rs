#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)] //~ ERROR: custom derive attribute panicked
struct Test<'a> {
    s: &'a str, //~^^ HELP: Serde does not support deserializing fields of type &str
}

fn main() {}
