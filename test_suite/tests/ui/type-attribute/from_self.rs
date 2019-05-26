use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(from = "Point")]
struct Point {}

fn main() {
}
