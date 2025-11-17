use serde_derive::{Deserialize, Serialize};

mod w {
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(_: D) -> Result<(), D::Error> {
        unimplemented!()
    }
    pub fn serialize<T, S: Serializer>(_: S) -> Result<S::Ok, S::Error> {
        unimplemented!()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(with = "w")]
struct W(u8);

#[derive(Serialize, Deserialize)]
#[serde(serialize_with = "w::serialize")]
struct S(u8);

#[derive(Serialize, Deserialize)]
#[serde(deserialize_with = "w::deserialize")]
struct D(u8);

fn main() {}
