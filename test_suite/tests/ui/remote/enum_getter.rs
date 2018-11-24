#[macro_use]
extern crate serde_derive;

mod remote {
    pub enum E {
        A { a: u8 },
    }
}

#[derive(Serialize)]
#[serde(remote = "remote::E")]
pub enum E {
    A {
        #[serde(getter = "get_a")]
        a: u8,
    },
}

fn main() {}
