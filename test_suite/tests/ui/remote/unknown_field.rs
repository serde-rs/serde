use serde_derive::{Deserialize, Serialize};

mod remote {
    pub struct S {
        pub a: u8,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::S")]
struct S {
    b: u8,
}

fn main() {}
