use serde_derive::{Deserialize, Serialize};

struct NoImpls;

#[derive(Serialize, Deserialize)]
enum E {
    S(u32, NoImpls)
}

fn main() {}
