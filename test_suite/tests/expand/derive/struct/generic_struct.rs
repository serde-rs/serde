use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GenericStruct<T> {
    x: T,
}

#[derive(Serialize, Deserialize)]
pub struct GenericNewTypeStruct<T>(T);
