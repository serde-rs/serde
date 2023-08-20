fn is_target_precompiled(target: &str) -> bool {
    target == "x86_64-unknown-linux-gnu"
}

fn main() {
    // Alternatively use platforms crate etc. for Types
    let target = match std::env::var("TARGET") {
        Ok(target) => target,
        _ => "".to_string(), // TARGET should be required ?
    };
    println!("cargo:warning=\"{target}\"");
    // This is required to simplify gating
    let derive_build = match std::env::var("CARGO_CFG_SERDE_DERIVE_BUILD").as_deref() {
        Ok("source") => "source",
        _ => match is_target_precompiled(&target) {
            true => "precompiled",
            false => "source",
        },
    };
    println!("cargo:rustc-cfg=serde_derive_build=\"{derive_build}\"");
}
