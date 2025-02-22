#![no_std]
#![no_main]

use core::ffi::c_int;

#[no_mangle]
extern "C" fn main(_argc: c_int, _argv: *const *const u8) -> c_int {
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        libc::abort();
    }
}

//////////////////////////////////////////////////////////////////////////////

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Unit;

#[derive(Serialize, Deserialize)]
pub struct Newtype(u8);

#[derive(Serialize, Deserialize)]
pub struct Tuple(u8, u8);

#[derive(Serialize, Deserialize)]
pub struct Struct {
    f: u8,
}

#[derive(Serialize, Deserialize)]
pub enum Enum {
    Unit,
    Newtype(u8),
    Tuple(u8, u8),
    Struct { f: u8 },
}
