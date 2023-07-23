use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/mod.rs");

    // Sometimes on Windows the git checkout does not correctly wire up the
    // symlink from serde_derive_internals/src to serde_derive/src/internals.
    // When this happens we'll just build based on relative paths within the git
    // repo.
    let mod_behind_symlink = Path::new("src/mod.rs");
    if !mod_behind_symlink.exists() {
        println!("cargo:rustc-cfg=serde_build_from_git");
    }
}
