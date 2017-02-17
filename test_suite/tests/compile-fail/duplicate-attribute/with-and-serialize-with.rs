#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
struct S {
    #[serde(with = "w", serialize_with = "s")] //~^^ HELP: duplicate serde attribute `serialize_with`
    x: (),
}

fn main() {}
