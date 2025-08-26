#![allow(clippy::trivially_copy_pass_by_ref, dead_code)]

use serde_derive::Deserialize;

macro_rules! declare_in_macro {
    ($with:literal) => {
        #[derive(Deserialize)]
        pub struct S(
            #[serde(with = $with)]
            #[allow(dead_code)]
            i32,
        );
    };
}

declare_in_macro!("with");

mod with {
    use serde::Deserializer;

    pub fn deserialize<'de, D>(_: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}
