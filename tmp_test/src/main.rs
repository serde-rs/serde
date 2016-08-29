#![feature(rustc_macro)]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[derive(Serialize)]
enum Macros {
    #[serde(rename = "macros 1.1")]
    OnePointOne,
}

fn main() {
    let s = Macros::OnePointOne;
    println!("{}", serde_json::to_string(&s).unwrap());
}
