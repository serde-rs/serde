#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
#[serde(tag = "type")] //~^ HELP: #[serde(tag = "...")] cannot be used with tuple variants
enum E {
    Tuple(u8, u8),
}

fn main() {}
