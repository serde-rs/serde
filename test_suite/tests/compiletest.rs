#![cfg(feature = "compiletest")]

extern crate compiletest_rs as compiletest;

#[test]
fn ui() {
    let config = compiletest::Config {
        mode: compiletest::common::Mode::Ui,
        src_base: std::path::PathBuf::from("tests/ui"),
        target_rustcflags: Some("-L deps/target/debug/deps".to_owned()),
        ..Default::default()
    };

    compiletest::run_tests(&config);
}
