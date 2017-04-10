#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S {
        a: u8,
    }

    impl S {
        pub fn get(&self) -> u16 {
            self.a as u16
        }
    }
}

#[derive(Serialize)] //~ ERROR: mismatched types
#[serde(remote = "remote::S")]
struct S {
    #[serde(getter = "remote::S::get")]
    a: u8, //~^^^^ expected u8, found u16
}

fn main() {}
