use std::env;
use std::process::Command;
use std::str::{self, FromStr};

fn main() {
    let rustc = match env::var_os("RUSTC") {
        Some(rustc) => rustc,
        None => return,
    };

    let output = match Command::new(rustc).arg("--version").output() {
        Ok(output) => output,
        Err(_) => return,
    };

    let version = match str::from_utf8(&output.stdout) {
        Ok(version) => version,
        Err(_) => return,
    };

    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return;
    }

    let next = match pieces.next() {
        Some(next) => next,
        None => return,
    };

    let minor = match u32::from_str(next) {
        Ok(minor) => minor,
        Err(_) => return,
    };

    // CString::into_boxed_c_str stabilized in Rust 1.20:
    // https://doc.rust-lang.org/std/ffi/struct.CString.html#method.into_boxed_c_str
    if minor >= 20 {
        println!("cargo:rustc-cfg=de_boxed_c_str");
    }

    // From<Box<T>> for Rc<T> / Arc<T> stabilized in Rust 1.21:
    // https://doc.rust-lang.org/std/rc/struct.Rc.html#impl-From<Box<T>>
    // https://doc.rust-lang.org/std/sync/struct.Arc.html#impl-From<Box<T>>
    if minor >= 21 {
        println!("cargo:rustc-cfg=de_rc_dst");
    }

    // 128-bit integers stabilized in Rust 1.26:
    // https://blog.rust-lang.org/2018/05/10/Rust-1.26.html
    if minor >= 26 {
        println!("cargo:rustc-cfg=integer128");
    }

    // Non-zero integers stabilized in Rust 1.28:
    // https://github.com/rust-lang/rust/pull/50808
    if minor >= 28 {
        println!("cargo:rustc-cfg=num_nonzero");
    }
}
