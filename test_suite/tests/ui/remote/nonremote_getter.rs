#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(getter = "S::get")]
    a: u8,
}

impl S {
    fn get(&self) -> u8 {
        self.a
    }
}

fn main() {}
