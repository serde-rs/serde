fn main() {
    println!("cargo:rustc-cfg=precompiled");
    println!("cargo:rustc-cfg=feature=\"deserialize_in_place\"");
}
