#![allow(clippy::trivially_copy_pass_by_ref, dead_code)]

use serde_derive::{Deserialize, Serialize};

macro_rules! declare_in_macro {
    ($with:literal) => {
        #[derive(Serialize, Deserialize)]
        pub struct S {
            #[serde(with = $with)]
            f: i32,
        }
    };
}

declare_in_macro!("with");

mod with {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(_: &i32, _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unimplemented!()
    }

    pub fn deserialize<'de, D>(_: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}
