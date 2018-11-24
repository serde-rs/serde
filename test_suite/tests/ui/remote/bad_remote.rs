#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S {
        a: u8,
    }
}

#[derive(Serialize)]
#[serde(remote = "~~~")]
struct S {
    a: u8,
}

fn main() {}
