#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: proc-macro derive panicked
#[serde(default)] //~^ HELP: #[serde(default)] can only be used on structs
struct T(u8, u8);

fn main() { }
