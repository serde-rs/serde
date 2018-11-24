// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(feature = "unstable")]

extern crate compiletest_rs as compiletest;

#[test]
fn compile_fail() {
    let config = compiletest::Config {
        mode: compiletest::common::Mode::CompileFail,
        src_base: std::path::PathBuf::from("tests/compile-fail"),
        target_rustcflags: Some("-L deps/target/debug/deps".to_owned()),
        ..Default::default()
    };

    compiletest::run_tests(&config);
}
