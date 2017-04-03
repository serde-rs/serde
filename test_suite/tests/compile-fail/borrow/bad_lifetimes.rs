#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: proc-macro derive panicked
struct Test<'a> {
    #[serde(borrow = "zzz")] //~^^ HELP: failed to parse borrowed lifetimes: "zzz"
    s: &'a str,
}

fn main() {}
