// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Rust JSON serialization library
// Copyright (c) 2011 Google Inc.

#![forbid(non_camel_case_types)]
#![allow(missing_docs)]

/*!
JSON parsing and serialization

# What is JSON?

JSON (JavaScript Object Notation) is a way to write data in Javascript.
Like XML it allows one to serialize structured data in a text format that can be read by humans
easily.
Its native compatibility with JavaScript and its simple syntax make it used widely.

Json data are serialized in a form of "key":"value".
Data types that can be serialized are JavaScript types :
boolean (`true` or `false`), number (`f64`), string, array, object, null.
An object is a series of string keys mapping to values, in `"key": value` format.
Arrays are enclosed in square brackets (&[ ... ]) and objects in curly brackets ({ ... }).
A simple JSON document serializing a person, his/her age, address and phone numbers could look like:

```ignore
{
    "FirstName": "John",
    "LastName": "Doe",
    "Age": 43,
    "Address": {
        "Street": "Downing Street 10",
        "City": "London",
        "Country": "Great Britain"
    },
    "PhoneNumbers": [
        "+44 1234567",
        "+44 2345678"
    ]
}
```

# Rust Type-based Serializing and Deserializing

Rust provides a mechanism for low boilerplate serializing and deserializing
of values to and from JSON via the serialization API.
To be able to serialize a piece of data, it must implement the `serde::Serialize` trait.
To be able to deserialize a piece of data, it must implement the `serde::Deserialize` trait.
The Rust compiler provides an annotation to automatically generate
the code for these traits: `#[derive_serialize]` and `#[derive_deserialize]`.

To serialize using `Serialize`:

```rust
#![feature(plugin)]
#![plugin(serde_macros)]

extern crate serde;

use std::io::WriteExt;
use serde::json;
use serde::Serialize;

#[derive_serialize]
pub struct TestStruct   {
    data_str: String,
}

fn main() {
    let to_serialize_object = TestStruct {
        data_str: "example of string to serialize".to_string()
    };

    let mut wr = Vec::new();
    {
        let mut serializer = json::Serializer::new(wr.by_ref());
        match to_serialize_object.serialize(&mut serializer) {
            Ok(()) => (),
            Err(e) => panic!("json serialization error: {:?}", e),
        }
    }
}
```

Two wrapper functions are provided to serialize a `Serialize` object
into a string (String) or buffer (~[u8]): `json::to_string(value)` and
`json::to_vec(value)`.

```rust
use serde::json;
let to_serialize_object = "example of string to serialize";
let serialized_str: String = json::to_string(&to_serialize_object).unwrap();
```

JSON API provide an enum `json::Value` and a trait `ToJson` to serialize
object.  The trait `ToJson` serialize object into a container `json::Value` and
the API provide writer to serialize them into a stream or a string ...

When using `ToJson` the `Serialize` trait implementation is not mandatory.

A basic `ToJson` example using a BTreeMap of attribute name / attribute value:


```rust
#![feature(plugin)]
#![plugin(serde_macros)]

extern crate serde;

use std::collections::BTreeMap;
use serde::json::{ToJson, Value};

pub struct MyStruct  {
    attr1: u8,
    attr2: String,
}

impl ToJson for MyStruct {
    fn to_json( &self ) -> Value {
        let mut d = BTreeMap::new();
        d.insert("attr1".to_string(), self.attr1.to_json());
        d.insert("attr2".to_string(), self.attr2.to_json());
        d.to_json()
    }
}

fn main() {
    let test = MyStruct {attr1: 1, attr2:"test".to_string()};
    let json: Value = test.to_json();
    let json_str: String = json.to_string();
}
```

Or you can use the helper type `ObjectBuilder`:

```rust
#![feature(plugin)]
#![plugin(serde_macros)]

extern crate serde;

use serde::json::{ObjectBuilder, ToJson, Value};

pub struct MyStruct  {
    attr1: u8,
    attr2: String,
}

impl ToJson for MyStruct {
    fn to_json( &self ) -> Value {
        ObjectBuilder::new()
            .insert("attr1".to_string(), &self.attr1)
            .insert("attr2".to_string(), &self.attr2)
            .unwrap()
    }
}

fn main() {
    let test = MyStruct {attr1: 1, attr2:"test".to_string()};
    let json: Value = test.to_json();
    let json_str: String = json.to_string();
}
```

To deserialize a JSON string using `Deserialize` trait:

```rust
#![feature(plugin)]
#![plugin(serde_macros)]

extern crate serde;

use serde::json;
use serde::Deserialize;

#[derive_deserialize]
pub struct MyStruct  {
     attr1: u8,
     attr2: String,
}

fn main() {
    let json_str_to_deserialize = "{ \"attr1\": 1, \"attr2\": \"toto\" }";
    let mut parser = json::Parser::new(json_str_to_deserialize.bytes());
    let deserialized_object: MyStruct = match Deserialize::deserialize(&mut parser) {
        Ok(v) => v,
        Err(e) => panic!("Decoding error: {:?}", e)
    };
}
```

# Examples of use

## Using Autoserialization

Create a struct called `TestStruct1` and serialize and deserialize it to and from JSON
using the serialization API, using the derived serialization code.

```rust
#![feature(plugin)]
#![plugin(serde_macros)]

extern crate serde;

use serde::json;

#[derive_serialize]
#[derive_deserialize]
pub struct TestStruct1  {
    data_int: u8,
    data_str: String,
    data_vector: Vec<u8>,
}

// To serialize use the `json::to_string` to serialize an object in a string.
// It calls the generated `Serialize` impl.
fn main() {
    let to_serialize_object = TestStruct1 {
        data_int: 1,
        data_str: "toto".to_string(),
        data_vector: vec![2,3,4,5]
    };
    let serialized_str: String = json::to_string(&to_serialize_object).unwrap();

    // To deserialize use the `json::from_str` function.

    let deserialized_object: TestStruct1 = match json::from_str(&serialized_str) {
        Ok(deserialized_object) => deserialized_object,
        Err(e) => panic!("json deserialization error: {:?}", e),
    };
}
```

## Using `ToJson`

This example use the ToJson impl to deserialize the JSON string.
Example of `ToJson` trait implementation for TestStruct1.

```rust
#![feature(plugin)]
#![plugin(serde_macros)]

extern crate serde;

use serde::json::ToJson;
use serde::json;
use serde::Deserialize;

#[derive_serialize]   // generate Serialize impl
#[derive_deserialize] // generate Deserialize impl
pub struct TestStruct1  {
    data_int: u8,
    data_str: String,
    data_vector: Vec<u8>,
}

impl ToJson for TestStruct1 {
    fn to_json( &self ) -> json::Value {
        json::builder::ObjectBuilder::new()
            .insert("data_int".to_string(), &self.data_int)
            .insert("data_str".to_string(), &self.data_str)
            .insert("data_vector".to_string(), &self.data_vector)
            .unwrap()
    }
}

fn main() {
    // Serialization using our impl of to_json

    let test: TestStruct1 = TestStruct1 {
        data_int: 1,
        data_str: "toto".to_string(),
        data_vector: vec![2,3,4,5],
    };
    let json: json::Value = test.to_json();
    let json_str: String = json.to_string();

    // Deserialize like before.

    let mut parser = json::Parser::new(json_str.bytes());
    let deserialized: TestStruct1 = Deserialize::deserialize(&mut parser).unwrap();
}
```

*/

pub use self::builder::{ArrayBuilder, ObjectBuilder};
pub use self::de::{
    Parser,
    from_str,
};
pub use self::error::{Error, ErrorCode};
pub use self::ser::{
    Serializer,
    PrettySerializer,
    to_writer,
    to_vec,
    to_string,
    to_pretty_writer,
    to_pretty_vec,
    to_pretty_string,
};
pub use self::value::{Value, ToJson, from_json};

pub mod builder;
pub mod de;
pub mod ser;
pub mod value;
pub mod error;
