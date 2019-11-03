use std::env;
use std::path::PathBuf;

#[cfg(not(windows))]
const CARGO_EXPAND_BIN: &str = "cargo-expand";

#[cfg(windows)]
const CARGO_EXPAND_BIN: &str = "cargo-expand.exe";

/// Scans paths in PATH env variable for a presence of `CARGO_EXPAND_BIN` file.
fn is_cargo_expand_present() -> bool {
    if let Ok(var) = env::var("PATH") {
        for path in var.split(":").map(PathBuf::from) {
            let cargo_expand_path = path.join(CARGO_EXPAND_BIN);
            if cargo_expand_path.exists() {
                return true;
            }
        }
    }

    false
}

pub fn main() {
    if is_cargo_expand_present() {
        println!("cargo:rustc-cfg=cargo_expand");
    }
}
