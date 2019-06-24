use serde_derive::{Serialize, Deserialize};

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
