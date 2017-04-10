#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S {
        a: u8,
    }
}

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
#[serde(remote = "~~~")] //~^ HELP: failed to parse path: "~~~"
struct S {
    a: u8,
}

fn main() {}
