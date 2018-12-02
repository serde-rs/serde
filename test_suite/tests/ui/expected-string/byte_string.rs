#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct S {
    #[serde(rename = b"byte string")]
    byte_string: (),
}

fn main() {}
