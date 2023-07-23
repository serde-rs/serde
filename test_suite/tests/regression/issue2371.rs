use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Nested;

#[derive(Deserialize)]
pub enum ExternallyTagged {
    Flatten {
        #[serde(flatten)]
        nested: Nested,
        string: &'static str,
    },
}

#[derive(Deserialize)]
#[serde(tag = "tag")]
pub enum InternallyTagged {
    Flatten {
        #[serde(flatten)]
        nested: Nested,
        string: &'static str,
    },
}

#[derive(Deserialize)]
#[serde(tag = "tag", content = "content")]
pub enum AdjacentlyTagged {
    Flatten {
        #[serde(flatten)]
        nested: Nested,
        string: &'static str,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum UntaggedWorkaround {
    Flatten {
        #[serde(flatten)]
        nested: Nested,
        string: &'static str,
    },
}
