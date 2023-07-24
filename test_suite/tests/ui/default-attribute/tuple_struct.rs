use serde_derive::Deserialize;

/// No errors expected
#[derive(Deserialize)]
struct T0(u8, u8);

/// No errors expected:
/// - if both fields are provided, both gets value from data
/// - if only one field is provided, the second gets default value
#[derive(Deserialize)]
struct T1(u8, #[serde(default)] u8);

/// Errors expected -- the first field can get default value only if sequence is
/// empty, but that mean that all other fields cannot be deserialized without
/// errors, so the `#[serde(default)]` attribute is superfluous
#[derive(Deserialize)]
struct T2(#[serde(default)] u8, u8, u8);

/// No errors expected:
/// - if both fields are provided, both gets value from data
/// - if only one field is provided, the second gets default value
/// - if none fields are provided, both gets default value
#[derive(Deserialize)]
struct T3(#[serde(default)] u8, #[serde(default)] u8);

////////////////////////////////////////////////////////////////////////////////

/// No errors expected -- missing fields gets default values
#[derive(Deserialize, Default)]
#[serde(default)]
struct T4(u8, u8);

/// No errors expected -- missing fields gets default values
#[derive(Deserialize, Default)]
#[serde(default)]
struct T5(#[serde(default)] u8, u8);

/// No errors expected -- missing fields gets default values
#[derive(Deserialize, Default)]
#[serde(default)]
struct T6(u8, #[serde(default)] u8);

/// No errors expected -- missing fields gets default values
#[derive(Deserialize, Default)]
#[serde(default)]
struct T7(#[serde(default)] u8, #[serde(default)] u8);

fn main() {}
