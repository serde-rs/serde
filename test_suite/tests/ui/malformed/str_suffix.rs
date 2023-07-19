use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(bound = ""huh)]
pub struct Struct {
    #[serde(rename = ""what)]
    pub field: i32,
}

fn main() {}
