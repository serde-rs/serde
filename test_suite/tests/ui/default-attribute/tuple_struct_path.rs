use serde_derive::Deserialize;

fn d<T>() -> T {
    unimplemented!()
}

// No errors expected:
// - If both fields are provided, both get value from data.
// - If only one field is provided, the second gets default value.
#[derive(Deserialize)]
struct T1(u8, #[serde(default = "d")] u8);

// ERROR: The first field can get default value only if sequence is empty, but
// that means that all other fields cannot be deserialized without errors.
#[derive(Deserialize)]
struct T2(#[serde(default = "d")] u8, u8, u8);

// No errors expected:
// - If both fields are provided, both get value from data.
// - If only one field is provided, the second gets default value.
// - If no fields are provided, both get default value.
#[derive(Deserialize)]
struct T3(#[serde(default = "d")] u8, #[serde(default = "d")] u8);

////////////////////////////////////////////////////////////////////////////////

// No errors expected -- missing fields get default values.
#[derive(Deserialize, Default)]
#[serde(default)]
struct T1D(#[serde(default = "d")] u8, u8);

// No errors expected -- missing fields get default values.
#[derive(Deserialize, Default)]
#[serde(default)]
struct T2D(u8, #[serde(default = "d")] u8);

// No errors expected -- missing fields get default values.
#[derive(Deserialize, Default)]
#[serde(default)]
struct T3D(#[serde(default = "d")] u8, #[serde(default = "d")] u8);

////////////////////////////////////////////////////////////////////////////////

// No errors expected -- missing fields get default values.
#[derive(Deserialize)]
#[serde(default = "d")]
struct T1Path(#[serde(default)] u8, u8);

// No errors expected -- missing fields get default values.
#[derive(Deserialize)]
#[serde(default = "d")]
struct T2Path(u8, #[serde(default)] u8);

// No errors expected -- missing fields get default values.
#[derive(Deserialize)]
#[serde(default = "d")]
struct T3Path(#[serde(default)] u8, #[serde(default)] u8);

////////////////////////////////////////////////////////////////////////////////

// No errors expected -- missing fields get default values.
#[derive(Deserialize)]
#[serde(default = "d")]
struct T1PathD(#[serde(default = "d")] u8, u8);

// No errors expected -- missing fields get default values.
#[derive(Deserialize)]
#[serde(default = "d")]
struct T2PathD(u8, #[serde(default = "d")] u8);

// No errors expected -- missing fields get default values.
#[derive(Deserialize)]
#[serde(default = "d")]
struct T3PathD(#[serde(default = "d")] u8, #[serde(default = "d")] u8);

fn main() {}
