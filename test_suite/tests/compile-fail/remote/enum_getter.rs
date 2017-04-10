#[macro_use]
extern crate serde_derive;

mod remote {
    pub enum E {
        A { a: u8 }
    }
}

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
#[serde(remote = "remote::E")]
pub enum E {
    A {
        #[serde(getter = "get_a")] //~^^^^ HELP: #[serde(getter = "...")] is not allowed in an enum
        a: u8,
    }
}

fn main() {}
