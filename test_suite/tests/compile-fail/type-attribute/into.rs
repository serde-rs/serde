#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
#[serde(into = "Option<T")] //~^ HELP: failed to parse type: into = "Option<T"
enum TestOne {
    Testing,
    One,
    Two,
    Three,
}
