#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: proc-macro derive panicked
#[serde(from="")]
enum TestOne {
    Testing,
    One,
    Two,
    Three,
}
