use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "subtype")]
enum Child {
    Topic, Sidebar
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", seq_form="xx")]
enum ParentNoSeq {
    Title,
    #[serde(untagged)]
    SubStructure(Child),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", seq_form="true")]
enum E2 {
    Title,
    #[serde(untagged)]
    SubStructure(Child),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", seq_form=true)]
enum Ok1 {
    Title,
    #[serde(untagged)]
    SubStructure(Child),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", seq_form=false)]
enum Ok2 {
    Title,
    #[serde(untagged)]
    SubStructure(Child),
}

fn main() {}
