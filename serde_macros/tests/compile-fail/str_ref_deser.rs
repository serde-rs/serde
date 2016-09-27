#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(serde_macros)]

#[derive(Serialize, Deserialize)] //~ ERROR: Serde does not support deserializing fields of type &str
struct Test<'a> {
    s: &'a str,
}

fn main() {}
