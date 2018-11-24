#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(tag = "conflict")]
enum E {
    A {
        #[serde(rename = "conflict")]
        x: (),
    },
}

fn main() {}
