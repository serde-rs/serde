#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(default)]
enum E {
    S { f: u8 },
}
