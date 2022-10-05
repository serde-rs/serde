use serde::Deserialize;

#[derive(Deserialize)]
struct Nested {}

// Regression test for https://github.com/serde-rs/serde/issues/1904
#[derive(Deserialize)]
enum WithFlatten1 {
    Tuple(f64, String),
    Flatten {
        #[serde(flatten)]
        nested: Nested,
    },
}

#[derive(Deserialize)]
enum WithFlatten2 {
    Flatten {
        #[serde(flatten)]
        nested: Nested,
    },
    Tuple(f64, String),
}
