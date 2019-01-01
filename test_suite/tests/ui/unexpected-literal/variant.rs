use serde_derive::Serialize;

#[derive(Serialize)]
enum E {
    #[serde("literal")]
    V,
}

fn main() {}
