#![allow(dead_code)]

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", seq_form=false)]
enum ParentNoSeq {
    Title,
    #[serde(untagged)]
    SubStructure(Child),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", seq_form=false)]
enum ParentChildNoSeq {
    Title,
    #[serde(untagged)]
    SubStructure(ChildNoSeq),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "subtype", seq_form=false)]
enum ChildNoSeq {
    Topic, Sidebar
}

#[derive(Debug, Deserialize)]
#[serde(tag = "subtype")]
enum Child {
    Topic, Sidebar
}
