#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: proc-macro derive panicked
#[serde(from = "Option<T")] //~^ HELP: failed to parse type: from = "Option<T"
enum TestOne {
    Testing,
    One,
    Two,
    Three,
}
