#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;

#[derive(Serialize)]
#[serde(abc="xyz")] //~ unknown serde container attribute `abc = "xyz"`
struct Foo {
    x: u32,
}

#[derive(Deserialize)]
#[serde(abc="xyz")] //~ unknown serde container attribute `abc = "xyz"`
struct Foo {
    x: u32,
}

#[derive(Serialize)]
struct Foo {
    #[serde(abc="xyz")] //~ unknown serde field attribute `abc = "xyz"`
    x: u32,
}

#[derive(Deserialize)]
struct Foo {
    #[serde(abc="xyz")] //~ unknown serde field attribute `abc = "xyz"`
    x: u32,
}

fn main() { }
