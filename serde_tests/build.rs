extern crate syntex;
extern crate serde_codegen;

use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    for &(src, dst) in &[
        ("tests/test.rs.in", "test.rs"),
        ("benches/bench.rs.in", "bench.rs"),
    ] {
        let src = Path::new(src);
        let dst = Path::new(&out_dir).join(dst);

        let mut registry = syntex::Registry::new();

        serde_codegen::register(&mut registry);
        registry.expand("", &src, &dst).unwrap();
    }
}
