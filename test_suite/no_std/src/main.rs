// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(lang_items, start)]
#![no_std]

extern crate libc;

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    0
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments,
                               _file: &'static str,
                               _line: u32) -> ! {
    unsafe {
        libc::abort()
    }
}

//////////////////////////////////////////////////////////////////////////////

#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
struct Unit;

#[derive(Serialize, Deserialize)]
struct Newtype(u8);

#[derive(Serialize, Deserialize)]
struct Tuple(u8, u8);

#[derive(Serialize, Deserialize)]
struct Struct { f: u8 }

#[derive(Serialize, Deserialize)]
enum Enum {
    Unit,
    Newtype(u8),
    Tuple(u8, u8),
    Struct { f: u8 },
}
