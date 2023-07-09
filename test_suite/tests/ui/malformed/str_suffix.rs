use serde::Serialize;

#[derive(Serialize)]
#[serde(bound = ""huh)]
pub struct Struct {
    #[serde(rename = ""what)]
    pub field: i32,
}

fn main() {}
