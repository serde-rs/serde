use std::env;
use std::process::Command;
use std::str;

// The rustc-cfg strings below are *not* public API. Please let us know by
// opening a GitHub issue if your build environment requires some way to enable
// these cfgs other than by executing our build script.
fn main() {
    let minor = match rustc_minor_version() {
        Some(minor) => minor,
        None => return,
    };

    // Check ability to use #[track_caller]:
    // https://doc.rust-lang.org/reference/attributes/codegen.html#the-track_caller-attribute
    //
    // Perhaps sometime it will be possible to replace by built-in `version` attribute:
    // https://github.com/rust-lang/rust/issues/64796
    if minor >= 46 {
        println!("cargo:rustc-cfg=has_track_caller");
    }
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    pieces.next()?.parse().ok()
}
