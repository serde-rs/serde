#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S {
        pub a: u8,
        pub b: u8,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::S")]
struct S {
    a: u8,
}

fn main() {}
