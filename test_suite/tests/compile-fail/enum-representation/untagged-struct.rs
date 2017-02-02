#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: custom derive attribute panicked
#[serde(untagged)] //~^ HELP: #[serde(untagged)] can only be used on enums
struct S;

fn main() {}
