use serde_derive::Serialize;

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

#[derive(Serialize)]
#[serde(remote = "remote::S")]
struct S {
    #[serde(getter = "remote::S::get")]
    a: u8,
}

fn main() {}
