use serde_derive::{Deserialize, Serialize};

mod remote {
    pub struct Struct<T, U> {
        pub t: T,
        pub u: U,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::StructGeneric<u8>")]
struct StructDef<U> {
    t: u8,
    u: U,
}

fn main() {}
