#![feature(custom_attribute, custom_derive, plugin, test)]
#![plugin(serde_macros)]

extern crate test;
extern crate serde;

use serde::json;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Default {
    a1: i32,
    #[serde(default)]
    a2: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Rename {
    a1: i32,
    #[serde(rename="a3")]
    a2: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DirectionRename {
    a1: i32,
    #[serde(rename_serialize="a3", rename_deserialize="a4")]
    a2: i32,
}

#[test]
fn test_default() {
    let deserialized_value: Default = json::from_str(&"{\"a1\":1,\"a2\":2}").unwrap();
    assert_eq!(deserialized_value, Default { a1: 1, a2: 2 });

    let deserialized_value: Default = json::from_str(&"{\"a1\":1}").unwrap();
    assert_eq!(deserialized_value, Default { a1: 1, a2: 0 });
}

#[test]
fn test_rename() {
    let value = Rename { a1: 1, a2: 2 };
    let serialized_value = json::to_string(&value).unwrap();
    assert_eq!(serialized_value, "{\"a1\":1,\"a3\":2}");

    let deserialized_value: Rename = json::from_str(&serialized_value).unwrap();
    assert_eq!(value, deserialized_value);
}

#[test]
fn test_direction_rename() {
    let value = DirectionRename { a1: 1, a2: 2 };
    let serialized_value = json::to_string(&value).unwrap();
    assert_eq!(serialized_value, "{\"a1\":1,\"a3\":2}");

    let deserialized_value = json::from_str("{\"a1\":1,\"a4\":2}").unwrap();
    assert_eq!(value, deserialized_value);
}
