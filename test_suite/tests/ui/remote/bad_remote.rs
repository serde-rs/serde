use serde_derive::Serialize;

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
