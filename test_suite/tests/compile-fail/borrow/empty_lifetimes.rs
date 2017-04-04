#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: proc-macro derive panicked
struct Test<'a> {
    #[serde(borrow = "")] //~^^ HELP: at least one lifetime must be borrowed
    s: &'a str,
}

fn main() {}
