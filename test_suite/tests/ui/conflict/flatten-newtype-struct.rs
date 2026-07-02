use serde_derive::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct Foo(#[serde(flatten)] HashMap<String, String>);

fn main() {}
