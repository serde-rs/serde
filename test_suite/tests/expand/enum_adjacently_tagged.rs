use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "tag", content = "content")]
pub enum AdjacentlyTaggedEnum<T, U> {
    Unit,
    NewType(T),
    Seq(T, U),
    Map { x: T, y: U },
}
