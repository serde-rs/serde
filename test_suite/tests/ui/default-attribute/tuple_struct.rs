use serde_derive::Deserialize;

// No errors expected.
#[derive(Deserialize)]
struct T0(u8, u8);

// No errors expected:
// - If both fields are provided, both get value from data.
// - If only one field is provided, the second gets default value.
#[derive(Deserialize)]
struct T1(u8, #[serde(default)] u8);

// ERROR: The first field can get default value only if sequence is empty, but
// that mean that all other fields cannot be deserialized without errors.
#[derive(Deserialize)]
struct T2(#[serde(default)] u8, u8, u8);

// No errors expected:
// - If both fields are provided, both get value from data.
// - If only one field is provided, the second gets default value.
// - If no fields are provided, both get default value.
#[derive(Deserialize)]
struct T3(#[serde(default)] u8, #[serde(default)] u8);

////////////////////////////////////////////////////////////////////////////////

// No errors expected -- missing fields get default values.
#[derive(Deserialize, Default)]
#[serde(default)]
struct T4(u8, u8);

// No errors expected -- missing fields get default values.
#[derive(Deserialize, Default)]
#[serde(default)]
struct T5(#[serde(default)] u8, u8);

// No errors expected -- missing fields get default values.
#[derive(Deserialize, Default)]
#[serde(default)]
struct T6(u8, #[serde(default)] u8);

// No errors expected -- missing fields get default values.
#[derive(Deserialize, Default)]
#[serde(default)]
struct T7(#[serde(default)] u8, #[serde(default)] u8);

fn main() {}
