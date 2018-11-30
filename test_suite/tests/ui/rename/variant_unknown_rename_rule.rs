#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
enum S {
    #[serde(rename_all = "abc")]
    V {
        name: u8,
        long_name: u8,
        very_long_name: u8,
    },
}

fn main() {}
