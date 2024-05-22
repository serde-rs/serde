use std::path::Path;

fn main() {
    // Warning: build.rs is not published to crates.io.

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/mod.rs");

    println!("cargo:rustc-cfg=check_cfg");
    println!("cargo:rustc-check-cfg=cfg(check_cfg)");
    println!("cargo:rustc-check-cfg=cfg(exhaustive)");
    println!("cargo:rustc-check-cfg=cfg(serde_build_from_git)");
    println!("cargo:rustc-check-cfg=cfg(feature, values(\"deserialize_in_place\"))");

    // Sometimes on Windows the git checkout does not correctly wire up the
    // symlink from serde_derive_internals/src to serde_derive/src/internals.
    // When this happens we'll just build based on relative paths within the git
    // repo.
    let mod_behind_symlink = Path::new("src/mod.rs");
    if !mod_behind_symlink.exists() {
        println!("cargo:rustc-cfg=serde_build_from_git");
    }
}
