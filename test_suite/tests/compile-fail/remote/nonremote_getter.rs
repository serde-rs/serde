#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
struct S {
    #[serde(getter = "S::get")] //~^^ HELP: #[serde(getter = "...")] can only be used in structs that have #[serde(remote = "...")]
    a: u8,
}

impl S {
    fn get(&self) -> u8 {
        self.a
    }
}

fn main() {}
