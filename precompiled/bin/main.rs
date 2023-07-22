extern crate proc_macro2;

use proc_macro2::watt;
use proc_macro2::watt::buffer::InputBuffer;
use std::alloc::{GlobalAlloc, Layout, System};
use std::io::{self, Read, Write};
use std::sync::atomic::Ordering;

struct MonotonicAllocator;

#[global_allocator]
static ALLOCATOR: MonotonicAllocator = MonotonicAllocator;

unsafe impl GlobalAlloc for MonotonicAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Leak: this cuts 3% of code size from the precompiled macro binary.
        // There is no way that serde_derive would fill up all memory on the
        // host. When the subprocess exits, operating system will clean this up.
    }
}

fn main() {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf).unwrap();

    let mut buf = InputBuffer::new(&buf);
    let derive = match buf.read_u8() {
        0 => serde_derive::derive_serialize,
        1 => serde_derive::derive_deserialize,
        2 => {
            serde_derive::DESERIALIZE_IN_PLACE.store(true, Ordering::Relaxed);
            serde_derive::derive_deserialize
        }
        _ => unreachable!(),
    };

    let input = watt::load(&mut buf);
    let output = derive(input);
    let bytes = watt::linearize(output);
    io::stdout().write_all(&bytes).unwrap();
}
