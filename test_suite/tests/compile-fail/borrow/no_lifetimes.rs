#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: proc-macro derive panicked
struct Test {
    #[serde(borrow)] //~^^ HELP: field `s` has no lifetimes to borrow
    s: String,
}

fn main() {}
