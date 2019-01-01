#![cfg(feature = "compiletest")]

use compiletest_rs as compiletest;

#[test]
fn ui() {
    let config = compiletest::Config {
        mode: compiletest::common::Mode::Ui,
        src_base: std::path::PathBuf::from("tests/ui"),
        target_rustcflags: Some(String::from("\
            --edition=2018 \
            -L deps/target/debug/deps \
            -Z unstable-options \
            --extern serde_derive \
        ")),
        ..Default::default()
    };

    compiletest::run_tests(&config);
}
