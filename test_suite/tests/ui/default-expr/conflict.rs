use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Conflict {
    #[serde(default, default_expr = 1)]
    value: u32,
}

fn main() {}
