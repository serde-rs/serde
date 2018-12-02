#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(default = "default_e")]
enum E {
    S { f: u8 },
}
