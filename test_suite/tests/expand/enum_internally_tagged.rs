use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "tag")]
pub enum InternallyTaggedEnum<T, U> {
    Unit,
    NewType(T),
    Map { x: T, y: U },
}
