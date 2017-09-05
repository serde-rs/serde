// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(feature = "unstable")]

extern crate compiletest_rs as compiletest;

use std::env;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();

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
