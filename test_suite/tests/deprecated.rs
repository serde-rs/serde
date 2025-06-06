#![deny(deprecated)]
#![allow(dead_code)]

use serde_derive::{Deserialize, Serialize};

/// deprecated enum
#[derive(Serialize, Deserialize)]
#[deprecated]
enum E1 {
    A,
    B,
}

/// deprecated struct
#[derive(Serialize, Deserialize)]
#[deprecated]
struct S1 {
    a: bool,
}

/// deprecated enum variant
#[derive(Serialize, Deserialize)]
enum E2 {
    A,
    #[deprecated]
    B,
}

/// deprecated struct field
#[derive(Serialize, Deserialize)]
struct S2 {
    #[deprecated]
    a: bool,
}
