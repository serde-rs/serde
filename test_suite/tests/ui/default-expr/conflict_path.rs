use serde_derive::Deserialize;

#[derive(Deserialize)]
struct ConflictPath {
    #[serde(default = "u32::default", default_expr = 1)]
    value: u32,
}

fn main() {}
