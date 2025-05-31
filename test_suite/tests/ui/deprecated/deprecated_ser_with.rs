#![deny(deprecated)]

use serde::Serializer;
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct Struct {
    #[serde(serialize_with = "deprecated_with")]
    pub field: i32,
}

#[deprecated]
fn deprecated_with<S>(_field: &i32, _serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    unimplemented!()
}

fn main() {}
