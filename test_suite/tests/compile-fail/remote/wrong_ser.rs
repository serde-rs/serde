#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S {
        pub a: u16,
    }
}

#[derive(Serialize)] //~ ERROR: mismatched types
#[serde(remote = "remote::S")]
struct S {
    a: u8, //~^^^ expected u8, found u16
}

fn main() {}
