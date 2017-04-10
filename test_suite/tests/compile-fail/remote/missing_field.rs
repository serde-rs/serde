#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S {
        pub a: u8,
        pub b: u8,
    }
}

#[derive(Serialize, Deserialize)] //~ ERROR: missing field `b` in initializer of `remote::S`
#[serde(remote = "remote::S")]
struct S {
    a: u8, //~^^^ ERROR: missing field `b` in initializer of `remote::S`
}

fn main() {}
