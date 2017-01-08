#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: custom derive attribute panicked
struct C {
    #[serde(abc="xyz")] //~^^ HELP: unknown serde field attribute `abc`
    x: u32,
}

fn main() { }
