use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(tag = "conflict")]
enum E {
    A {
        #[serde(alias = "conflict")]
        x: (),
    },
}

fn main() {}
