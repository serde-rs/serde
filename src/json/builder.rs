// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::BTreeMap;

use json::value::{ToJson, Value};

pub struct ArrayBuilder {
    array: Vec<Value>,
}

impl ArrayBuilder {
    pub fn new() -> ArrayBuilder {
        ArrayBuilder { array: Vec::new() }
    }

    pub fn unwrap(self) -> Value {
        Value::Array(self.array)
    }

    pub fn push<T: ToJson>(self, value: T) -> ArrayBuilder {
        let mut builder = self;
        builder.array.push(value.to_json());
        builder
    }

    pub fn push_array(self, f: |ArrayBuilder| -> ArrayBuilder) -> ArrayBuilder {
        let builder = ArrayBuilder::new();
        self.push(f(builder).unwrap())
    }

    pub fn push_object(self, f: |ObjectBuilder| -> ObjectBuilder) -> ArrayBuilder {
        let builder = ObjectBuilder::new();
        self.push(f(builder).unwrap())
    }
}

pub struct ObjectBuilder {
    object: BTreeMap<String, Value>,
}

impl ObjectBuilder {
    pub fn new() -> ObjectBuilder {
        ObjectBuilder { object: BTreeMap::new() }
    }

    pub fn unwrap(self) -> Value {
        Value::Object(self.object)
    }

    pub fn insert<V: ToJson>(self, key: String, value: V) -> ObjectBuilder {
        let mut builder = self;
        builder.object.insert(key, value.to_json());
        builder
    }

    pub fn insert_array(self, key: String, f: |ArrayBuilder| -> ArrayBuilder) -> ObjectBuilder {
        let builder = ArrayBuilder::new();
        self.insert(key, f(builder).unwrap())
    }

    pub fn insert_object(self, key: String, f: |ObjectBuilder| -> ObjectBuilder) -> ObjectBuilder {
        let builder = ObjectBuilder::new();
        self.insert(key, f(builder).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use json::value::Value;
    use super::{ArrayBuilder, ObjectBuilder};

    #[test]
    fn test_array_builder() {
        let value = ArrayBuilder::new().unwrap();
        assert_eq!(value, Value::Array(Vec::new()));

        let value = ArrayBuilder::new()
            .push(1i)
            .push(2i)
            .push(3i)
            .unwrap();
        assert_eq!(value, Value::Array(vec!(Value::Integer(1), Value::Integer(2), Value::Integer(3))));

        let value = ArrayBuilder::new()
            .push_array(|bld| bld.push(1i).push(2i).push(3i))
            .unwrap();
        assert_eq!(value, Value::Array(vec!(Value::Array(vec!(Value::Integer(1), Value::Integer(2), Value::Integer(3))))));

        let value = ArrayBuilder::new()
            .push_object(|bld|
                bld
                    .insert("a".to_string(), 1i)
                    .insert("b".to_string(), 2i))
            .unwrap();

        let mut map = BTreeMap::new();
        map.insert("a".to_string(), Value::Integer(1));
        map.insert("b".to_string(), Value::Integer(2));
        assert_eq!(value, Value::Array(vec!(Value::Object(map))));
    }

    #[test]
    fn test_object_builder() {
        let value = ObjectBuilder::new().unwrap();
        assert_eq!(value, Value::Object(BTreeMap::new()));

        let value = ObjectBuilder::new()
            .insert("a".to_string(), 1i)
            .insert("b".to_string(), 2i)
            .unwrap();

        let mut map = BTreeMap::new();
        map.insert("a".to_string(), Value::Integer(1));
        map.insert("b".to_string(), Value::Integer(2));
        assert_eq!(value, Value::Object(map));
    }
}
