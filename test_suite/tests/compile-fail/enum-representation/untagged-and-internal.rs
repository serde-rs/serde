#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
#[serde(untagged)]
#[serde(tag = "type")] //~^^ HELP: enum cannot be both untagged and internally tagged
enum E {
    A(u8),
    B(String),
}

fn main() {}
