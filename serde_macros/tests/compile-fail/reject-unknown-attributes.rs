#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;

#[derive(Serialize)] //~ unknown serde container attribute `abc`
#[serde(abc="xyz")]
struct Foo {
    x: u32,
}

#[derive(Deserialize)] //~ unknown serde container attribute `abc`
#[serde(abc="xyz")]
struct Foo {
    x: u32,
}

#[derive(Serialize)] //~ unknown serde field attribute `abc`
struct Foo {
    #[serde(abc="xyz")]
    x: u32,
}

#[derive(Deserialize)] //~ unknown serde field attribute `abc`
struct Foo {
    #[serde(abc="xyz")]
    x: u32,
}

fn main() { }
