#![allow(dead_code)] // we do not read enum fields

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Nested;

#[derive(Deserialize)]
pub enum ExternallyTagged1 {
    Tuple(f64, String),
    Flatten {
        #[serde(flatten)]
        nested: Nested,
    },
}

#[derive(Deserialize)]
pub enum ExternallyTagged2 {
    Flatten {
        #[serde(flatten)]
        nested: Nested,
    },
    Tuple(f64, String),
}

// Internally tagged enums cannot contain tuple variants so not tested here

#[derive(Deserialize)]
#[serde(tag = "tag", content = "content")]
pub enum AdjacentlyTagged1 {
    Tuple(f64, String),
    Flatten {
        #[serde(flatten)]
        nested: Nested,
    },
}

#[derive(Deserialize)]
#[serde(tag = "tag", content = "content")]
pub enum AdjacentlyTagged2 {
    Flatten {
        #[serde(flatten)]
        nested: Nested,
    },
    Tuple(f64, String),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Untagged1 {
    Tuple(f64, String),
    Flatten {
        #[serde(flatten)]
        nested: Nested,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Untagged2 {
    Flatten {
        #[serde(flatten)]
        nested: Nested,
    },
    Tuple(f64, String),
}
