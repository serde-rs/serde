#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(serde_macros)]

#[derive(Serialize, Deserialize)]
struct Test<'a> {
    s: &'a str, //~ ERROR: Serde does not support deserializing fields of type &str
}

fn main() {}
