use serde_derive::{Deserialize, Serialize};

struct NoImpls;

#[derive(Serialize, Deserialize)]
enum E {
    S { x1: u32, x2: NoImpls },
}

fn main() {}
