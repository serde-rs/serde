extern crate proc_macro2;

use proc_macro2::watt;
use proc_macro2::watt::buffer::InputBuffer;
use std::io::{self, Read, Write};

fn main() {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf).unwrap();

    let mut buf = InputBuffer::new(&buf);
    let derive = match buf.read_u8() {
        0 => serde_derive::derive_serialize,
        1 => serde_derive::derive_deserialize,
        _ => unreachable!(),
    };

    let input = watt::load(&mut buf);
    let output = derive(input);
    let bytes = watt::linearize(output);
    io::stdout().write_all(&bytes).unwrap();
}
