#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: proc-macro derive panicked
#[serde(default)] //~^ HELP: #[serde(default)] can only be used on structs
enum E {
    S { f: u8 },
}
