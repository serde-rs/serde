#![feature(custom_derive, plugin, test, custom_attribute)]
#![plugin(serde_macros)]
//! Tests for YAML serialzation only
extern crate test;
extern crate serde;

#[test]
fn test_yaml_write_null() {
    let want = "null";
    assert!(false);
}
