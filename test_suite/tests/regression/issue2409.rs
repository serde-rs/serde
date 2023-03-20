use serde::Deserialize;

macro_rules! bug {
    ($serde_path:literal) => {
        #[derive(Deserialize)]
        #[serde(crate = $serde_path)]
        pub struct Struct;
    };
}

bug!("serde");
