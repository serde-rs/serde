use serde_derive::Serialize;

#[derive(Serialize, PartialEq, Debug)]
struct Struct {
    #[serde(skip_serializing_if_default)]
    field: u8,
}

fn main() {
    let value = Struct { field: 0 };
    let serialized = serde_json::to_string(&value).unwrap();
    assert_eq!(serialized, r#"{}"#);

    let value = Struct { field: 42 };
    let serialized = serde_json::to_string(&value).unwrap();
    assert_eq!(serialized, r#"{"field":42}"#);
}