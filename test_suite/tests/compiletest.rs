#![cfg(feature = "unstable-testing")]

extern crate compiletest_rs as compiletest;

use std::env;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::default_config();

    config.mode = mode.parse().expect("invalid mode");
    config.target_rustcflags = Some("-L deps/target/debug/deps".to_owned());
    if let Ok(name) = env::var("TESTNAME") {
        config.filter = Some(name);
    }
    config.src_base = format!("tests/{}", mode).into();

    compiletest::run_tests(&config);
}

#[test]
fn compile_fail() {
    run_mode("compile-fail");
}

#[test]
fn run_pass() {
    run_mode("run-pass");
}
