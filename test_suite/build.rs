use std::process::{Command, ExitStatus, Stdio};

fn has_cargo_expand() -> bool {
    let cargo_expand = if cfg!(windows) {
        "cargo-expand.exe"
    } else {
        "cargo-expand"
    };

    Command::new(cargo_expand)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .as_ref()
        .map(ExitStatus::success)
        .unwrap_or(false)
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(feature = "expandtest") && has_cargo_expand() {
        println!("cargo:rustc-cfg=expandtest");
    }
}
