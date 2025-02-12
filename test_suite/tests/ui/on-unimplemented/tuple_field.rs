use serde_derive::{Deserialize, Serialize};

struct NoImpls;

#[derive(Serialize, Deserialize)]
struct S(u32, NoImpls);

fn main() {}
