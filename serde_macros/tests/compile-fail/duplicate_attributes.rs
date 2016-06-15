#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(serde_macros)]

#[derive(Serialize)]
struct S {
    #[serde(rename(serialize="x"))]
    #[serde(rename(serialize="y"))] //~ ERROR: duplicate serde attribute `rename`
    a: (),

    #[serde(rename(serialize="x"))]
    #[serde(rename="y")] //~ ERROR: duplicate serde attribute `rename`
    b: (),

    #[serde(rename(serialize="x"))]
    #[serde(rename(deserialize="y"))] // ok
    c: (),

    #[serde(rename="x")]
    #[serde(rename(deserialize="y"))] //~ ERROR: duplicate serde attribute `rename`
    d: (),
}

fn main() {}
