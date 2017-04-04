#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: proc-macro derive panicked
struct Test<'a> {
    #[serde(borrow = "'b")] //~^^ HELP: field `s` does not have lifetime 'b
    s: &'a str,
}

fn main() {}
