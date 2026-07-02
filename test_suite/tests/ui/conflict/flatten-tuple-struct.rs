use serde_derive::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct Foo(u32, #[serde(flatten)] HashMap<String, String>);

fn main() {}
