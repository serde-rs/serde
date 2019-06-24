use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(tag = "conflict")]
enum E {
    A {
        #[serde(rename = "conflict")]
        x: (),
    },
}

fn main() {}
