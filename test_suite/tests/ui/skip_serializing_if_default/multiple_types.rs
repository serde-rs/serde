use serde_derive::Serialize;

#[derive(Serialize, PartialEq, Debug)]
struct Struct {
    #[serde(skip_serializing_if_default)]
    option_field: Option<u8>,
    #[serde(skip_serializing_if_default)]
    vec_field: Vec<u8>,
    #[serde(skip_serializing_if_default)]
    string_field: String,
    #[serde(skip_serializing_if_default)]
    bool_field: bool,
}

fn main() {
    // All default values should be omitted
    let value = Struct {
        option_field: None,
        vec_field: Vec::new(),
        string_field: String::new(),
        bool_field: false,
    };
    let serialized = serde_json::to_string(&value).unwrap();
    assert_eq!(serialized, r#"{}"#);

    // Non-default values should be included
    let value = Struct {
        option_field: Some(42),
        vec_field: vec![1, 2, 3],
        string_field: "hello".to_string(),
        bool_field: true,
    };
    let serialized = serde_json::to_string(&value).unwrap();
    assert_eq!(serialized, r#"{"option_field":42,"vec_field":[1,2,3],"string_field":"hello","bool_field":true}"#);
}