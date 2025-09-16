#![deny(deprecated)]
#![allow(dead_code)]

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[deprecated]
enum DeprecatedEnum {
    A,
    B,
}

#[derive(Serialize, Deserialize)]
#[deprecated]
struct DeprecatedStruct {
    a: bool,
}

#[derive(Serialize, Deserialize)]
enum DeprecatedVariant {
    A,
    #[deprecated]
    B,
}

#[derive(Serialize, Deserialize)]
struct DeprecatedField {
    #[deprecated]
    a: bool,
}
