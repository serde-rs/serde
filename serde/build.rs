use std::env;
use std::process::Command;
use std::str::{self, FromStr};

// The rustc-cfg strings below are *not* public API. Please let us know by
// opening a GitHub issue if your build environment requires some way to enable
// these cfgs other than by executing our build script.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let minor = match rustc_minor_version() {
        Some(minor) => minor,
        None => return,
    };

    if minor >= 77 {
        println!("cargo:rustc-check-cfg=cfg(no_core_cstr)");
        println!("cargo:rustc-check-cfg=cfg(no_core_num_saturating)");
        println!("cargo:rustc-check-cfg=cfg(no_core_try_from)");
        println!("cargo:rustc-check-cfg=cfg(no_float_copysign)");
        println!("cargo:rustc-check-cfg=cfg(no_num_nonzero_signed)");
        println!("cargo:rustc-check-cfg=cfg(no_relaxed_trait_bounds)");
        println!("cargo:rustc-check-cfg=cfg(no_serde_derive)");
        println!("cargo:rustc-check-cfg=cfg(no_std_atomic)");
        println!("cargo:rustc-check-cfg=cfg(no_std_atomic64)");
        println!("cargo:rustc-check-cfg=cfg(no_systemtime_checked_add)");
        println!("cargo:rustc-check-cfg=cfg(no_target_has_atomic)");
    }

    // TryFrom was stabilized in Rust 1.34:
    // https://blog.rust-lang.org/2019/04/11/Rust-1.34.0.html#tryfrom-and-tryinto
    if minor < 34 {
        println!("cargo:rustc-cfg=no_core_try_from");
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
