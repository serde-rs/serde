#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;

#[derive(Serialize)]
struct Foo(u32, #[serde(flatten)] HashMap<String, String>);

fn main() {}
