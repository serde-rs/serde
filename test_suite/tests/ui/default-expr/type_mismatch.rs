// Tests that a type mismatch in default_expr is caught by Rust (no implicit coercion).
// "abc" is &'static str, not String — Serde must not convert it automatically.

use serde_derive::Deserialize;

#[derive(Deserialize)]
struct TypeMismatch {
    #[serde(default_expr = "abc")]
    host: String,
}

fn main() {}
