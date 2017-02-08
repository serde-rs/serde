#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
enum E {
    #[serde(abc="xyz")] //~^^ HELP: unknown serde variant attribute `abc`
    V,
}

fn main() { }
