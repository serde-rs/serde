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
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        let mut len_le32 = [0u8; 4];
        if let Err(_) = stdin.read_exact(&mut len_le32) {
            break;
        }
        let len = u32::from_le_bytes(len_le32) as usize;
        let mut buf = Vec::with_capacity(len);
        buf.resize(len, 0);
        stdin.read_exact(&mut buf).unwrap();

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

        let size = (bytes.len() as u32).to_le_bytes();
        stdout.write_all(&size).unwrap();
        stdout.write_all(&bytes).unwrap();
        stdout.flush().unwrap();
    }
}
