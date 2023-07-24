use serde_derive::Deserialize;

fn d<T>() -> T {
    unimplemented!()
}

/// No errors expected:
/// - if both fields are provided, both gets value from data
/// - if only one field is provided, the second gets default value
#[derive(Deserialize)]
struct T1(u8, #[serde(default = "d")] u8);

/// Errors expected -- the first field can get default value only if sequence is
/// empty, but that mean that all other fields cannot be deserialized without
/// errors, so the `#[serde(default)]` attribute is superfluous
#[derive(Deserialize)]
struct T2(#[serde(default = "d")] u8, u8, u8);

/// No errors expected:
/// - if both fields are provided, both gets value from data
/// - if only one field is provided, the second gets default value
/// - if none fields are provided, both gets default value
#[derive(Deserialize)]
struct T3(#[serde(default = "d")] u8, #[serde(default = "d")] u8);

////////////////////////////////////////////////////////////////////////////////

/// No errors expected -- missing fields gets default values
#[derive(Deserialize, Default)]
#[serde(default)]
struct T1D(#[serde(default = "d")] u8, u8);

/// No errors expected -- missing fields gets default values
#[derive(Deserialize, Default)]
#[serde(default)]
struct T2D(u8, #[serde(default = "d")] u8);

/// No errors expected -- missing fields gets default values
#[derive(Deserialize, Default)]
#[serde(default)]
struct T3D(#[serde(default = "d")] u8, #[serde(default = "d")] u8);

////////////////////////////////////////////////////////////////////////////////

/// No errors expected -- missing fields gets default values
#[derive(Deserialize)]
#[serde(default = "d")]
struct T1Path(#[serde(default)] u8, u8);

/// No errors expected -- missing fields gets default values
#[derive(Deserialize)]
#[serde(default = "d")]
struct T2Path(u8, #[serde(default)] u8);

/// No errors expected -- missing fields gets default values
#[derive(Deserialize)]
#[serde(default = "d")]
struct T3Path(#[serde(default)] u8, #[serde(default)] u8);

////////////////////////////////////////////////////////////////////////////////

/// No errors expected -- missing fields gets default values
#[derive(Deserialize)]
#[serde(default = "d")]
struct T1PathD(#[serde(default = "d")] u8, u8);

/// No errors expected -- missing fields gets default values
#[derive(Deserialize)]
#[serde(default = "d")]
struct T2PathD(u8, #[serde(default = "d")] u8);

/// No errors expected -- missing fields gets default values
#[derive(Deserialize)]
#[serde(default = "d")]
struct T3PathD(#[serde(default = "d")] u8, #[serde(default = "d")] u8);

fn main() {}
