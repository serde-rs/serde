#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)] //~ ERROR: proc-macro derive panicked
struct Test<'a> {
    s: &'a str, //~^^ HELP: Serde does not support deserializing fields of type &str
}

fn main() {}
