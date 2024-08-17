#![allow(dead_code)] // we do not read enum fields

use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub enum A {
    B {
        c: String,
    },
    D {
        #[serde(flatten)]
        e: E,
    },
}

#[derive(Deserialize)]
pub struct E {}
