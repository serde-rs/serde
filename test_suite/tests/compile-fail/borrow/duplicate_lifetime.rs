#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: proc-macro derive panicked
struct Test<'a> {
    #[serde(borrow = "'a + 'a")] //~^^ HELP: duplicate borrowed lifetime `'a`
    s: &'a str,
}

fn main() {}
