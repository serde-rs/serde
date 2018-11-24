#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S {
        a: u8,
    }
}

#[derive(Serialize)]
#[serde(remote = "remote::S")]
struct S {
    #[serde(getter = "~~~")]
    a: u8,
}

fn main() {}
