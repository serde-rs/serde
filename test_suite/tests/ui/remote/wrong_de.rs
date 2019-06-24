use serde_derive::Deserialize;

mod remote {
    pub struct S(pub u16);
}

#[derive(Deserialize)]
#[serde(remote = "remote::S")]
struct S(u8);

fn main() {}
