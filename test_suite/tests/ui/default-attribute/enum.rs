use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
enum E {
    // No errors expected.
    T0(u8, u8),

    // No errors expected:
    // - If both fields are provided, both get value from data.
    // - If only one field is provided, the second gets default value.
    T1(u8, #[serde(default)] u8),

    // ERROR: The first field can get default value only if sequence is empty, but
    // that mean that all other fields cannot be deserialized without errors.
    T2(#[serde(default)] u8, u8, u8),

    // No errors expected:
    // - If both fields are provided, both get value from data.
    // - If only one field is provided, the second gets default value.
    // - If no fields are provided, both get default value.
    T3(#[serde(default)] u8, #[serde(default)] u8),

    S { f: u8 },
}

fn main() {}
