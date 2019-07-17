use std::env;
use std::process::Command;
use std::str::{self, FromStr};

// The rustc-cfg strings below are *not* public API. Please let us know by
// opening a GitHub issue if your build environment requires some way to enable
// these cfgs other than by executing our build script.
fn main() {
    let minor = match rustc_minor_version() {
        Some(minor) => minor,
        None => return,
    };

    let target = env::var("TARGET").unwrap();
    let emscripten = target == "asmjs-unknown-emscripten" || target == "wasm32-unknown-emscripten";

    let has_atomic_integers = target_has_at_least_atomic_u64(&target);

    // std::collections::Bound was stabilized in Rust 1.17
    // but it was moved to core::ops later in Rust 1.26:
    // https://doc.rust-lang.org/core/ops/enum.Bound.html
    if minor >= 26 {
        println!("cargo:rustc-cfg=ops_bound");
    } else if minor >= 17 && cfg!(feature = "std") {
        println!("cargo:rustc-cfg=collections_bound");
    }

    // core::cmp::Reverse stabilized in Rust 1.19:
    // https://doc.rust-lang.org/stable/core/cmp/struct.Reverse.html
    if minor >= 19 {
        println!("cargo:rustc-cfg=core_reverse");
    }

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

    // Duration available in core since Rust 1.25:
    // https://blog.rust-lang.org/2018/03/29/Rust-1.25.html#library-stabilizations
    if minor >= 25 {
        println!("cargo:rustc-cfg=core_duration");
    }

    // 128-bit integers stabilized in Rust 1.26:
    // https://blog.rust-lang.org/2018/05/10/Rust-1.26.html
    //
    // Disabled on Emscripten targets as Emscripten doesn't
    // currently support integers larger than 64 bits.
    if minor >= 26 && !emscripten {
        println!("cargo:rustc-cfg=integer128");
    }

    // Inclusive ranges methods stabilized in Rust 1.27:
    // https://github.com/rust-lang/rust/pull/50758
    if minor >= 27 {
        println!("cargo:rustc-cfg=range_inclusive");
    }

    // Non-zero integers stabilized in Rust 1.28:
    // https://github.com/rust-lang/rust/pull/50808
    if minor >= 28 {
        println!("cargo:rustc-cfg=num_nonzero");
    }

    if minor >= 34 && has_atomic_integers {
        println!("cargo:rustc-cfg=std_integer_atomics");
    }
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = match env::var_os("RUSTC") {
        Some(rustc) => rustc,
        None => return None,
    };

    let output = match Command::new(rustc).arg("--version").output() {
        Ok(output) => output,
        Err(_) => return None,
    };

    let version = match str::from_utf8(&output.stdout) {
        Ok(version) => version,
        Err(_) => return None,
    };

    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }

    let next = match pieces.next() {
        Some(next) => next,
        None => return None,
    };

    u32::from_str(next).ok()
}

fn target_has_at_least_atomic_u64(target: &str) -> bool {
    // The cfg variable target_has_atomic is unstable
    // so this data comes from the  src/librustc_target/spec/*.rs
    // files in the rust source. Generally, it's 64-bit platforms
    // plus i686.
    if target.starts_with("x86_64") || target.starts_with("i686") ||
        target.starts_with("aarch64") || target.starts_with("powerpc64") ||
        target.starts_with("sparc64") || target.starts_with("mips64el") {
        true
    } else {
        false
    }
}
