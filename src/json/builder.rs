// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::TreeMap;
use std::str::StrAllocating;

use json::value::{ToJson, Value};

pub struct ListBuilder {
    list: Vec<Value>,
}

impl ListBuilder {
    pub fn new() -> ListBuilder {
        ListBuilder { list: Vec::new() }
    }

    pub fn unwrap(self) -> Value {
        Value::List(self.list)
    }

    pub fn push<T: ToJson>(self, value: T) -> ListBuilder {
        let mut builder = self;
        builder.list.push(value.to_json());
        builder
    }

    pub fn push_list(self, f: |ListBuilder| -> ListBuilder) -> ListBuilder {
        let builder = ListBuilder::new();
        self.push(f(builder).unwrap())
    }

    pub fn push_object(self, f: |ObjectBuilder| -> ObjectBuilder) -> ListBuilder {
        let builder = ObjectBuilder::new();
        self.push(f(builder).unwrap())
    }
}

pub struct ObjectBuilder {
    object: TreeMap<String, Value>,
}

impl ObjectBuilder {
    pub fn new() -> ObjectBuilder {
        ObjectBuilder { object: TreeMap::new() }
    }

    pub fn unwrap(self) -> Value {
        Value::Object(self.object)
    }

    pub fn insert<K: StrAllocating, V: ToJson>(self, key: K, value: V) -> ObjectBuilder {
        let mut builder = self;
        builder.object.insert(key.into_string(), value.to_json());
        builder
    }

    pub fn insert_list<S: StrAllocating>(self, key: S, f: |ListBuilder| -> ListBuilder) -> ObjectBuilder {
        let builder = ListBuilder::new();
        self.insert(key.into_string(), f(builder).unwrap())
    }

    pub fn insert_object<S: StrAllocating>(self, key: S, f: |ObjectBuilder| -> ObjectBuilder) -> ObjectBuilder {
        let builder = ObjectBuilder::new();
        self.insert(key.into_string(), f(builder).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::TreeMap;

    use json::value::Value;
    use super::{ListBuilder, ObjectBuilder};

    #[test]
    fn test_list_builder() {
        let value = ListBuilder::new().unwrap();
        assert_eq!(value, Value::List(Vec::new()));

        let value = ListBuilder::new()
            .push(1i)
            .push(2i)
            .push(3i)
            .unwrap();
        assert_eq!(value, Value::List(vec!(Value::Integer(1), Value::Integer(2), Value::Integer(3))));

        let value = ListBuilder::new()
            .push_list(|bld| bld.push(1i).push(2i).push(3i))
            .unwrap();
        assert_eq!(value, Value::List(vec!(Value::List(vec!(Value::Integer(1), Value::Integer(2), Value::Integer(3))))));

        let value = ListBuilder::new()
            .push_object(|bld|
                bld
                    .insert("a".to_string(), 1i)
                    .insert("b".to_string(), 2i))
            .unwrap();

        let mut map = TreeMap::new();
        map.insert("a".to_string(), Value::Integer(1));
        map.insert("b".to_string(), Value::Integer(2));
        assert_eq!(value, Value::List(vec!(Value::Object(map))));
    }

    #[test]
    fn test_object_builder() {
        let value = ObjectBuilder::new().unwrap();
        assert_eq!(value, Value::Object(TreeMap::new()));

        let value = ObjectBuilder::new()
            .insert("a".to_string(), 1i)
            .insert("b".to_string(), 2i)
            .unwrap();

        let mut map = TreeMap::new();
        map.insert("a".to_string(), Value::Integer(1));
        map.insert("b".to_string(), Value::Integer(2));
        assert_eq!(value, Value::Object(map));
    }
}
