use serde_derive::{Deserialize, Serialize};

struct NoImpls;

#[derive(Serialize, Deserialize)]
struct S {
    x1: u32,
    x2: NoImpls,
}

fn main() {}
