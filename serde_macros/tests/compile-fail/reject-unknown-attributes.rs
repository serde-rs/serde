#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;

#[derive(Serialize)]
#[serde(abc="xyz")] //~ unknown serde container attribute `abc = "xyz"`
struct A {
    x: u32,
}

#[derive(Deserialize)]
#[serde(abc="xyz")] //~ unknown serde container attribute `abc = "xyz"`
struct B {
    x: u32,
}

#[derive(Serialize)]
struct C {
    #[serde(abc="xyz")] //~ unknown serde field attribute `abc = "xyz"`
    x: u32,
}

#[derive(Deserialize)]
struct D {
    #[serde(abc="xyz")] //~ unknown serde field attribute `abc = "xyz"`
    x: u32,
}

fn main() { }
