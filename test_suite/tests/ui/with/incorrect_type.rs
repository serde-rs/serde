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
struct W(#[serde(with = "w")] u8, u8);

#[derive(Serialize, Deserialize)]
struct S(#[serde(serialize_with = "w::serialize")] u8, u8);

#[derive(Serialize, Deserialize)]
struct D(#[serde(deserialize_with = "w::deserialize")] u8, u8);

fn main() {}
