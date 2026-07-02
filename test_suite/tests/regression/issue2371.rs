#![allow(dead_code)]

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Nested;

#[derive(Deserialize)]
pub enum ExternallyTagged {
    Flatten {
        #[serde(flatten)]
        #[allow(dead_code)]
        nested: Nested,
        #[allow(dead_code)]
        string: &'static str,
    },
}

#[derive(Deserialize)]
#[serde(tag = "tag")]
pub enum InternallyTagged {
    Flatten {
        #[serde(flatten)]
        #[allow(dead_code)]
        nested: Nested,
        #[allow(dead_code)]
        string: &'static str,
    },
}

#[derive(Deserialize)]
#[serde(tag = "tag", content = "content")]
pub enum AdjacentlyTagged {
    Flatten {
        #[serde(flatten)]
        #[allow(dead_code)]
        nested: Nested,
        #[allow(dead_code)]
        string: &'static str,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum UntaggedWorkaround {
    Flatten {
        #[serde(flatten)]
        #[allow(dead_code)]
        nested: Nested,
        #[allow(dead_code)]
        string: &'static str,
    },
}
