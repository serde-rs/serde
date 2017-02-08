#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
struct S {
    #[serde(rename="x", serialize="y")] //~^^ HELP: unknown serde field attribute `serialize`
    x: (),
}

fn main() {}
