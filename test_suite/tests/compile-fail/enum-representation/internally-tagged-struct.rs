#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
#[serde(tag = "type")] //~^ HELP: #[serde(tag = "...")] can only be used on enums
struct S;

fn main() {}
