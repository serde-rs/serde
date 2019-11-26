use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GenericTupleStruct<T, U>(T, U);
