#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
enum E {
    #[serde(abc = "xyz")]
    V,
}

fn main() {}
