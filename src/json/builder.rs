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

use ser::{self, Serialize};
use json::value::{self, Value};

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

    pub fn push<T: ser::Serialize>(mut self, v: T) -> ArrayBuilder {
        self.array.push(value::to_value(&v));
        self
    }

    pub fn push_array<F>(mut self, f: F) -> ArrayBuilder where
        F: FnOnce(ArrayBuilder) -> ArrayBuilder
    {
        let builder = ArrayBuilder::new();
        self.array.push(f(builder).unwrap());
        self
    }

    pub fn push_object<F>(mut self, f: F) -> ArrayBuilder where
        F: FnOnce(ObjectBuilder) -> ObjectBuilder
    {
        let builder = ObjectBuilder::new();
        self.array.push(f(builder).unwrap());
        self
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

    pub fn insert<V: ser::Serialize>(mut self, k: String, v: V) -> ObjectBuilder {
        self.object.insert(k, value::to_value(&v));
        self
    }

    pub fn insert_array<F>(mut self, key: String, f: F) -> ObjectBuilder where
        F: FnOnce(ArrayBuilder) -> ArrayBuilder
    {
        let builder = ArrayBuilder::new();
        self.object.insert(key, f(builder).unwrap());
        self
    }

    pub fn insert_object<F>(mut self, key: String, f: F) -> ObjectBuilder where
        F: FnOnce(ObjectBuilder) -> ObjectBuilder
    {
        let builder = ObjectBuilder::new();
        self.object.insert(key, f(builder).unwrap());
        self
    }
}
