#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: custom derive attribute panicked
#[serde(tag = "type")] //~^ HELP: #[serde(tag = "...")] can only be used on enums
struct S;

fn main() {}
