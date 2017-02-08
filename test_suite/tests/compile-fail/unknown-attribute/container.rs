#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
#[serde(abc="xyz")] //~^ HELP: unknown serde container attribute `abc`
struct A {
    x: u32,
}

fn main() { }
