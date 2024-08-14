use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum E {
	#[serde(alias = "foo")]
	A(u8),
	#[serde(rename = "bar")]
	B(String),
}
fn main() {}
