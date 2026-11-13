#![no_std]

extern crate alloc;

use alloc::{
    borrow::Cow,
    rc::{Rc, Weak},
};

fn assert_serialize<T: serde::Serialize>() {}
fn assert_deserialize<T: for<'de> serde::Deserialize<'de>>() {}

pub fn assert_non_atomic_rc_impls() {
    assert_serialize::<Rc<u8>>();
    assert_deserialize::<Rc<u8>>();
    assert_serialize::<Weak<u8>>();
    assert_deserialize::<Weak<u8>>();
    assert_serialize::<Cow<'static, str>>();
    assert_deserialize::<Cow<'static, str>>();
}
