#![allow(clippy::decimal_literal_representation, clippy::unreadable_literal)]
#![cfg_attr(feature = "unstable", feature(never_type))]

use serde::Deserialize;

//////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
#[serde(rename = "A")]
struct Av1 {
    usize_value: usize
}

#[derive(Deserialize)]
#[serde(versions(Av1))]
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
