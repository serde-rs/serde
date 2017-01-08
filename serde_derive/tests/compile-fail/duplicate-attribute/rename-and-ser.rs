#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: custom derive attribute panicked
struct S {
    #[serde(rename="x", serialize="y")] //~^^ HELP: unknown serde field attribute `serialize`
    x: (),
}

fn main() {}
