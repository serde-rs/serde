#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct Foo(#[serde(flatten)] HashMap<String, String>);

fn main() {}
