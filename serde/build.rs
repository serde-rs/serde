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

    // 128-bit integers stabilized in Rust 1.26:
    // https://blog.rust-lang.org/2018/05/10/Rust-1.26.html
    if minor >= 26 {
        println!("cargo:rustc-cfg=integer128");
    }
}
