#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
#[serde(into="")]
enum TestOne {
    Testing,
    One,
    Two,
    Three,
}
