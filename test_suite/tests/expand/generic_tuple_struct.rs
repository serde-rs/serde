use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct GenericTupleStruct<T, U>(T, U);
