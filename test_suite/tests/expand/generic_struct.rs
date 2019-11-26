use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GenericStruct<T> {
    x: T,
}

#[derive(Serialize, Deserialize)]
pub struct GenericNewTypeStruct<T>(T);
