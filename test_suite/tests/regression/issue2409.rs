#![allow(dead_code)]

use serde_derive::Deserialize;

macro_rules! bug {
    ($serde_path:literal) => {
        #[derive(Deserialize)]
        #[serde(crate = $serde_path)]
        pub struct Struct;
    };
}

bug!("serde");
