use serde_derive::Serialize;

#[derive(Serialize)]
enum E {
    #[serde(abc = "xyz")]
    V,
}

fn main() {}
