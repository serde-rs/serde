use serde_derive::Serialize;

#[derive(Serialize)]
struct S {
    #[serde(rename = b"byte string")]
    byte_string: (),
}

fn main() {}
