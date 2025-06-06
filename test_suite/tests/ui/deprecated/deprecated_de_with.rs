#![deny(deprecated)]

use serde::Deserializer;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Struct {
    #[serde(deserialize_with = "deprecated_with")]
    pub field: i32,
}

#[deprecated]
fn deprecated_with<'de, D>(_deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    unimplemented!()
}

fn main() {}
