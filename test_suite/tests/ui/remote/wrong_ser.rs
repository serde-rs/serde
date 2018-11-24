#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S {
        pub a: u16,
    }
}

#[derive(Serialize)]
#[serde(remote = "remote::S")]
struct S {
    a: u8,
}

fn main() {}
