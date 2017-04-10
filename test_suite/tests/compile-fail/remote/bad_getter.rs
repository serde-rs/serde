#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S {
        a: u8,
    }
}

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
#[serde(remote = "remote::S")]
struct S {
    #[serde(getter = "~~~")] //~^^^ HELP: failed to parse path: "~~~"
    a: u8,
}

fn main() {}
