use serde_derive::Serialize;

#[derive(Serialize, PartialEq, Debug)]
struct TupleStruct(
    #[serde(skip_serializing_if_default)]
    u8,
    #[serde(skip_serializing_if_default)]
    String,
);

#[derive(Serialize, PartialEq, Debug)]
enum Enum {
    #[serde(skip_serializing_if_default)]
    TupleVariant(u8, String),
    #[serde(skip_serializing_if_default)]
    StructVariant {
        #[serde(skip_serializing_if_default)]
        field: u8,
        #[serde(skip_serializing_if_default)]
        other: String,
    },
}

fn main() {
    // Test tuple struct with default values
    let tuple_struct = TupleStruct(0, String::new());
    let serialized = serde_json::to_string(&tuple_struct).unwrap();
    assert_eq!(serialized, r#"[]"#);

    // Test tuple struct with non-default values
    let tuple_struct = TupleStruct(42, "hello".to_string());
    let serialized = serde_json::to_string(&tuple_struct).unwrap();
    assert_eq!(serialized, r#"[42,"hello"]"#);

    // Test tuple variant with default values
    let enum_val = Enum::TupleVariant(0, String::new());
    let serialized = serde_json::to_string(&enum_val).unwrap();
    assert_eq!(serialized, r#"{"TupleVariant":[]}"#);

    // Test tuple variant with non-default values
    let enum_val = Enum::TupleVariant(42, "hello".to_string());
    let serialized = serde_json::to_string(&enum_val).unwrap();
    assert_eq!(serialized, r#"{"TupleVariant":[42,"hello"]}"#);

    // Test struct variant with default values
    let enum_val = Enum::StructVariant { field: 0, other: String::new() };
    let serialized = serde_json::to_string(&enum_val).unwrap();
    assert_eq!(serialized, r#"{"StructVariant":{}}"#);

    // Test struct variant with non-default values
    let enum_val = Enum::StructVariant { field: 99, other: "world".to_string() };
    let serialized = serde_json::to_string(&enum_val).unwrap();
    assert_eq!(serialized, r#"{"StructVariant":{"field":99,"other":"world"}}"#);
}