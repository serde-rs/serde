#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#![deny(identity_op)]

// The derived implementation uses 0+1 to add up the number of fields
// serialized, which Clippy warns about. If the expansion info is registered
// correctly, the Clippy lint is not triggered.
#[derive(Serialize)]
struct A { b: u8 }

fn main() {}
