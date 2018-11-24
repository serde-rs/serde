#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
#[serde(into = "Option<T")]
enum TestOne {
    Testing,
    One,
    Two,
    Three,
}
