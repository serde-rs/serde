use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum GenericEnum<T, U> {
    Unit,
    NewType(T),
    Seq(T, U),
    Map { x: T, y: U },
}
