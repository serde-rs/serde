#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct Foo(u32, #[serde(flatten)] HashMap<String, String>);

fn main() {}
