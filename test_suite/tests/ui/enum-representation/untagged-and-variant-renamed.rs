use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum E {
	#[serde(alias = "different-name")]
	A(u8),
	#[serde(rename = "different-name")]
	B(String),
}
fn main() {}
