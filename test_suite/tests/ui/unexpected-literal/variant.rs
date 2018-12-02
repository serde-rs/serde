#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
enum E {
    #[serde("literal")]
    V,
}

fn main() {}
