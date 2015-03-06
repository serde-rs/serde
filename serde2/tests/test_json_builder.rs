extern crate serde2;

use std::collections::BTreeMap;

use serde2::json::value::Value;
use serde2::json::builder::{ArrayBuilder, ObjectBuilder};

#[test]
fn test_array_builder() {
    let value = ArrayBuilder::new().unwrap();
    assert_eq!(value, Value::Array(Vec::new()));

    let value = ArrayBuilder::new()
        .push(1)
        .push(2)
        .push(3)
        .unwrap();
    assert_eq!(value, Value::Array(vec!(Value::I64(1), Value::I64(2), Value::I64(3))));

    let value = ArrayBuilder::new()
        .push_array(|bld| bld.push(1).push(2).push(3))
        .unwrap();
    assert_eq!(value, Value::Array(vec!(Value::Array(vec!(Value::I64(1), Value::I64(2), Value::I64(3))))));

    let value = ArrayBuilder::new()
        .push_object(|bld|
            bld
                .insert("a".to_string(), 1)
                .insert("b".to_string(), 2))
        .unwrap();

    let mut map = BTreeMap::new();
    map.insert("a".to_string(), Value::I64(1));
    map.insert("b".to_string(), Value::I64(2));
    assert_eq!(value, Value::Array(vec!(Value::Object(map))));
}

#[test]
fn test_object_builder() {
    let value = ObjectBuilder::new().unwrap();
    assert_eq!(value, Value::Object(BTreeMap::new()));

    let value = ObjectBuilder::new()
        .insert("a".to_string(), 1)
        .insert("b".to_string(), 2)
        .unwrap();

    let mut map = BTreeMap::new();
    map.insert("a".to_string(), Value::I64(1));
    map.insert("b".to_string(), Value::I64(2));
    assert_eq!(value, Value::Object(map));
}
