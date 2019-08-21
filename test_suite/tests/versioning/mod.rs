#![allow(clippy::decimal_literal_representation, clippy::unreadable_literal)]
#![cfg_attr(feature = "unstable", feature(never_type))]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
use std::ffi::{CStr, CString, OsString};
use std::fmt::Debug;
use std::net;
use std::num::Wrapping;
use std::ops::Bound;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak as RcWeak};
use std::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI8, AtomicIsize, AtomicU16, AtomicU32, AtomicU8,
    AtomicUsize, Ordering,
};
use std::sync::{Arc, Weak as ArcWeak};
use std::time::{Duration, UNIX_EPOCH};

#[cfg(target_arch = "x86_64")]
use std::sync::atomic::{AtomicI64, AtomicU64};

use fnv::FnvHasher;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_test::{assert_de_tokens, assert_de_tokens_error, Configure, Token};

//////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
#[serde(rename = "A")]
struct Av1 {
    usize_value: usize
}

#[derive(Deserialize)]
#[serde(versions = {1: Av1, 2: A})]
struct A {
    bool_value: bool
}

#[derive(Deserialize)]
struct AMap {
    a: A
}

#[derive(Deserialize)]
struct ASeq {
    a: [A; 2]
}

//////////////////////////////////////////////////////////////////////////
