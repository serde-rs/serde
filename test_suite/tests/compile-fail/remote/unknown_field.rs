#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S {
        pub a: u8,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::S")]
struct S {
    b: u8, //~^^^ ERROR: no field `b` on type `&remote::S`
}

fn main() {}
