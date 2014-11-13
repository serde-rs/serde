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
Arrays are enclosed in square brackets ([ ... ]) and objects in curly brackets ({ ... }).
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
the code for these traits: `#[deriving_serialize]` and `#[deriving_deserialize]`.

To serialize using `Serialize`:

```rust
#![feature(phase)]
#[phase(plugin)]
extern crate serde_macros;
extern crate serde;

use std::io::{MemWriter, AsRefWriter};
use serde::json;
use serde::Serialize;

#[deriving_serialize]
pub struct TestStruct   {
    data_str: String,
}

fn main() {
    let to_serialize_object = TestStruct {
        data_str: "example of string to serialize".to_string()
    };

    let mut m = MemWriter::new();
    {
        let mut serializer = json::Serializer::new(m.by_ref());
        match to_serialize_object.serialize(&mut serializer) {
            Ok(()) => (),
            Err(e) => panic!("json serialization error: {}", e),
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

JSON API provide an enum `json::Json` and a trait `ToJson` to serialize object.
The trait `ToJson` serialize object into a container `json::Json` and the API provide writer
to serialize them into a stream or a string ...

When using `ToJson` the `Serialize` trait implementation is not mandatory.

A basic `ToJson` example using a TreeMap of attribute name / attribute value:


```rust
#![feature(phase)]
#[phase(plugin)]
extern crate serde_macros;
extern crate serde;

use std::collections::TreeMap;
use serde::json;
use serde::json::ToJson;

pub struct MyStruct  {
    attr1: u8,
    attr2: String,
}

impl ToJson for MyStruct {
    fn to_json( &self ) -> json::Json {
        let mut d = TreeMap::new();
        d.insert("attr1".to_string(), self.attr1.to_json());
        d.insert("attr2".to_string(), self.attr2.to_json());
        json::Object(d)
    }
}

fn main() {
    let test2: MyStruct = MyStruct {attr1: 1, attr2:"test".to_string()};
    let tjson: json::Json = test2.to_json();
    let json_str: String = tjson.to_string();
}
```

To deserialize a JSON string using `Deserialize` trait :

```rust
#![feature(phase)]
#[phase(plugin)]
extern crate serde_macros;
extern crate serde;

use serde::json;
use serde::Deserialize;

#[deriving_deserialize]
pub struct MyStruct  {
     attr1: u8,
     attr2: String,
}

fn main() {
    let json_str_to_deserialize = "{ \"attr1\": 1, \"attr2\": \"toto\" }";
    let mut parser = json::Parser::new(json_str_to_deserialize.bytes());
    let deserialized_object: MyStruct = match Deserialize::deserialize(&mut parser) {
        Ok(v) => v,
        Err(e) => panic!("Decoding error: {}", e)
    };
}
```

# Examples of use

## Using Autoserialization

Create a struct called `TestStruct1` and serialize and deserialize it to and from JSON
using the serialization API, using the derived serialization code.

```rust
#![feature(phase)]
#[phase(plugin)]
extern crate serde_macros;
extern crate serde;

use serde::json;

#[deriving_serialize]
#[deriving_deserialize]
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

    let deserialized_object: TestStruct1 = match json::from_str(serialized_str.as_slice()) {
        Ok(deserialized_object) => deserialized_object,
        Err(e) => panic!("json deserialization error: {}", e),
    };
}
```

## Using `ToJson`

This example use the ToJson impl to deserialize the JSON string.
Example of `ToJson` trait implementation for TestStruct1.

```rust
#![feature(phase)]
#[phase(plugin)]
extern crate serde_macros;
extern crate serde;

use std::collections::TreeMap;
use serde::json::ToJson;
use serde::json;
use serde::Deserialize;

#[deriving_serialize]   // generate Serialize impl
#[deriving_deserialize] // generate Deserialize impl
pub struct TestStruct1  {
    data_int: u8,
    data_str: String,
    data_vector: Vec<u8>,
}

impl ToJson for TestStruct1 {
    fn to_json( &self ) -> json::Json {
        let mut d = TreeMap::new();
        d.insert("data_int".to_string(), self.data_int.to_json());
        d.insert("data_str".to_string(), self.data_str.to_json());
        d.insert("data_vector".to_string(), self.data_vector.to_json());
        json::Object(d)
    }
}

fn main() {
    // Serialization using our impl of to_json

    let test2: TestStruct1 = TestStruct1 {
        data_int: 1,
        data_str: "toto".to_string(),
        data_vector: vec![2,3,4,5],
    };
    let tjson: json::Json = test2.to_json();
    let json_str: String = tjson.to_string().into_string();

    // Deserialize like before.

    let mut parser = json::Parser::new(json_str.as_slice().bytes());
    let deserialized: TestStruct1 = Deserialize::deserialize(&mut parser).unwrap();
}
```

*/

use std::char;
use std::collections::{HashMap, TreeMap, tree_map};
use std::f32;
use std::f64;
use std::fmt;
use std::io::{IoResult, MemWriter, AsRefWriter};
use std::io;
use std::num::{FPNaN, FPInfinite};
use std::num;
use std::str::ScalarValue;
use std::str;
use std::string;
use std::vec::Vec;
use std::vec;

use de;
use ser::Serialize;
use ser;

pub mod builder;

/// Represents a json value
#[deriving(Clone, PartialEq, PartialOrd)]
pub enum Json {
    Null,
    Boolean(bool),
    Integer(i64),
    Floating(f64),
    String(string::String),
    List(JsonList),
    Object(JsonObject),
}

pub type JsonList = Vec<Json>;
pub type JsonObject = TreeMap<string::String, Json>;

impl Json {
    /// Serializes a json value into an io::writer.  Uses a single line.
    pub fn to_writer<W: Writer>(&self, wr: W) -> SerializeResult {
        let mut serializer = Serializer::new(wr);
        self.serialize(&mut serializer)
    }

    /// Serializes a json value into an io::writer.
    /// Pretty-prints in a more readable format.
    pub fn to_pretty_writer<W: Writer>(&self, wr: W) -> SerializeResult {
        let mut serializer = PrettySerializer::new(wr);
        self.serialize(&mut serializer)
    }

    /// Serializes a json value into a string
    pub fn to_pretty_string(&self) -> string::String {
        let mut wr = MemWriter::new();
        self.to_pretty_writer(wr.by_ref()).unwrap();
        str::from_utf8(wr.unwrap().as_slice()).unwrap().to_string()
    }

     /// If the Json value is an Object, returns the value associated with the provided key.
    /// Otherwise, returns None.
    pub fn find<'a>(&'a self, key: &string::String) -> Option<&'a Json>{
        match self {
            &Object(ref map) => map.get(key),
            _ => None
        }
    }

    /// Attempts to get a nested Json Object for each key in `keys`.
    /// If any key is found not to exist, find_path will return None.
    /// Otherwise, it will return the Json value associated with the final key.
    pub fn find_path<'a>(&'a self, keys: &[&string::String]) -> Option<&'a Json>{
        let mut target = self;
        for key in keys.iter() {
            match target.find(*key) {
                Some(t) => { target = t; },
                None => return None
            }
        }
        Some(target)
    }

    /// If the Json value is an Object, performs a depth-first search until
    /// a value associated with the provided key is found. If no value is found
    /// or the Json value is not an Object, returns None.
    pub fn search<'a>(&'a self, key: &string::String) -> Option<&'a Json> {
        match self {
            &Object(ref map) => {
                match map.get(key) {
                    Some(json_value) => Some(json_value),
                    None => {
                        let mut value : Option<&'a Json> = None;
                        for (_, v) in map.iter() {
                            value = v.search(key);
                            if value.is_some() {
                                break;
                            }
                        }
                        value
                    }
                }
            },
            _ => None
        }
    }

    /// Returns true if the Json value is an Object. Returns false otherwise.
    pub fn is_object<'a>(&'a self) -> bool {
        self.as_object().is_some()
    }

    /// If the Json value is an Object, returns the associated TreeMap.
    /// Returns None otherwise.
    pub fn as_object<'a>(&'a self) -> Option<&'a JsonObject> {
        match *self {
            Object(ref map) => Some(map),
            _ => None
        }
    }

    /// Returns true if the Json value is a List. Returns false otherwise.
    pub fn is_list<'a>(&'a self) -> bool {
        self.as_list().is_some()
    }

    /// If the Json value is a List, returns the associated vector.
    /// Returns None otherwise.
    pub fn as_list<'a>(&'a self) -> Option<&'a JsonList> {
        match *self {
            List(ref list) => Some(list),
            _ => None
        }
    }

    /// Returns true if the Json value is a String. Returns false otherwise.
    pub fn is_string<'a>(&'a self) -> bool {
        self.as_string().is_some()
    }

    /// If the Json value is a String, returns the associated str.
    /// Returns None otherwise.
    pub fn as_string<'a>(&'a self) -> Option<&'a str> {
        match *self {
            String(ref s) => Some(s.as_slice()),
            _ => None
        }
    }

    /// Returns true if the Json value is a i64 or f64. Returns false otherwise.
    pub fn is_number(&self) -> bool {
        match *self {
            Integer(_) | Floating(_) => true,
            _ => false,
        }
    }

    /// Returns true if the Json value is a i64. Returns false otherwise.
    pub fn is_i64(&self) -> bool {
        match *self {
            Integer(_) => true,
            _ => false,
        }
    }

    /// Returns true if the Json value is a f64. Returns false otherwise.
    pub fn is_f64(&self) -> bool {
        match *self {
            Floating(_) => true,
            _ => false,
        }
    }

    /// If the Json value is a i64, returns the associated i64.
    /// Returns None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Integer(n) => Some(n),
            Floating(n) => Some(n as i64),
            _ => None
        }
    }

    /// If the Json value is a f64, returns the associated f64.
    /// Returns None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Integer(n) => Some(n as f64),
            Floating(n) => Some(n),
            _ => None
        }
    }

    /// Returns true if the Json value is a Boolean. Returns false otherwise.
    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }

    /// If the Json value is a Boolean, returns the associated bool.
    /// Returns None otherwise.
    pub fn as_boolean(&self) -> Option<bool> {
        match *self {
            Boolean(b) => Some(b),
            _ => None
        }
    }

    /// Returns true if the Json value is a Null. Returns false otherwise.
    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    /// If the Json value is a Null, returns ().
    /// Returns None otherwise.
    pub fn as_null(&self) -> Option<()> {
        match *self {
            Null => Some(()),
            _ => None
        }
    }
}

struct WriterFormatter<'a, 'b: 'a>(&'a mut fmt::Formatter<'b>);

impl<'a, 'b> Writer for WriterFormatter<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        let WriterFormatter(ref mut f) = *self;
        f.write(buf).map_err(|_| io::IoError::last_error())
    }
}

impl fmt::Show for Json {
    /// Serializes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_writer(WriterFormatter(f)).map_err(|_| fmt::WriteError)
    }
}

impl<S: ser::Serializer<E>, E> ser::Serialize<S, E> for Json {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        match *self {
            Null => {
                ().serialize(s)
            }
            Boolean(v) => {
                v.serialize(s)
            }
            Integer(v) => {
                v.serialize(s)
            }
            Floating(v) => {
                v.serialize(s)
            }
            String(ref v) => {
                v.serialize(s)
            }
            List(ref v) => {
                v.serialize(s)
            }
            Object(ref v) => {
                v.serialize(s)
            }
        }
    }
}

impl<D: de::Deserializer<E>, E> de::Deserialize<D, E> for Json {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<Json, E> {
        match token {
            de::Null => Ok(Null),
            de::Bool(x) => Ok(Boolean(x)),
            de::Int(x) => Ok(Integer(x as i64)),
            de::I8(x) => Ok(Integer(x as i64)),
            de::I16(x) => Ok(Integer(x as i64)),
            de::I32(x) => Ok(Integer(x as i64)),
            de::I64(x) => Ok(Integer(x)),
            de::Uint(x) => Ok(Integer(x as i64)),
            de::U8(x) => Ok(Integer(x as i64)),
            de::U16(x) => Ok(Integer(x as i64)),
            de::U32(x) => Ok(Integer(x as i64)),
            de::U64(x) => Ok(Integer(x as i64)),
            de::F32(x) => Ok(Floating(x as f64)),
            de::F64(x) => Ok(Floating(x)),
            de::Char(x) => Ok(String(x.to_string())),
            de::Str(x) => Ok(String(x.to_string())),
            de::String(x) => Ok(String(x)),
            de::Option(false) => Ok(Null),
            de::Option(true) => de::Deserialize::deserialize(d),
            de::TupleStart(_) | de::SeqStart(_) => {
                let list = try!(de::Deserialize::deserialize_token(d, token));
                Ok(List(list))
            }
            de::StructStart(_, _) | de::MapStart(_) => {
                let object = try!(de::Deserialize::deserialize_token(d, token));
                Ok(Object(object))
            }
            de::EnumStart(_, name, len) => {
                let token = de::SeqStart(len);
                let fields: Vec<Json> = try!(de::Deserialize::deserialize_token(d, token));
                let mut object = TreeMap::new();
                object.insert(name.to_string(), List(fields));
                Ok(Object(object))
            }
            de::End => Err(d.syntax_error(de::End, [de::EndKind])),
        }
    }
}

enum JsonDeserializerState {
    JsonDeserializerValueState(Json),
    JsonDeserializerListState(vec::MoveItems<Json>),
    JsonDeserializerObjectState(tree_map::MoveEntries<string::String, Json>),
    JsonDeserializerEndState,
}

pub struct JsonDeserializer {
    stack: Vec<JsonDeserializerState>,
}

impl JsonDeserializer {
    /// Creates a new deserializer instance for deserializing the specified JSON value.
    pub fn new(json: Json) -> JsonDeserializer {
        JsonDeserializer {
            stack: vec!(JsonDeserializerValueState(json)),
        }
    }
}

impl Iterator<Result<de::Token, ParserError>> for JsonDeserializer {
    #[inline]
    fn next(&mut self) -> Option<Result<de::Token, ParserError>> {
        loop {
            match self.stack.pop() {
                Some(JsonDeserializerValueState(value)) => {
                    let token = match value {
                        Null => de::Null,
                        Boolean(x) => de::Bool(x),
                        Integer(x) => de::I64(x),
                        Floating(x) => de::F64(x),
                        String(x) => de::String(x),
                        List(x) => {
                            let len = x.len();
                            self.stack.push(JsonDeserializerListState(x.into_iter()));
                            de::SeqStart(len)
                        }
                        Object(x) => {
                            let len = x.len();
                            self.stack.push(JsonDeserializerObjectState(x.into_iter()));
                            de::MapStart(len)
                        }
                    };

                    return Some(Ok(token));
                }
                Some(JsonDeserializerListState(mut iter)) => {
                    match iter.next() {
                        Some(value) => {
                            self.stack.push(JsonDeserializerListState(iter));
                            self.stack.push(JsonDeserializerValueState(value));
                            // loop around.
                        }
                        None => {
                            return Some(Ok(de::End));
                        }
                    }
                }
                Some(JsonDeserializerObjectState(mut iter)) => {
                    match iter.next() {
                        Some((key, value)) => {
                            self.stack.push(JsonDeserializerObjectState(iter));
                            self.stack.push(JsonDeserializerValueState(value));
                            return Some(Ok(de::String(key)));
                        }
                        None => {
                            return Some(Ok(de::End));
                        }
                    }
                }
                Some(JsonDeserializerEndState) => {
                    return Some(Ok(de::End));
                }
                None => { return None; }
            }
        }
    }
}

impl de::Deserializer<ParserError> for JsonDeserializer {
    fn end_of_stream_error(&mut self) -> ParserError {
        SyntaxError(EOFWhileParsingValue, 0, 0)
    }

    fn syntax_error(&mut self, token: de::Token, expected: &[de::TokenKind]) -> ParserError {
        SyntaxError(DeserializerError(token, ExpectTokens(expected.to_vec())), 0, 0)
    }

    fn unexpected_name_error(&mut self, token: de::Token) -> ParserError {
        SyntaxError(DeserializerError(token, ExpectName), 0, 0)
    }

    fn conversion_error(&mut self, token: de::Token) -> ParserError {
        SyntaxError(DeserializerError(token, ExpectConversion), 0, 0)
    }

    #[inline]
    fn missing_field<
        T: de::Deserialize<JsonDeserializer, ParserError>
    >(&mut self, _field: &'static str) -> Result<T, ParserError> {
        // JSON can represent `null` values as a missing value, so this isn't
        // necessarily an error.
        de::Deserialize::deserialize_token(self, de::Null)
    }

    // Special case treating options as a nullable value.
    #[inline]
    fn expect_option<
        U: de::Deserialize<JsonDeserializer, ParserError>
    >(&mut self, token: de::Token) -> Result<Option<U>, ParserError> {
        match token {
            de::Null => Ok(None),
            token => {
                let value: U = try!(de::Deserialize::deserialize_token(self, token));
                Ok(Some(value))
            }
        }
    }

    // Special case treating enums as a String or a `{"variant": "...", "fields": [...]}`.
    #[inline]
    fn expect_enum_start(&mut self,
                         token: de::Token,
                         _name: &str,
                         variants: &[&str]) -> Result<uint, ParserError> {
        let variant = match token {
            de::MapStart(_) => {
                let state = match self.stack.pop() {
                    Some(state) => state,
                    None => { panic!("state machine error, state stack empty"); }
                };

                let mut iter = match state {
                    JsonDeserializerObjectState(iter) => iter,
                    _ => { panic!("state machine error, expected an object"); }
                };

                let (variant, fields) = match iter.next() {
                    Some((variant, List(fields))) => (variant, fields),
                    Some((key, value)) => {
                        return Err(ExpectedError("List".to_string(), format!("{} => {}", key, value)));
                    }
                    None => { return Err(MissingFieldError("<variant-name>".to_string())); }
                };

                // Error out if there are other fields in the enum.
                match iter.next() {
                    Some((key, value)) => {
                        return Err(ExpectedError("None".to_string(), format!("{} => {}", key, value)));
                    }
                    None => { }
                }

                self.stack.push(JsonDeserializerEndState);

                for field in fields.into_iter().rev() {
                    self.stack.push(JsonDeserializerValueState(field));
                }

                variant
            }
            token => {
                return Err(ExpectedError("String or Object".to_string(),
                                         format!("{}", token)))
            }
        };

        match variants.iter().position(|v| *v == variant.as_slice()) {
            Some(idx) => Ok(idx),
            None => Err(UnknownVariantError(variant)),
        }
    }

    #[inline]
    fn expect_struct_start(&mut self, token: de::Token, _name: &str) -> Result<(), ParserError> {
        match token {
            de::MapStart(_) => Ok(()),
            _ => Err(self.syntax_error(token, [de::MapStartKind])),
        }
    }
}

/// The failed expectation of InvalidSyntax
#[deriving(Clone, PartialEq, Show)]
pub enum SyntaxExpectation {
    ListCommaOrEnd,
    ObjectCommaOrEnd,
    SomeValue,
    SomeIdent,
    EnumMapStart,
    EnumVariantString,
    EnumToken,
    EnumEndToken,
    EnumEnd,
}

/// JSON deserializer expectations
#[deriving(Clone, PartialEq, Show)]
pub enum DeserializerExpectation {
    ExpectTokens(Vec<de::TokenKind>),
    ExpectName,
    ExpectConversion,
}

/// The errors that can arise while parsing a JSON stream.
#[deriving(Clone, PartialEq)]
pub enum ErrorCode {
    DeserializerError(de::Token, DeserializerExpectation),
    EOFWhileParsingList,
    EOFWhileParsingObject,
    EOFWhileParsingString,
    EOFWhileParsingValue,
    ExpectedColon,
    InvalidEscape,
    InvalidNumber,
    InvalidSyntax(SyntaxExpectation),
    InvalidUnicodeCodePoint,
    KeyMustBeAString,
    LoneLeadingSurrogateInHexEscape,
    MissingField(&'static str),
    NotFourDigit,
    NotUtf8,
    TrailingCharacters,
    UnexpectedEndOfHexEscape,
    UnknownVariant,
    UnrecognizedHex,
}

#[deriving(Clone, PartialEq, Show)]
pub enum ParserError {
    /// msg, line, col
    SyntaxError(ErrorCode, uint, uint),
    IoError(io::IoErrorKind, &'static str),
    ExpectedError(string::String, string::String),
    MissingFieldError(string::String),
    UnknownVariantError(string::String),
}

// Builder and Parser have the same errors.
pub type BuilderError = ParserError;

/*
#[deriving(Clone, Eq, Show)]
pub enum DecoderError {
    ParseError(ParserError),
    ExpectedError(String, String),
    MissingFieldError(String),
    UnknownVariantError(String),
}
*/

impl fmt::Show for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeserializerError(ref token, ref expect) => write!(f,
                "deserializer found {} when {}", token, expect),
            EOFWhileParsingList => "EOF While parsing list".fmt(f),
            EOFWhileParsingObject => "EOF While parsing object".fmt(f),
            EOFWhileParsingString => "EOF While parsing string".fmt(f),
            EOFWhileParsingValue => "EOF While parsing value".fmt(f),
            ExpectedColon => "expected `:`".fmt(f),
            InvalidEscape => "invalid escape".fmt(f),
            InvalidNumber => "invalid number".fmt(f),
            InvalidSyntax(expect) => write!(f, "invalid syntax, expected: {}", expect),
            InvalidUnicodeCodePoint => "invalid unicode code point".fmt(f),
            KeyMustBeAString => "key must be a string".fmt(f),
            LoneLeadingSurrogateInHexEscape => "lone leading surrogate in hex escape".fmt(f),
            MissingField(field) => write!(f, "missing field \"{}\"", field),
            NotFourDigit => "invalid \\u escape (not four digits)".fmt(f),
            NotUtf8 => "contents not utf-8".fmt(f),
            TrailingCharacters => "trailing characters".fmt(f),
            UnexpectedEndOfHexEscape => "unexpected end of hex escape".fmt(f),
            UnknownVariant => "unknown variant".fmt(f),
            UnrecognizedHex => "invalid \\u escape (unrecognized hex)".fmt(f),
        }
    }
}

/*
fn io_error_to_error(io: io::IoError) -> ParserError {
    IoError(io.kind, io.desc)
}
*/

pub type SerializeResult = io::IoResult<()>;

pub fn escape_bytes<W: Writer>(wr: &mut W, bytes: &[u8]) -> IoResult<()> {
    try!(wr.write_str("\""));

    let mut start = 0;

    for (i, byte) in bytes.iter().enumerate() {
        let escaped = match *byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x08' => "\\b",
            b'\x0c' => "\\f",
            b'\n' => "\\n",
            b'\r' => "\\r",
            b'\t' => "\\t",
            _ => { continue; }
        };

        if start < i {
            try!(wr.write(bytes.slice(start, i)));
        }

        try!(wr.write_str(escaped));

        start = i + 1;
    }

    if start != bytes.len() {
        try!(wr.write(bytes.slice_from(start)));
    }

    wr.write_str("\"")
}

pub fn escape_str<W: Writer>(wr: &mut W, v: &str) -> IoResult<()> {
    escape_bytes(wr, v.as_bytes())
}

pub fn escape_char<W: Writer>(wr: &mut W, v: char) -> IoResult<()> {
    let mut buf = [0, .. 4];
    v.encode_utf8(buf);
    escape_bytes(wr, buf)
}

fn fmt_f32_or_null<W: Writer>(wr: &mut W, v: f32) -> IoResult<()> {
    match v.classify() {
        FPNaN | FPInfinite => wr.write_str("null"),
        _ => wr.write_str(f32::to_str_digits(v, 6).as_slice()),
    }
}

fn fmt_f64_or_null<W: Writer>(wr: &mut W, v: f64) -> IoResult<()> {
    match v.classify() {
        FPNaN | FPInfinite => wr.write_str("null"),
        _ => wr.write_str(f64::to_str_digits(v, 6).as_slice()),
    }
}

fn spaces<W: Writer>(wr: &mut W, mut n: uint) -> IoResult<()> {
    const LEN: uint = 16;
    const BUF: [u8, ..LEN] = [b' ', ..LEN];

    while n >= LEN {
        try!(wr.write(BUF));
        n -= LEN;
    }

    if n > 0 {
        wr.write(BUF.slice_to(n))
    } else {
        Ok(())
    }
}

/*
#[deriving(Show)]
enum SerializerState {
    ValueState,
    TupleState,
    StructState,
    EnumState,
}
*/

/// A structure for implementing serialization to JSON.
pub struct Serializer<W> {
    wr: W,
    first: bool,
}

impl<W: Writer> Serializer<W> {
    /// Creates a new JSON serializer whose output will be written to the writer
    /// specified.
    pub fn new(wr: W) -> Serializer<W> {
        Serializer {
            wr: wr,
            first: true,
        }
    }

    /// Unwrap the Writer from the Serializer.
    pub fn unwrap(self) -> W {
        self.wr
    }
}

impl<W: Writer> ser::Serializer<io::IoError> for Serializer<W> {
    #[inline]
    fn serialize_null(&mut self) -> IoResult<()> {
        self.wr.write_str("null")
    }

    #[inline]
    fn serialize_bool(&mut self, v: bool) -> IoResult<()> {
        if v {
            self.wr.write_str("true")
        } else {
            self.wr.write_str("false")
        }
    }

    #[inline]
    fn serialize_int(&mut self, v: int) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i8(&mut self, v: i8) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i16(&mut self, v: i16) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i32(&mut self, v: i32) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i64(&mut self, v: i64) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_uint(&mut self, v: uint) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u8(&mut self, v: u8) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u16(&mut self, v: u16) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u32(&mut self, v: u32) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u64(&mut self, v: u64) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_f32(&mut self, v: f32) -> IoResult<()> {
        fmt_f32_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_f64(&mut self, v: f64) -> IoResult<()> {
        fmt_f64_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_char(&mut self, v: char) -> IoResult<()> {
        escape_char(&mut self.wr, v)
    }

    #[inline]
    fn serialize_str(&mut self, v: &str) -> IoResult<()> {
        escape_str(&mut self.wr, v)
    }

    #[inline]
    fn serialize_tuple_start(&mut self, _len: uint) -> IoResult<()> {
        self.first = true;
        self.wr.write_str("[")
    }

    #[inline]
    fn serialize_tuple_elt<
        T: Serialize<Serializer<W>, io::IoError>
    >(&mut self, value: &T) -> IoResult<()> {
        if self.first {
            self.first = false;
        } else {
            try!(self.wr.write_str(","));
        }
        value.serialize(self)
    }

    #[inline]
    fn serialize_tuple_end(&mut self) -> IoResult<()> {
        self.wr.write_str("]")
    }

    #[inline]
    fn serialize_struct_start(&mut self, _name: &str, _len: uint) -> IoResult<()> {
        self.first = true;
        self.wr.write_str("{")
    }

    #[inline]
    fn serialize_struct_elt<
        T: Serialize<Serializer<W>, io::IoError>
    >(&mut self, name: &str, value: &T) -> IoResult<()> {
        if self.first {
            self.first = false;
        } else {
            try!(self.wr.write_str(","));
        }
        try!(name.serialize(self));
        try!(self.wr.write_str(":"));
        value.serialize(self)
    }

    #[inline]
    fn serialize_struct_end(&mut self) -> IoResult<()> {
        self.wr.write_str("}")
    }

    #[inline]
    fn serialize_enum_start(&mut self, _name: &str, variant: &str, _len: uint) -> IoResult<()> {
        self.first = true;
        try!(self.wr.write_str("{"));
        try!(self.serialize_str(variant));
        self.wr.write_str(":[")
    }

    #[inline]
    fn serialize_enum_elt<
        T: Serialize<Serializer<W>, io::IoError>
    >(&mut self, value: &T) -> IoResult<()> {
        if self.first {
            self.first = false;
        } else {
            try!(self.wr.write_str(","));
        }
        value.serialize(self)
    }

    #[inline]
    fn serialize_enum_end(&mut self) -> IoResult<()> {
        self.wr.write_str("]}")
    }

    #[inline]
    fn serialize_option<
        T: Serialize<Serializer<W>, io::IoError>
    >(&mut self, v: &Option<T>) -> IoResult<()> {
        match *v {
            Some(ref v) => {
                v.serialize(self)
            }
            None => {
                self.serialize_null()
            }
        }
    }

    #[inline]
    fn serialize_seq<
        T: Serialize<Serializer<W>, io::IoError>,
        Iter: Iterator<T>
    >(&mut self, mut iter: Iter) -> IoResult<()> {
        try!(self.wr.write_str("["));
        let mut first = true;
        for elt in iter {
            if first {
                first = false;
            } else {
                try!(self.wr.write_str(","));
            }
            try!(elt.serialize(self));

        }
        self.wr.write_str("]")
    }

    #[inline]
    fn serialize_map<
        K: Serialize<Serializer<W>, io::IoError>,
        V: Serialize<Serializer<W>, io::IoError>,
        Iter: Iterator<(K, V)>
    >(&mut self, mut iter: Iter) -> IoResult<()> {
        //Warning: WriterFormatter was added to work around
        // the stack overflow happening on this line
        try!(self.wr.write_str("{"));
        let mut first = true;
        for (key, value) in iter {
            if first {
                first = false;
            } else {
                try!(self.wr.write_str(","));
            }
            try!(key.serialize(self));
            try!(self.wr.write_str(":"));
            try!(value.serialize(self));
        }
        self.wr.write_str("}")
    }
}

/// Another serializer for JSON, but prints out human-readable JSON instead of
/// compact data
pub struct PrettySerializer<W> {
    wr: W,
    indent: uint,
    first: bool,
}

impl<W: Writer> PrettySerializer<W> {
    /// Creates a new serializer whose output will be written to the specified writer
    pub fn new(wr: W) -> PrettySerializer<W> {
        PrettySerializer {
            wr: wr,
            indent: 0,
            first: true,
        }
    }

    /// Unwrap the Writer from the Serializer.
    pub fn unwrap(self) -> W {
        self.wr
    }

    #[inline]
    fn serialize_sep(&mut self) -> IoResult<()> {
        if self.first {
            self.first = false;
            self.indent += 2;
            try!(self.wr.write_str("\n"));
        } else {
            try!(self.wr.write_str(",\n"));
        }

        spaces(&mut self.wr, self.indent)
    }

    #[inline]
    fn serialize_end(&mut self, s: &str) -> IoResult<()> {
        if !self.first {
            try!(self.wr.write_str("\n"));
            self.indent -= 2;
            try!(spaces(&mut self.wr, self.indent));
        }

        self.first = false;

        self.wr.write_str(s)
    }
}

impl<W: Writer> ser::Serializer<io::IoError> for PrettySerializer<W> {
    #[inline]
    fn serialize_null(&mut self) -> IoResult<()> {
        self.wr.write_str("null")
    }

    #[inline]
    fn serialize_bool(&mut self, v: bool) -> IoResult<()> {
        if v {
            self.wr.write_str("true")
        } else {
            self.wr.write_str("false")
        }
    }

    #[inline]
    fn serialize_int(&mut self, v: int) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i8(&mut self, v: i8) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i16(&mut self, v: i16) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i32(&mut self, v: i32) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i64(&mut self, v: i64) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_uint(&mut self, v: uint) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u8(&mut self, v: u8) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u16(&mut self, v: u16) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u32(&mut self, v: u32) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u64(&mut self, v: u64) -> IoResult<()> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_f32(&mut self, v: f32) -> IoResult<()> {
        fmt_f32_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_f64(&mut self, v: f64) -> IoResult<()> {
        fmt_f64_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_char(&mut self, v: char) -> IoResult<()> {
        escape_char(&mut self.wr, v)
    }

    #[inline]
    fn serialize_str(&mut self, v: &str) -> IoResult<()> {
        escape_str(&mut self.wr, v)
    }

    #[inline]
    fn serialize_tuple_start(&mut self, _len: uint) -> IoResult<()> {
        self.first = true;
        self.wr.write_str("[")
    }

    #[inline]
    fn serialize_tuple_elt<
        T: Serialize<PrettySerializer<W>, io::IoError>
    >(&mut self, value: &T) -> IoResult<()> {
        try!(self.serialize_sep());
        value.serialize(self)
    }

    #[inline]
    fn serialize_tuple_end(&mut self) -> IoResult<()> {
        self.serialize_end("]")
    }

    #[inline]
    fn serialize_struct_start(&mut self, _name: &str, _len: uint) -> IoResult<()> {
        self.first = true;
        self.wr.write_str("{")
    }

    #[inline]
    fn serialize_struct_elt<
        T: Serialize<PrettySerializer<W>, io::IoError>
    >(&mut self, name: &str, value: &T) -> IoResult<()> {
        try!(self.serialize_sep());
        try!(self.serialize_str(name));
        try!(self.wr.write_str(": "));
        value.serialize(self)
    }

    #[inline]
    fn serialize_struct_end(&mut self) -> IoResult<()> {
        self.serialize_end("}")
    }

    #[inline]
    fn serialize_enum_start(&mut self, _name: &str, variant: &str, _len: uint) -> IoResult<()> {
        self.first = true;
        try!(self.wr.write_str("{"));
        try!(self.serialize_sep());
        try!(self.serialize_str(variant));
        self.first = true;
        self.wr.write_str(": [")
    }

    #[inline]
    fn serialize_enum_elt<
        T: Serialize<PrettySerializer<W>, io::IoError>
    >(&mut self, value: &T) -> IoResult<()> {
        try!(self.serialize_sep());
        value.serialize(self)
    }

    #[inline]
    fn serialize_enum_end(&mut self) -> IoResult<()> {
        try!(self.serialize_tuple_end());
        self.serialize_struct_end()
    }

    #[inline]
    fn serialize_option<
        T: Serialize<PrettySerializer<W>, io::IoError>
    >(&mut self, v: &Option<T>) -> IoResult<()> {
        match *v {
            Some(ref v) => {
                v.serialize(self)
            }
            None => {
                self.serialize_null()
            }
        }
    }

    #[inline]
    fn serialize_seq<
        T: Serialize<PrettySerializer<W>, io::IoError>,
        Iter: Iterator<T>
    >(&mut self, mut iter: Iter) -> IoResult<()> {
        try!(self.wr.write_str("["));

        self.first = true;
        for elt in iter {
            try!(self.serialize_sep());
            try!(elt.serialize(self));
        }

        self.serialize_end("]")
    }

    #[inline]
    fn serialize_map<
        K: Serialize<PrettySerializer<W>, io::IoError>,
        V: Serialize<PrettySerializer<W>, io::IoError>,
        Iter: Iterator<(K, V)>
    >(&mut self, mut iter: Iter) -> IoResult<()> {
        try!(self.wr.write_str("{"));

        self.first = true;
        for (key, value) in iter {
            try!(self.serialize_sep());
            try!(key.serialize(self));
            try!(self.wr.write_str(": "));
            try!(value.serialize(self));
        }

        self.serialize_end("}")
    }
}

/// Encode the specified struct into a json `[u8]` buffer.
#[inline]
pub fn to_vec<
    T: ser::Serialize<Serializer<MemWriter>, io::IoError>
>(value: &T) -> Vec<u8> {
    let wr = MemWriter::with_capacity(1024);
    let mut serializer = Serializer::new(wr);
    // We are writing to a MemWriter, which doesn't fail. So we can ignore
    // the error.
    value.serialize(&mut serializer).unwrap();
    serializer.unwrap().unwrap()
}

/// Encode the specified struct into a json `String` buffer.
#[inline]
pub fn to_string<
    T: ser::Serialize<Serializer<MemWriter>, io::IoError>
>(value: &T) -> Result<string::String, Vec<u8>> {
    let buf = to_vec(value);
    string::String::from_utf8(buf)
}

/// Encode the specified struct into a json `[u8]` buffer.
pub fn to_pretty_vec<
    T: ser::Serialize<PrettySerializer<MemWriter>, io::IoError>
>(value: &T) -> Vec<u8> {
    let wr = MemWriter::new();
    let mut serializer = PrettySerializer::new(wr);
    value.serialize(&mut serializer).unwrap();
    serializer.unwrap().unwrap()
}

/// Encode the specified struct into a json `String` buffer.
pub fn to_pretty_string<
    T: ser::Serialize<PrettySerializer<MemWriter>, io::IoError>
>(value: &T) -> Result<string::String, Vec<u8>> {
    let buf = to_pretty_vec(value);
    string::String::from_utf8(buf)
}

/*
/// The output of the streaming parser.
#[deriving(Eq, Clone, Show)]
pub enum JsonEvent {
    ObjectStart,
    ObjectEnd,
    ListStart,
    ListEnd,
    BooleanValue(bool),
    NumberValue(f64),
    StringValue(String),
    NullValue,
    Error(ParserError),
}
*/

#[deriving(PartialEq, Show)]
enum ParserState {
    // Parse a value.
    ParseValue,
    // Parse a value or ']'.
    ParseListStart,
    // Parse ',' or ']' after an element in a list.
    ParseListCommaOrEnd,
    // Parse a key:value or an ']'.
    ParseObjectStart,
    // Parse ',' or ']' after an element in an object.
    ParseObjectCommaOrEnd,
    // Parse a key in an object.
    //ParseObjectKey,
    // Parse a value in an object.
    ParseObjectValue,
}

/*
/// A Stack represents the current position of the parser in the logical
/// structure of the JSON stream.
/// For example foo.bar[3].x
pub struct Stack {
    stack: Vec<InternalStackElement>,
    str_buffer: Vec<u8>,
}

/// StackElements compose a Stack.
/// For example, Key("foo"), Key("bar"), Index(3) and Key("x") are the
/// StackElements compositing the stack that represents foo.bar[3].x
#[deriving(Eq, Clone, Show)]
pub enum StackElement<'l> {
    Index(u32),
    Key(&'l str),
}

// Internally, Key elements are stored as indices in a buffer to avoid
// allocating a string for every member of an object.
#[deriving(Eq, Clone, Show)]
enum InternalStackElement {
    InternalIndex(u32),
    InternalKey(u16, u16), // start, size
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            stack: Vec::new(),
            str_buffer: Vec::new(),
        }
    }

    /// Returns The number of elements in the Stack.
    pub fn len(&self) -> uint { self.stack.len() }

    /// Returns true if the stack is empty, equivalent to self.len() == 0.
    pub fn is_empty(&self) -> bool { self.stack.len() == 0 }

    /// Provides access to the StackElement at a given index.
    /// lower indices are at the bottom of the stack while higher indices are
    /// at the top.
    pub fn get<'l>(&'l self, idx: uint) -> StackElement<'l> {
        return match *self.stack.get(idx) {
          InternalIndex(i) => { Index(i) }
          InternalKey(start, size) => {
            Key(str::from_utf8(self.str_buffer.slice(start as uint, (start+size) as uint)).unwrap())
          }
        }
    }

    /// Compares this stack with an array of StackElements.
    pub fn is_equal_to(&self, rhs: &[StackElement]) -> bool {
        if self.stack.len() != rhs.len() { return false; }
        for i in range(0, rhs.len()) {
            if self.get(i) != rhs[i] { return false; }
        }
        return true;
    }

    /// Returns true if the bottom-most elements of this stack are the same as
    /// the ones passed as parameter.
    pub fn starts_with(&self, rhs: &[StackElement]) -> bool {
        if self.stack.len() < rhs.len() { return false; }
        for i in range(0, rhs.len()) {
            if self.get(i) != rhs[i] { return false; }
        }
        return true;
    }

    /// Returns true if the top-most elements of this stack are the same as
    /// the ones passed as parameter.
    pub fn ends_with(&self, rhs: &[StackElement]) -> bool {
        if self.stack.len() < rhs.len() { return false; }
        let offset = self.stack.len() - rhs.len();
        for i in range(0, rhs.len()) {
            if self.get(i + offset) != rhs[i] { return false; }
        }
        return true;
    }

    /// Returns the top-most element (if any).
    pub fn top<'l>(&'l self) -> Option<StackElement<'l>> {
        return match self.stack.last() {
            None => None,
            Some(&InternalIndex(i)) => Some(Index(i)),
            Some(&InternalKey(start, size)) => {
                Some(Key(str::from_utf8(
                    self.str_buffer.slice(start as uint, (start+size) as uint)
                ).unwrap()))
            }
        }
    }

    // Used by Parser to insert Key elements at the top of the stack.
    fn push_key(&mut self, key: String) {
        self.stack.push(InternalKey(self.str_buffer.len() as u16, key.len() as u16));
        for c in key.as_bytes().iter() {
            self.str_buffer.push(*c);
        }
    }

    // Used by Parser to insert Index elements at the top of the stack.
    fn push_index(&mut self, index: u32) {
        self.stack.push(InternalIndex(index));
    }

    // Used by Parser to remove the top-most element of the stack.
    fn pop(&mut self) {
        assert!(!self.is_empty());
        match *self.stack.last().unwrap() {
            InternalKey(_, sz) => {
                let new_size = self.str_buffer.len() - sz as uint;
                unsafe {
                    self.str_buffer.set_len(new_size);
                }
            }
            InternalIndex(_) => {}
        }
        self.stack.pop();
    }

    // Used by Parser to test whether the top-most element is an index.
    fn last_is_index(&self) -> bool {
        if self.is_empty() { return false; }
        return match *self.stack.last().unwrap() {
            InternalIndex(_) => true,
            _ => false,
        }
    }

    // Used by Parser to increment the index of the top-most element.
    fn bump_index(&mut self) {
        let len = self.stack.len();
        let idx = match *self.stack.last().unwrap() {
          InternalIndex(i) => { i + 1 }
          _ => { panic!(); }
        };
        *self.stack.get_mut(len - 1) = InternalIndex(idx);
    }
}
*/

/// A streaming JSON parser implemented as an iterator of JsonEvent, consuming
/// an iterator of char.
pub struct Parser<Iter> {
    rdr: Iter,
    ch: Option<u8>,
    line: uint,
    col: uint,
    // A state machine is kept to make it possible to interupt and resume parsing.
    state_stack: Vec<ParserState>,
    buf: Vec<u8>,
}

impl<Iter: Iterator<u8>> Iterator<Result<de::Token, ParserError>> for Parser<Iter> {
    #[inline]
    fn next(&mut self) -> Option<Result<de::Token, ParserError>> {
        let state = match self.state_stack.pop() {
            Some(state) => state,
            None => {
                // If we have no state left, then we're expecting the structure
                // to be done, so make sure there are no trailing characters.

                self.parse_whitespace();

                if self.eof() {
                    return None;
                } else {
                    return Some(self.error(TrailingCharacters));
                }
            }
        };

        match state {
            ParseValue => Some(self.parse_value()),
            ParseListStart => Some(self.parse_list_start()),
            ParseListCommaOrEnd => Some(self.parse_list_comma_or_end()),
            ParseObjectStart => {
                match self.parse_object_start() {
                    Ok(Some(s)) => Some(Ok(de::String(s.to_string()))),
                    Ok(None) => Some(Ok(de::End)),
                    Err(err) => Some(Err(err)),
                }
            }
            ParseObjectCommaOrEnd => {
                match self.parse_object_comma_or_end() {
                    Ok(Some(s)) => Some(Ok(de::String(s.to_string()))),
                    Ok(None) => Some(Ok(de::End)),
                    Err(err) => Some(Err(err)),
                }
            }
            //ParseObjectKey => Some(self.parse_object_key()),
            ParseObjectValue => Some(self.parse_object_value()),
        }
    }
}

impl<Iter: Iterator<u8>> Parser<Iter> {
    /// Creates the JSON parser.
    #[inline]
    pub fn new(rdr: Iter) -> Parser<Iter> {
        let mut p = Parser {
            rdr: rdr,
            ch: Some(b'\x00'),
            line: 1,
            col: 0,
            state_stack: vec!(ParseValue),
            buf: Vec::with_capacity(100),
        };
        p.bump();
        return p;
    }

    #[inline(always)]
    fn eof(&self) -> bool { self.ch.is_none() }

    #[inline]
    fn ch_or_null(&self) -> u8 { self.ch.unwrap_or(b'\x00') }

    #[inline(always)]
    fn bump(&mut self) {
        self.ch = self.rdr.next();

        if self.ch_is(b'\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }

    #[inline]
    fn next_char(&mut self) -> Option<u8> {
        self.bump();
        self.ch
    }

    #[inline(always)]
    fn ch_is(&self, c: u8) -> bool {
        self.ch == Some(c)
    }

    #[inline]
    fn error<T>(&self, reason: ErrorCode) -> Result<T, ParserError> {
        Err(SyntaxError(reason, self.line, self.col))
    }

    #[inline]
    fn parse_whitespace(&mut self) {
        while self.ch_is(b' ') ||
              self.ch_is(b'\n') ||
              self.ch_is(b'\t') ||
              self.ch_is(b'\r') { self.bump(); }
    }

    #[inline]
    fn parse_number(&mut self) -> Result<de::Token, ParserError> {
        let mut neg = 1;

        if self.ch_is(b'-') {
            self.bump();
            neg = -1;
        }

        let res = try!(self.parse_integer());

        if self.ch_is(b'.') || self.ch_is(b'e') || self.ch_is(b'E') {
            let neg = neg as f64;
            let mut res = res as f64;

            if self.ch_is(b'.') {
                res = try!(self.parse_decimal(res));
            }

            if self.ch_is(b'e') || self.ch_is(b'E') {
                res = try!(self.parse_exponent(res));
            }

            Ok(de::F64(neg * res))
        } else {
            Ok(de::I64(neg * res))
        }
    }

    #[inline]
    fn parse_integer(&mut self) -> Result<i64, ParserError> {
        let mut res = 0;

        match self.ch_or_null() {
            b'0' => {
                self.bump();

                // There can be only one leading '0'.
                match self.ch_or_null() {
                    b'0' ... b'9' => return self.error(InvalidNumber),
                    _ => ()
                }
            },
            b'1' ... b'9' => {
                while !self.eof() {
                    match self.ch_or_null() {
                        c @ b'0' ... b'9' => {
                            res *= 10;
                            res += (c as i64) - ('0' as i64);
                            self.bump();
                        }
                        _ => break,
                    }
                }
            }
            _ => return self.error(InvalidNumber),
        }

        Ok(res)
    }

    #[inline]
    fn parse_decimal(&mut self, res: f64) -> Result<f64, ParserError> {
        self.bump();

        // Make sure a digit follows the decimal place.
        match self.ch_or_null() {
            b'0' ... b'9' => (),
             _ => return self.error(InvalidNumber)
        }

        let mut res = res;
        let mut dec = 1.0;
        while !self.eof() {
            match self.ch_or_null() {
                c @ b'0' ... b'9' => {
                    dec /= 10.0;
                    res += (((c as int) - (b'0' as int)) as f64) * dec;
                    self.bump();
                }
                _ => break,
            }
        }

        Ok(res)
    }

    #[inline]
    fn parse_exponent(&mut self, mut res: f64) -> Result<f64, ParserError> {
        self.bump();

        let mut exp = 0u;
        let mut neg_exp = false;

        if self.ch_is(b'+') {
            self.bump();
        } else if self.ch_is(b'-') {
            self.bump();
            neg_exp = true;
        }

        // Make sure a digit follows the exponent place.
        match self.ch_or_null() {
            b'0' ... b'9' => (),
            _ => return self.error(InvalidNumber)
        }
        while !self.eof() {
            match self.ch_or_null() {
                c @ b'0' ... b'9' => {
                    exp *= 10;
                    exp += (c as uint) - (b'0' as uint);

                    self.bump();
                }
                _ => break
            }
        }

        let exp: f64 = num::pow(10u as f64, exp);
        if neg_exp {
            res /= exp;
        } else {
            res *= exp;
        }

        Ok(res)
    }

    #[inline]
    fn decode_hex_escape(&mut self) -> Result<u16, ParserError> {
        let mut i = 0u;
        let mut n = 0u16;
        while i < 4u && !self.eof() {
            self.bump();
            n = match self.ch_or_null() {
                c @ b'0' ... b'9' => n * 16_u16 + ((c as u16) - (b'0' as u16)),
                b'a' | b'A' => n * 16_u16 + 10_u16,
                b'b' | b'B' => n * 16_u16 + 11_u16,
                b'c' | b'C' => n * 16_u16 + 12_u16,
                b'd' | b'D' => n * 16_u16 + 13_u16,
                b'e' | b'E' => n * 16_u16 + 14_u16,
                b'f' | b'F' => n * 16_u16 + 15_u16,
                _ => return self.error(InvalidEscape)
            };

            i += 1u;
        }

        // Error out if we didn't parse 4 digits.
        if i != 4u {
            return self.error(InvalidEscape);
        }

        Ok(n)
    }

    #[inline]
    fn parse_string(&mut self) -> Result<&str, ParserError> {
        self.buf.clear();

        let mut escape = false;


        loop {
            let ch = match self.next_char() {
                Some(ch) => ch,
                None => { return self.error(EOFWhileParsingString); }
            };

            if escape {
                match ch {
                    b'"' => self.buf.push(b'"'),
                    b'\\' => self.buf.push(b'\\'),
                    b'/' => self.buf.push(b'/'),
                    b'b' => self.buf.push(b'\x08'),
                    b'f' => self.buf.push(b'\x0c'),
                    b'n' => self.buf.push(b'\n'),
                    b'r' => self.buf.push(b'\r'),
                    b't' => self.buf.push(b'\t'),
                    b'u' => {
                        let c = match try!(self.decode_hex_escape()) {
                            0xDC00 ... 0xDFFF => return self.error(LoneLeadingSurrogateInHexEscape),

                            // Non-BMP characters are encoded as a sequence of
                            // two hex escapes, representing UTF-16 surrogates.
                            n1 @ 0xD800 ... 0xDBFF => {
                                let c1 = self.next_char();
                                let c2 = self.next_char();
                                match (c1, c2) {
                                    (Some(b'\\'), Some(b'u')) => (),
                                    _ => return self.error(UnexpectedEndOfHexEscape),
                                }

                                let buf = [n1, try!(self.decode_hex_escape())];
                                match str::utf16_items(buf.as_slice()).next() {
                                    Some(ScalarValue(c)) => c,
                                    _ => return self.error(LoneLeadingSurrogateInHexEscape),
                                }
                            }

                            n => match char::from_u32(n as u32) {
                                Some(c) => c,
                                None => return self.error(InvalidUnicodeCodePoint),
                            }
                        };

                        let mut buf = [0u8, .. 4];
                        let len = c.encode_utf8(buf).unwrap_or(0);
                        self.buf.extend(buf.slice_to(len).iter().map(|b| *b));
                    }
                    _ => return self.error(InvalidEscape),
                }
                escape = false;
            } else {
                match ch {
                    b'"' => {
                        self.bump();
                        return Ok(str::from_utf8(self.buf.as_slice()).unwrap());
                    }
                    b'\\' => {
                        escape = true;
                    }
                    ch => {
                        self.buf.push(ch);
                    }
                }
            }
        }
    }

    #[inline]
    fn parse_list_start(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.ch_is(b']') {
            self.bump();
            Ok(de::End)
        } else {
            self.state_stack.push(ParseListCommaOrEnd);
            self.parse_value()
        }
    }

    #[inline]
    fn parse_list_comma_or_end(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.ch_is(b',') {
            self.bump();
            self.state_stack.push(ParseListCommaOrEnd);
            self.parse_value()
        } else if self.ch_is(b']') {
            self.bump();
            Ok(de::End)
        } else if self.eof() {
            self.error_event(EOFWhileParsingList)
        } else {
            self.error_event(InvalidSyntax(ListCommaOrEnd))
        }
    }

    #[inline]
    fn parse_object_start(&mut self) -> Result<Option<&str>, ParserError> {
        self.parse_whitespace();

        if self.ch_is(b'}') {
            self.bump();
            Ok(None)
        } else {
            Ok(Some(try!(self.parse_object_key())))
        }
    }

    #[inline]
    fn parse_object_comma_or_end(&mut self) -> Result<Option<&str>, ParserError> {
        self.parse_whitespace();

        if self.ch_is(b',') {
            self.bump();
            Ok(Some(try!(self.parse_object_key())))
        } else if self.ch_is(b'}') {
            self.bump();
            Ok(None)
        } else if self.eof() {
            self.error_event(EOFWhileParsingObject)
        } else {
            self.error_event(InvalidSyntax(ObjectCommaOrEnd))
        }
    }

    #[inline]
    fn parse_object_key(&mut self) -> Result<&str, ParserError> {
        self.parse_whitespace();

        if self.eof() {
            return self.error_event(EOFWhileParsingString);
        }

        match self.ch_or_null() {
            b'"' => {
                self.state_stack.push(ParseObjectValue);

                Ok(try!(self.parse_string()))
            }
            _ => self.error_event(KeyMustBeAString),
        }
    }

    #[inline]
    fn parse_object_value(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.ch_is(b':') {
            self.bump();
            self.state_stack.push(ParseObjectCommaOrEnd);
            self.parse_value()
        } else if self.eof() {
            self.error_event(EOFWhileParsingObject)
        } else {
            self.error_event(ExpectedColon)
        }
    }

    #[inline]
    fn parse_value(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.eof() {
            return self.error_event(EOFWhileParsingValue);
        }

        match self.ch_or_null() {
            b'n' => self.parse_ident(b"ull", de::Null),
            b't' => self.parse_ident(b"rue", de::Bool(true)),
            b'f' => self.parse_ident(b"alse", de::Bool(false)),
            b'0' ... b'9' | b'-' => self.parse_number(),
            b'"' => {
                Ok(de::String(try!(self.parse_string()).to_string()))
            }
            b'[' => {
                self.bump();
                self.state_stack.push(ParseListStart);
                Ok(de::SeqStart(0))
            }
            b'{' => {
                self.bump();
                self.state_stack.push(ParseObjectStart);
                Ok(de::MapStart(0))
            }
            _ => {
                self.error_event(InvalidSyntax(SomeValue))
            }
        }
    }

    #[inline]
    fn parse_ident(&mut self, ident: &[u8], token: de::Token) -> Result<de::Token, ParserError> {
        if ident.iter().all(|c| Some(*c) == self.next_char()) {
            self.bump();
            Ok(token)
        } else {
            self.error_event(InvalidSyntax(SomeIdent))
        }
    }

    #[inline]
    fn error_event<T>(&mut self, reason: ErrorCode) -> Result<T, ParserError> {
        self.state_stack.clear();
        Err(SyntaxError(reason, self.line, self.col))
    }
}

impl<Iter: Iterator<u8>> de::Deserializer<ParserError> for Parser<Iter> {
    fn end_of_stream_error(&mut self) -> ParserError {
        SyntaxError(EOFWhileParsingValue, self.line, self.col)
    }

    fn syntax_error(&mut self, token: de::Token, expected: &[de::TokenKind]) -> ParserError {
        SyntaxError(DeserializerError(token, ExpectTokens(expected.to_vec())), self.line, self.col)
    }

    fn unexpected_name_error(&mut self, token: de::Token) -> ParserError {
        SyntaxError(DeserializerError(token, ExpectName), self.line, self.col)
    }

    fn conversion_error(&mut self, token: de::Token) -> ParserError {
        SyntaxError(DeserializerError(token, ExpectConversion), self.line, self.col)
    }

    #[inline]
    fn missing_field<
        T: de::Deserialize<Parser<Iter>, ParserError>
    >(&mut self, _field: &'static str) -> Result<T, ParserError> {
        // JSON can represent `null` values as a missing value, so this isn't
        // necessarily an error.
        de::Deserialize::deserialize_token(self, de::Null)
    }

    // Special case treating options as a nullable value.
    #[inline]
    fn expect_option<
        U: de::Deserialize<Parser<Iter>, ParserError>
    >(&mut self, token: de::Token) -> Result<Option<U>, ParserError> {
        match token {
            de::Null => Ok(None),
            token => {
                let value: U = try!(de::Deserialize::deserialize_token(self, token));
                Ok(Some(value))
            }
        }
    }

    // Special case treating enums as a `{"<variant-name>": [<fields>]}`.
    #[inline]
    fn expect_enum_start(&mut self,
                         token: de::Token,
                         _name: &str,
                         variants: &[&str]) -> Result<uint, ParserError> {
        match token {
            de::MapStart(_) => { }
            _ => { return self.error(InvalidSyntax(EnumMapStart)); }
        };

        // Enums only have one field in them, which is the variant name.
        let variant = match try!(self.expect_token()) {
            de::String(variant) => variant,
            _ => { return self.error(InvalidSyntax(EnumVariantString)); }
        };

        // The variant's field is a list of the values.
        match try!(self.expect_token()) {
            de::SeqStart(_) => { }
            _ => { return self.error(InvalidSyntax(EnumToken)); }
        }

        match variants.iter().position(|v| *v == variant.as_slice()) {
            Some(idx) => Ok(idx),
            None => self.error(UnknownVariant),
        }
    }

    fn expect_enum_end(&mut self) -> Result<(), ParserError> {
        // There will be one `End` for the list, and one for the object.
        match try!(self.expect_token()) {
            de::End => {
                match try!(self.expect_token()) {
                    de::End => Ok(()),
                    _ => self.error(InvalidSyntax(EnumEndToken)),
                }
            }
            _ => self.error(InvalidSyntax(EnumEnd)),
        }
    }

    #[inline]
    fn expect_struct_start(&mut self, token: de::Token, _name: &str) -> Result<(), ParserError> {
        match token {
            de::MapStart(_) => Ok(()),
            _ => Err(self.syntax_error(token, [de::MapStartKind])),
        }
    }

    #[inline]
    fn expect_struct_field_or_end(&mut self,
                                  fields: &'static [&'static str]
                                 ) -> Result<Option<Option<uint>>, ParserError> {
        let result = match self.state_stack.pop() {
            Some(ParseObjectStart) => {
                try!(self.parse_object_start())
            }
            Some(ParseObjectCommaOrEnd) => {
                try!(self.parse_object_comma_or_end())
            }
            _ => panic!("invalid internal state"),
        };

        let s = match result {
            Some(s) => s,
            None => { return Ok(None); }
        };

        Ok(Some(fields.iter().position(|field| *field == s.as_slice())))
    }
}

/// Decodes a json value from an `Iterator<u8>`.
pub fn from_iter<
    Iter: Iterator<u8>,
    T: de::Deserialize<Parser<Iter>, ParserError>
>(iter: Iter) -> Result<T, ParserError> {
    let mut parser = Parser::new(iter);
    let value = try!(de::Deserialize::deserialize(&mut parser));

    // Make sure the whole stream has been consumed.
    match parser.next() {
        Some(Ok(_token)) => parser.error(TrailingCharacters),
        Some(Err(err)) => Err(err),
        None => Ok(value),
    }
}

/// Decodes a json value from a string
pub fn from_str<
    'a,
    T: de::Deserialize<Parser<str::Bytes<'a>>, ParserError>
>(s: &'a str) -> Result<T, BuilderError> {
    from_iter(s.bytes())
}

/// Decodes a json value from a `Json`.
pub fn from_json<
    T: de::Deserialize<JsonDeserializer, ParserError>
>(json: Json) -> Result<T, ParserError> {
    let mut d = JsonDeserializer::new(json);
    de::Deserialize::deserialize(&mut d)
}

macro_rules! expect(
    ($e:expr, Null) => ({
        match $e {
            Null => Ok(()),
            other => Err(ExpectedError("Null".to_string(),
                                       format!("{}", other)))
        }
    });
    ($e:expr, $t:ident) => ({
        match $e {
            $t(v) => Ok(v),
            other => {
                Err(ExpectedError(stringify!($t).to_string(),
                                  format!("{}", other)))
            }
        }
    })
)

/// A trait for converting values to JSON
pub trait ToJson {
    /// Converts the value of `self` to an instance of JSON
    fn to_json(&self) -> Json;
}

impl ToJson for Json {
    fn to_json(&self) -> Json { (*self).clone() }
}

impl ToJson for int {
    fn to_json(&self) -> Json { Integer(*self as i64) }
}

impl ToJson for i8 {
    fn to_json(&self) -> Json { Integer(*self as i64) }
}

impl ToJson for i16 {
    fn to_json(&self) -> Json { Integer(*self as i64) }
}

impl ToJson for i32 {
    fn to_json(&self) -> Json { Integer(*self as i64) }
}

impl ToJson for i64 {
    fn to_json(&self) -> Json { Integer(*self as i64) }
}

impl ToJson for uint {
    fn to_json(&self) -> Json { Integer(*self as i64) }
}

impl ToJson for u8 {
    fn to_json(&self) -> Json { Integer(*self as i64) }
}

impl ToJson for u16 {
    fn to_json(&self) -> Json { Integer(*self as i64) }
}

impl ToJson for u32 {
    fn to_json(&self) -> Json { Integer(*self as i64) }
}

impl ToJson for u64 {
    fn to_json(&self) -> Json { Integer(*self as i64) }
}

impl ToJson for f32 {
    fn to_json(&self) -> Json { Floating(*self as f64) }
}

impl ToJson for f64 {
    fn to_json(&self) -> Json { Floating(*self) }
}

impl ToJson for bool {
    fn to_json(&self) -> Json { Boolean(*self) }
}

impl<'a> ToJson for &'a str {
    fn to_json(&self) -> Json { String(self.to_string()) }
}

impl ToJson for string::String {
    fn to_json(&self) -> Json { String((*self).clone()) }
}

macro_rules! peel_to_json_tuple {
    ($name:ident, $($other:ident,)*) => (impl_to_json_tuple!($($other,)*))
}

macro_rules! impl_to_json_tuple {
    () => {
        impl<> ToJson for () {
            #[inline]
            fn to_json(&self) -> Json {
                Null
            }
        }
    };
    ( $($name:ident,)+ ) => {
        impl<$($name: ToJson),*> ToJson for ($($name,)*) {
            #[inline]
            #[allow(non_snake_case)]
            fn to_json(&self) -> Json {
                // FIXME: how can we count macro args?
                let mut len = 0;
                $({ let $name = 1; len += $name; })*;

                let ($(ref $name,)*) = *self;

                let mut list = Vec::with_capacity(len);
                $(
                    list.push($name.to_json());
                 )*

                List(list)
            }
        }
        peel_to_json_tuple!($($name,)*)
    }
}

impl_to_json_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

impl<A:ToJson> ToJson for Vec<A> {
    fn to_json(&self) -> Json { List(self.iter().map(|elt| elt.to_json()).collect()) }
}

impl<A:ToJson> ToJson for TreeMap<string::String, A> {
    fn to_json(&self) -> Json {
        let mut d = TreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_json());
        }
        Object(d)
    }
}

impl<A:ToJson> ToJson for HashMap<string::String, A> {
    fn to_json(&self) -> Json {
        let mut d = TreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_json());
        }
        Object(d)
    }
}

impl<A:ToJson> ToJson for Option<A> {
    fn to_json(&self) -> Json {
        match *self {
          None => Null,
          Some(ref value) => value.to_json()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Show;
    use std::io;
    use std::str;
    use std::string;
    use std::collections::TreeMap;

    use super::{Json, Null, Boolean, Floating, String, List, Object};
    use super::{Parser, ParserError, from_str};
    use super::{JsonDeserializer, ToJson, from_json};
    use super::{
        EOFWhileParsingList,
        EOFWhileParsingObject,
        EOFWhileParsingString,
        EOFWhileParsingValue,
        ExpectedColon,
        InvalidNumber,
        InvalidSyntax,
        KeyMustBeAString,
        TrailingCharacters,
        SyntaxError,
        SomeIdent,
        SomeValue,
        ObjectCommaOrEnd,
        ListCommaOrEnd,
    };
    use de;
    use ser::{Serialize, Serializer};
    use ser;

    macro_rules! treemap {
        ($($k:expr => $v:expr),*) => ({
            let mut _m = ::std::collections::TreeMap::new();
            $(_m.insert($k, $v);)*
            _m
        })
    }

    #[deriving(PartialEq, Show)]
    #[deriving_serialize]
    #[deriving_deserialize]
    enum Animal {
        Dog,
        Frog(string::String, Vec<int>)
    }

    impl ToJson for Animal {
        fn to_json(&self) -> Json {
            match *self {
                Dog => {
                    Object(
                        treemap!(
                            "Dog".to_string() => List(vec!())
                        )
                    )
                }
                Frog(ref x0, ref x1) => {
                    Object(
                        treemap!(
                            "Frog".to_string() => List(vec!(x0.to_json(), x1.to_json()))
                        )
                    )
                }
            }
        }
    }

    #[deriving(PartialEq, Show)]
    #[deriving_serialize]
    #[deriving_deserialize]
    struct Inner {
        a: (),
        b: uint,
        c: Vec<string::String>,
    }

    impl ToJson for Inner {
        fn to_json(&self) -> Json {
            Object(
                treemap!(
                    "a".to_string() => self.a.to_json(),
                    "b".to_string() => self.b.to_json(),
                    "c".to_string() => self.c.to_json()
                )
            )
        }
    }

    #[deriving(PartialEq, Show)]
    #[deriving_serialize]
    #[deriving_deserialize]
    struct Outer {
        inner: Vec<Inner>,
    }

    impl ToJson for Outer {
        fn to_json(&self) -> Json {
            Object(
                treemap!(
                    "inner".to_string() => self.inner.to_json()
                )
            )
        }
    }

    fn test_encode_ok<
        T: PartialEq + Show + ToJson + ser::Serialize<super::Serializer<io::MemWriter>, io::IoError>
    >(errors: &[(T, &str)]) {
        for &(ref value, out) in errors.iter() {
            let out = out.to_string();

            let s = super::to_string(value).unwrap();
            assert_eq!(s, out);

            let s = super::to_string(&value.to_json()).unwrap();
            assert_eq!(s, out);
        }
    }

    fn test_pretty_encode_ok<
        T: PartialEq + Show + ToJson + ser::Serialize<super::PrettySerializer<io::MemWriter>, io::IoError>
    >(errors: &[(T, &str)]) {
        for &(ref value, out) in errors.iter() {
            let out = out.to_string();

            let s = super::to_pretty_string(value).unwrap();
            assert_eq!(s, out);

            let s = super::to_pretty_string(&value.to_json()).unwrap();
            assert_eq!(s, out);
        }
    }

    #[test]
    fn test_write_null() {
        let tests = [
            ((), "null"),
        ];
        test_encode_ok(tests);
        test_pretty_encode_ok(tests);
    }

    #[test]
    fn test_write_i64() {
        let tests = [
            (3i, "3"),
            (-2i, "-2"),
            (-1234i, "-1234"),
        ];
        test_encode_ok(tests);
        test_pretty_encode_ok(tests);
    }

    #[test]
    fn test_write_f64() {
        let tests = [
            (3.0f64, "3"),
            (3.1, "3.1"),
            (-1.5, "-1.5"),
            (0.5, "0.5"),
        ];
        test_encode_ok(tests);
        test_pretty_encode_ok(tests);
    }

    #[test]
    fn test_write_str() {
        let tests = [
            ("", "\"\""),
            ("foo", "\"foo\""),
        ];
        test_encode_ok(tests);
        test_pretty_encode_ok(tests);
    }

    #[test]
    fn test_write_bool() {
        let tests = [
            (true, "true"),
            (false, "false"),
        ];
        test_encode_ok(tests);
        test_pretty_encode_ok(tests);
    }

    #[test]
    fn test_write_list() {
        test_encode_ok([
            (vec!(), "[]"),
            (vec!(true), "[true]"),
            (vec!(true, false), "[true,false]"),
        ]);

        test_pretty_encode_ok([
            (vec!(), "[]"),
            (
                vec!(true),
                concat!(
                    "[\n",
                    "  true\n",
                    "]"
                ),
            ),
            (
                vec!(true, false),
                concat!(
                    "[\n",
                    "  true,\n",
                    "  false\n",
                    "]"
                ),
            ),
        ]);

        let long_test_list = List(vec![
            Boolean(false),
            Null,
            List(vec![String("foo\nbar".to_string()), Floating(3.5)])]);

        test_encode_ok([
            (long_test_list, "[false,null,[\"foo\\nbar\",3.5]]"),
        ]);

        let long_test_list = List(vec![
            Boolean(false),
            Null,
            List(vec![String("foo\nbar".to_string()), Floating(3.5)])]);

        test_pretty_encode_ok([
            (
                long_test_list,
                concat!(
                    "[\n",
                    "  false,\n",
                    "  null,\n",
                    "  [\n",
                    "    \"foo\\nbar\",\n",
                    "    3.5\n",
                    "  ]\n",
                    "]"
                )
            )
        ]);
    }

    #[test]
    fn test_write_object() {
        test_encode_ok([
            (treemap!(), "{}"),
            (treemap!("a".to_string() => true), "{\"a\":true}"),
            (
                treemap!(
                    "a".to_string() => true,
                    "b".to_string() => false
                ),
                "{\"a\":true,\"b\":false}"),
        ]);

        test_pretty_encode_ok([
            (treemap!(), "{}"),
            (
                treemap!("a".to_string() => true),
                concat!(
                    "{\n",
                    "  \"a\": true\n",
                    "}"
                ),
            ),
            (
                treemap!(
                    "a".to_string() => true,
                    "b".to_string() => false
                ),
                concat!(
                    "{\n",
                    "  \"a\": true,\n",
                    "  \"b\": false\n",
                    "}"
                ),
            ),
        ]);

        let complex_obj = Object(treemap!(
            "b".to_string() => List(vec!(
                Object(treemap!("c".to_string() => String("\x0c\r".to_string()))),
                Object(treemap!("d".to_string() => String("".to_string())))
            ))
        ));

        test_encode_ok([
            (
                complex_obj.clone(),
                "{\
                    \"b\":[\
                        {\"c\":\"\\f\\r\"},\
                        {\"d\":\"\"}\
                    ]\
                }"
            ),
        ]);

        test_pretty_encode_ok([
            (
                complex_obj.clone(),
                concat!(
                    "{\n",
                    "  \"b\": [\n",
                    "    {\n",
                    "      \"c\": \"\\f\\r\"\n",
                    "    },\n",
                    "    {\n",
                    "      \"d\": \"\"\n",
                    "    }\n",
                    "  ]\n",
                    "}"
                ),
            )
        ]);
    }

    #[test]
    fn test_write_tuple() {
        test_encode_ok([
            (
                (5i,),
                "[5]",
            ),
        ]);

        test_pretty_encode_ok([
            (
                (5i,),
                concat!(
                    "[\n",
                    "  5\n",
                    "]"
                ),
            ),
        ]);

        test_encode_ok([
            (
                (5i, (6i, "abc")),
                "[5,[6,\"abc\"]]",
            ),
        ]);

        test_pretty_encode_ok([
            (
                (5i, (6i, "abc")),
                concat!(
                    "[\n",
                    "  5,\n",
                    "  [\n",
                    "    6,\n",
                    "    \"abc\"\n",
                    "  ]\n",
                    "]"
                ),
            ),
        ]);
    }

    #[test]
    fn test_write_enum() {
        test_encode_ok([
            (Dog, "{\"Dog\":[]}"),
            (Frog("Henry".to_string(), vec!()), "{\"Frog\":[\"Henry\",[]]}"),
            (Frog("Henry".to_string(), vec!(349)), "{\"Frog\":[\"Henry\",[349]]}"),
            (Frog("Henry".to_string(), vec!(349, 102)), "{\"Frog\":[\"Henry\",[349,102]]}"),
        ]);

        test_pretty_encode_ok([
            (
                Dog,
                concat!(
                    "{\n",
                    "  \"Dog\": []\n",
                    "}"
                ),
            ),
            (
                Frog("Henry".to_string(), vec!()),
                concat!(
                    "{\n",
                    "  \"Frog\": [\n",
                    "    \"Henry\",\n",
                    "    []\n",
                    "  ]\n",
                    "}"
                ),
            ),
            (
                Frog("Henry".to_string(), vec!(349)),
                concat!(
                    "{\n",
                    "  \"Frog\": [\n",
                    "    \"Henry\",\n",
                    "    [\n",
                    "      349\n",
                    "    ]\n",
                    "  ]\n",
                    "}"
                ),
            ),
            (
                Frog("Henry".to_string(), vec!(349, 102)),
                concat!(
                    "{\n",
                    "  \"Frog\": [\n",
                    "    \"Henry\",\n",
                    "    [\n",
                    "      349,\n",
                    "      102\n",
                    "    ]\n",
                    "  ]\n",
                    "}"
                ),
            ),
        ]);
    }

    #[test]
    fn test_write_option() {
        test_encode_ok([
            (None, "null"),
            (Some("jodhpurs"), "\"jodhpurs\""),
        ]);

        test_encode_ok([
            (None, "null"),
            (Some(vec!("foo", "bar")), "[\"foo\",\"bar\"]"),
        ]);

        test_pretty_encode_ok([
            (None, "null"),
            (Some("jodhpurs"), "\"jodhpurs\""),
        ]);

        test_pretty_encode_ok([
            (None, "null"),
            (
                Some(vec!("foo", "bar")),
                concat!(
                    "[\n",
                    "  \"foo\",\n",
                    "  \"bar\"\n",
                    "]"
                ),
            ),
        ]);

    }

    // FIXME (#5527): these could be merged once UFCS is finished.
    fn test_parse_err<
        'a,
        T: Show + de::Deserialize<Parser<str::Bytes<'a>>, ParserError>
    >(errors: &[(&'a str, ParserError)]) {
        for &(s, ref err) in errors.iter() {
            let v: Result<T, ParserError> = from_str(s);
            assert_eq!(v.unwrap_err(), *err);
        }
    }

    fn test_parse_ok<
        'a,
        T: PartialEq + Show + ToJson + de::Deserialize<Parser<str::Bytes<'a>>, ParserError>
    >(errors: &[(&'a str, T)]) {
        for &(s, ref value) in errors.iter() {
            let v: T = from_str(s).unwrap();
            assert_eq!(v, *value);

            let v: Json = from_str(s).unwrap();
            assert_eq!(v, value.to_json());
        }
    }

    fn test_json_deserialize_ok<
        T: PartialEq + Show + ToJson + de::Deserialize<JsonDeserializer, ParserError>
    >(errors: &[T]) {
        for value in errors.iter() {
            let v: T = from_json(value.to_json()).unwrap();
            assert_eq!(v, *value);

            // Make sure we can round trip back to `Json`.
            let v: Json = from_json(value.to_json()).unwrap();
            assert_eq!(v, value.to_json());
        }
    }

    #[test]
    fn test_parse_null() {
        test_parse_err::<()>([
            ("n", SyntaxError(InvalidSyntax(SomeIdent), 1, 2)),
            ("nul", SyntaxError(InvalidSyntax(SomeIdent), 1, 4)),
            ("nulla", SyntaxError(TrailingCharacters, 1, 5)),
        ]);

        test_parse_ok([
            ("null", ()),
        ]);
    }

    #[test]
    fn test_json_deserialize_null() {
        test_json_deserialize_ok([
            (),
        ]);
    }

    #[test]
    fn test_parse_bool() {
        test_parse_err::<bool>([
            ("t", SyntaxError(InvalidSyntax(SomeIdent), 1, 2)),
            ("truz", SyntaxError(InvalidSyntax(SomeIdent), 1, 4)),
            ("f", SyntaxError(InvalidSyntax(SomeIdent), 1, 2)),
            ("faz", SyntaxError(InvalidSyntax(SomeIdent), 1, 3)),
            ("truea", SyntaxError(TrailingCharacters, 1, 5)),
            ("falsea", SyntaxError(TrailingCharacters, 1, 6)),
        ]);

        test_parse_ok([
            ("true", true),
            ("false", false),
        ]);
    }

    #[test]
    fn test_json_deserialize_bool() {
        test_json_deserialize_ok([
            true,
            false,
        ]);
    }

    #[test]
    fn test_parse_number_errors() {
        test_parse_err::<f64>([
            ("+", SyntaxError(InvalidSyntax(SomeValue), 1, 1)),
            (".", SyntaxError(InvalidSyntax(SomeValue), 1, 1)),
            ("-", SyntaxError(InvalidNumber, 1, 2)),
            ("00", SyntaxError(InvalidNumber, 1, 2)),
            ("1.", SyntaxError(InvalidNumber, 1, 3)),
            ("1e", SyntaxError(InvalidNumber, 1, 3)),
            ("1e+", SyntaxError(InvalidNumber, 1, 4)),
            ("1a", SyntaxError(TrailingCharacters, 1, 2)),
        ]);
    }

    #[test]
    fn test_parse_i64() {
        test_parse_ok([
            ("3", 3i64),
            ("-2", -2),
            ("-1234", -1234),
        ]);
    }

    #[test]
    fn test_parse_f64() {
        test_parse_ok([
            ("3.0", 3.0f64),
            ("3.1", 3.1),
            ("-1.2", -1.2),
            ("0.4", 0.4),
            ("0.4e5", 0.4e5),
            ("0.4e15", 0.4e15),
            ("0.4e-01", 0.4e-01),
        ]);
    }

    #[test]
    fn test_json_deserialize_numbers() {
        test_json_deserialize_ok([
            3.0f64,
            3.1,
            -1.2,
            0.4,
            0.4e5,
            0.4e15,
            0.4e-01,
        ]);
    }

    #[test]
    fn test_parse_string() {
        test_parse_err::<string::String>([
            ("\"", SyntaxError(EOFWhileParsingString, 1, 2)),
            ("\"lol", SyntaxError(EOFWhileParsingString, 1, 5)),
            ("\"lol\"a", SyntaxError(TrailingCharacters, 1, 6)),
        ]);

        test_parse_ok([
            ("\"\"", "".to_string()),
            ("\"foo\"", "foo".to_string()),
            ("\"\\\"\"", "\"".to_string()),
            ("\"\\b\"", "\x08".to_string()),
            ("\"\\n\"", "\n".to_string()),
            ("\"\\r\"", "\r".to_string()),
            ("\"\\t\"", "\t".to_string()),
            ("\"\\u12ab\"", "\u12ab".to_string()),
            ("\"\\uAB12\"", "\uAB12".to_string()),
        ]);
    }

    #[test]
    fn test_json_deserialize_str() {
        test_json_deserialize_ok([
            "".to_string(),
            "foo".to_string(),
            "\"".to_string(),
            "\x08".to_string(),
            "\n".to_string(),
            "\r".to_string(),
            "\t".to_string(),
            "\u12ab".to_string(),
            "\uAB12".to_string(),
        ]);
    }

    #[test]
    fn test_parse_list() {
        test_parse_err::<Vec<f64>>([
            ("[", SyntaxError(EOFWhileParsingValue, 1, 2)),
            ("[ ", SyntaxError(EOFWhileParsingValue, 1, 3)),
            ("[1", SyntaxError(EOFWhileParsingList,  1, 3)),
            ("[1,", SyntaxError(EOFWhileParsingValue, 1, 4)),
            ("[1,]", SyntaxError(InvalidSyntax(SomeValue), 1, 4)),
            ("[1 2]", SyntaxError(InvalidSyntax(ListCommaOrEnd), 1, 4)),
            ("[]a", SyntaxError(TrailingCharacters, 1, 3)),
        ]);

        test_parse_ok([
            ("[]", vec!()),
            ("[ ]", vec!()),
            ("[null]", vec!(())),
            ("[ null ]", vec!(())),
        ]);

        test_parse_ok([
            ("[true]", vec!(true)),
        ]);

        test_parse_ok([
            ("[3,1]", vec!(3i, 1)),
            ("[ 3 , 1 ]", vec!(3i, 1)),
        ]);

        test_parse_ok([
            ("[[3], [1, 2]]", vec!(vec!(3i), vec!(1, 2))),
        ]);

        let v: () = from_str("[]").unwrap();
        assert_eq!(v, ());

        test_parse_ok([
            ("[1, 2, 3]", (1u, 2u, 3u)),
        ]);
    }

    #[test]
    fn test_json_deserialize_list() {
        test_json_deserialize_ok([
            vec!(),
            vec!(()),
        ]);

        test_json_deserialize_ok([
            vec!(true),
        ]);

        test_json_deserialize_ok([
            vec!(3i, 1),
        ]);

        test_json_deserialize_ok([
            vec!(vec!(3i), vec!(1, 2)),
        ]);
    }

    #[test]
    fn test_parse_object() {
        test_parse_err::<TreeMap<string::String, int>>([
            ("{", SyntaxError(EOFWhileParsingString, 1, 2)),
            ("{ ", SyntaxError(EOFWhileParsingString, 1, 3)),
            ("{1", SyntaxError(KeyMustBeAString, 1, 2)),
            ("{ \"a\"", SyntaxError(EOFWhileParsingObject, 1, 6)),
            ("{\"a\"", SyntaxError(EOFWhileParsingObject, 1, 5)),
            ("{\"a\" ", SyntaxError(EOFWhileParsingObject, 1, 6)),
            ("{\"a\" 1", SyntaxError(ExpectedColon, 1, 6)),
            ("{\"a\":", SyntaxError(EOFWhileParsingValue, 1, 6)),
            ("{\"a\":1", SyntaxError(EOFWhileParsingObject, 1, 7)),
            ("{\"a\":1 1", SyntaxError(InvalidSyntax(ObjectCommaOrEnd), 1, 8)),
            ("{\"a\":1,", SyntaxError(EOFWhileParsingString, 1, 8)),
            ("{}a", SyntaxError(TrailingCharacters, 1, 3)),
        ]);

        test_parse_ok([
            ("{}", treemap!()),
            ("{ }", treemap!()),
            (
                "{\"a\":3}",
                treemap!("a".to_string() => 3i)
            ),
            (
                "{ \"a\" : 3 }",
                treemap!("a".to_string() => 3i)
            ),
            (
                "{\"a\":3,\"b\":4}",
                treemap!("a".to_string() => 3i, "b".to_string() => 4)
            ),
            (
                "{ \"a\" : 3 , \"b\" : 4 }",
                treemap!("a".to_string() => 3i, "b".to_string() => 4),
            ),
        ]);

        test_parse_ok([
            (
                "{\"a\": {\"b\": 3, \"c\": 4}}",
                treemap!("a".to_string() => treemap!("b".to_string() => 3i, "c".to_string() => 4i)),
            ),
        ]);
    }

    #[test]
    fn test_json_deserialize_object() {
        test_json_deserialize_ok([
            treemap!(),
            treemap!("a".to_string() => 3i),
            treemap!("a".to_string() => 3i, "b".to_string() => 4),
        ]);

        test_json_deserialize_ok([
            treemap!("a".to_string() => treemap!("b".to_string() => 3i, "c".to_string() => 4)),
        ]);
    }

    #[test]
    fn test_parse_struct() {
        test_parse_ok([
            (
                "{
                    \"inner\": []
                }",
                Outer {
                    inner: vec![]
                },
            ),
            (
                "{
                    \"inner\": [
                        { \"a\": null, \"b\": 2, \"c\": [\"abc\", \"xyz\"] }
                    ]
                }",
                Outer {
                    inner: vec![
                        Inner { a: (), b: 2, c: vec!["abc".to_string(), "xyz".to_string()] }
                    ]
                },
            )
        ]);
    }

    #[test]
    fn test_json_deserialize_struct() {
        test_json_deserialize_ok([
            Outer {
                inner: vec![
                    Inner { a: (), b: 2, c: vec!["abc".to_string(), "xyz".to_string()] }
                ]
            },
        ]);
    }

    #[test]
    fn test_parse_option() {
        test_parse_ok([
            ("null", None),
            ("\"jodhpurs\"", Some("jodhpurs".to_string())),
        ]);

        #[deriving(PartialEq, Show)]
        #[deriving_serialize]
        #[deriving_deserialize]
        struct Foo {
            x: Option<int>,
        }

        let value: Foo = from_str("{}").unwrap();
        assert_eq!(value, Foo { x: None });

        let value: Foo = from_str("{ \"x\": 5 }").unwrap();
        assert_eq!(value, Foo { x: Some(5) });
    }

    #[test]
    fn test_json_deserialize_option() {
        test_json_deserialize_ok([
            None,
            Some("jodhpurs".to_string()),
        ]);
    }

    #[test]
    fn test_parse_enum() {
        test_parse_ok([
            ("{\"Dog\": []}", Dog),
            (
                "{\"Frog\": [\"Henry\", []]}",
                Frog("Henry".to_string(), vec!()),
            ),
            (
                "{\"Frog\": [\"Henry\", [349]]}",
                Frog("Henry".to_string(), vec!(349)),
            ),
            (
                "{\"Frog\": [\"Henry\", [349, 102]]}",
                Frog("Henry".to_string(), vec!(349, 102)),
            ),
        ]);

        test_parse_ok([
            (
                concat!(
                    "{",
                    "  \"a\": {\"Dog\": []},",
                    "  \"b\": {\"Frog\":[\"Henry\", []]}",
                    "}"
                ),
                treemap!(
                    "a".to_string() => Dog,
                    "b".to_string() => Frog("Henry".to_string(), vec!())
                )
            ),
        ]);
    }

    #[test]
    fn test_json_deserialize_enum() {
        test_json_deserialize_ok([
            Dog,
            Frog("Henry".to_string(), vec!()),
            Frog("Henry".to_string(), vec!(349)),
            Frog("Henry".to_string(), vec!(349, 102)),
        ]);
    }

    #[test]
    fn test_multiline_errors() {
        test_parse_err::<TreeMap<string::String, string::String>>([
            ("{\n  \"foo\":\n \"bar\"", SyntaxError(EOFWhileParsingObject, 3u, 8u)),
        ]);
    }

    /*
    #[deriving(Decodable)]
    struct DecodeStruct {
        x: f64,
        y: bool,
        z: String,
        w: Vec<DecodeStruct>
    }
    #[deriving(Decodable)]
    enum DecodeEnum {
        A(f64),
        B(String)
    }
    fn check_err<T: Decodable<Decoder, DecoderError>>(to_parse: &'static str,
                                                      expected: DecoderError) {
        let res: DecodeResult<T> = match from_str(to_parse) {
            Err(e) => Err(ParseError(e)),
            Ok(json) => Decodable::decode(&mut Decoder::new(json))
        };
        match res {
            Ok(_) => panic!("`{}` parsed & decoded ok, expecting error `{}`",
                              to_parse, expected),
            Err(ParseError(e)) => panic!("`{}` is not valid json: {}",
                                           to_parse, e),
            Err(e) => {
                assert_eq!(e, expected);
            }
        }
    }
    #[test]
    fn test_decode_errors_struct() {
        check_err::<DecodeStruct>("[]", ExpectedError("Object".to_string(), "[]".to_string()));
        check_err::<DecodeStruct>("{\"x\": true, \"y\": true, \"z\": \"\", \"w\": []}",
                                  ExpectedError("Number".to_string(), "true".to_string()));
        check_err::<DecodeStruct>("{\"x\": 1, \"y\": [], \"z\": \"\", \"w\": []}",
                                  ExpectedError("Boolean".to_string(), "[]".to_string()));
        check_err::<DecodeStruct>("{\"x\": 1, \"y\": true, \"z\": {}, \"w\": []}",
                                  ExpectedError("String".to_string(), "{}".to_string()));
        check_err::<DecodeStruct>("{\"x\": 1, \"y\": true, \"z\": \"\", \"w\": null}",
                                  ExpectedError("List".to_string(), "null".to_string()));
        check_err::<DecodeStruct>("{\"x\": 1, \"y\": true, \"z\": \"\"}",
                                  MissingFieldError("w".to_string()));
    }
    #[test]
    fn test_decode_errors_enum() {
        check_err::<DecodeEnum>("{}",
                                MissingFieldError("variant".to_string()));
        check_err::<DecodeEnum>("{\"variant\": 1}",
                                ExpectedError("String".to_string(), "1".to_string()));
        check_err::<DecodeEnum>("{\"variant\": \"A\"}",
                                MissingFieldError("fields".to_string()));
        check_err::<DecodeEnum>("{\"variant\": \"A\", \"fields\": null}",
                                ExpectedError("List".to_string(), "null".to_string()));
        check_err::<DecodeEnum>("{\"variant\": \"C\", \"fields\": []}",
                                UnknownVariantError("C".to_string()));
    }
    */

    #[test]
    fn test_find(){
        let json_value: Json = from_str("{\"dog\" : \"cat\"}").unwrap();
        let found_str = json_value.find(&"dog".to_string());
        assert!(found_str.is_some() && found_str.unwrap().as_string().unwrap() == "cat");
    }

    #[test]
    fn test_find_path(){
        let json_value: Json = from_str("{\"dog\":{\"cat\": {\"mouse\" : \"cheese\"}}}").unwrap();
        let found_str = json_value.find_path(&[&"dog".to_string(),
                                             &"cat".to_string(), &"mouse".to_string()]);
        assert!(found_str.is_some() && found_str.unwrap().as_string().unwrap() == "cheese");
    }

    #[test]
    fn test_search(){
        let json_value: Json = from_str("{\"dog\":{\"cat\": {\"mouse\" : \"cheese\"}}}").unwrap();
        let found_str = json_value.search(&"mouse".to_string()).and_then(|j| j.as_string());
        assert!(found_str.is_some());
        assert!(found_str.unwrap() == "cheese");
    }

    #[test]
    fn test_is_object() {
        let json_value: Json = from_str("{}").unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_as_object() {
        let json_value: Json = from_str("{}").unwrap();
        let json_object = json_value.as_object();
        let map = TreeMap::<string::String, Json>::new();
        assert_eq!(json_object, Some(&map));
    }

    #[test]
    fn test_is_list() {
        let json_value: Json = from_str("[1, 2, 3]").unwrap();
        assert!(json_value.is_list());
    }

    #[test]
    fn test_as_list() {
        let json_value: Json = from_str("[1, 2, 3]").unwrap();
        let json_list = json_value.as_list();
        let expected_length = 3;
        assert!(json_list.is_some() && json_list.unwrap().len() == expected_length);
    }

    #[test]
    fn test_is_string() {
        let json_value: Json = from_str("\"dog\"").unwrap();
        assert!(json_value.is_string());
    }

    #[test]
    fn test_as_string() {
        let json_value: Json = from_str("\"dog\"").unwrap();
        let json_str = json_value.as_string();
        let expected_str = "dog";
        assert_eq!(json_str, Some(expected_str));
    }

    #[test]
    fn test_is_number() {
        let json_value: Json = from_str("12").unwrap();
        assert!(json_value.is_number());

        let json_value: Json = from_str("12.0").unwrap();
        assert!(json_value.is_number());
    }

    #[test]
    fn test_is_i64() {
        let json_value: Json = from_str("12").unwrap();
        assert!(json_value.is_i64());

        let json_value: Json = from_str("12.0").unwrap();
        assert!(!json_value.is_i64());
    }

    #[test]
    fn test_is_f64() {
        let json_value: Json = from_str("12").unwrap();
        assert!(!json_value.is_f64());

        let json_value: Json = from_str("12.0").unwrap();
        assert!(json_value.is_f64());
    }

    #[test]
    fn test_as_i64() {
        let json_value: Json = from_str("12").unwrap();
        assert_eq!(json_value.as_i64(), Some(12));
    }

    #[test]
    fn test_as_f64() {
        let json_value: Json = from_str("12").unwrap();
        assert_eq!(json_value.as_f64(), Some(12.0));
    }

    #[test]
    fn test_is_boolean() {
        let json_value: Json = from_str("false").unwrap();
        assert!(json_value.is_boolean());
    }

    #[test]
    fn test_as_boolean() {
        let json_value: Json = from_str("false").unwrap();
        let json_bool = json_value.as_boolean();
        let expected_bool = false;
        assert!(json_bool.is_some() && json_bool.unwrap() == expected_bool);
    }

    #[test]
    fn test_is_null() {
        let json_value: Json = from_str("null").unwrap();
        assert!(json_value.is_null());
    }

    #[test]
    fn test_as_null() {
        let json_value: Json = from_str("null").unwrap();
        let json_null = json_value.as_null();
        let expected_null = ();
        assert!(json_null.is_some() && json_null.unwrap() == expected_null);
    }

    /*
    #[test]
    fn test_encode_hashmap_with_numeric_key() {
        use std::str::from_utf8;
        use std::io::MemWriter;
        use std::collections::HashMap;
        let mut hm: HashMap<uint, bool> = HashMap::new();
        hm.insert(1, true);
        let mut mem_buf = MemWriter::new();
        {
            let mut serializer = Serializer::new(&mut mem_buf as &mut Writer);
            hm.serialize(&mut serializer).unwrap();
        }
        let bytes = mem_buf.unwrap();
        let json_str = from_utf8(bytes.as_slice()).unwrap();
        let _json_value: Json = from_str(json_str).unwrap();
    }
    #[test]
    fn test_prettyencode_hashmap_with_numeric_key() {
        use std::str::from_utf8;
        use std::io::MemWriter;
        use std::collections::HashMap;
        let mut hm: HashMap<uint, bool> = HashMap::new();
        hm.insert(1, true);
        let mut mem_buf = MemWriter::new();
        {
            let mut serializer = PrettySerializer::new(&mut mem_buf as &mut Writer);
            hm.serialize(&mut serializer).unwrap()
        }
        let bytes = mem_buf.unwrap();
        let json_str = from_utf8(bytes.as_slice()).unwrap();
        let _json_value: Json = from_str(json_str).unwrap();
    }

    #[test]
    fn test_hashmap_with_numeric_key_can_handle_double_quote_delimited_key() {
        use std::collections::HashMap;
        let json_str = "{\"1\":true}";
        let map: HashMap<uint, bool> = from_str(json_str).unwrap();
        let mut m = HashMap::new();
        m.insert(1u, true);
        assert_eq!(map, m);
    }
    */

    /*
    fn assert_stream_equal(src: &str, expected: ~[(JsonEvent, ~[StackElement])]) {
        let mut parser = Parser::new(src.chars());
        let mut i = 0;
        loop {
            let evt = match parser.next() {
                Some(e) => e,
                None => { break; }
            };
            let (ref expected_evt, ref expected_stack) = expected[i];
            if !parser.stack().is_equal_to(expected_stack.as_slice()) {
                panic!("Parser stack is not equal to {}", expected_stack);
            }
            assert_eq!(&evt, expected_evt);
            i+=1;
        }
    }
    #[test]
    fn test_streaming_parser() {
        assert_stream_equal(
            r#"{ "foo":"bar", "array" : [0, 1, 2,3 ,4,5], "idents":[null,true,false]}"#,
            ~[
                (ObjectStart,             ~[]),
                  (StringValue("bar".to_string()),   ~[Key("foo")]),
                  (ListStart,             ~[Key("array")]),
                    (NumberValue(0.0),    ~[Key("array"), Index(0)]),
                    (NumberValue(1.0),    ~[Key("array"), Index(1)]),
                    (NumberValue(2.0),    ~[Key("array"), Index(2)]),
                    (NumberValue(3.0),    ~[Key("array"), Index(3)]),
                    (NumberValue(4.0),    ~[Key("array"), Index(4)]),
                    (NumberValue(5.0),    ~[Key("array"), Index(5)]),
                  (ListEnd,               ~[Key("array")]),
                  (ListStart,             ~[Key("idents")]),
                    (NullValue,           ~[Key("idents"), Index(0)]),
                    (BooleanValue(true),  ~[Key("idents"), Index(1)]),
                    (BooleanValue(false), ~[Key("idents"), Index(2)]),
                  (ListEnd,               ~[Key("idents")]),
                (ObjectEnd,               ~[]),
            ]
        );
    }
    fn last_event(src: &str) -> JsonEvent {
        let mut parser = Parser::new(src.chars());
        let mut evt = NullValue;
        loop {
            evt = match parser.next() {
                Some(e) => e,
                None => return evt,
            }
        }
    }
    #[test]
    #[ignore(cfg(target_word_size = "32"))] // FIXME(#14064)
    fn test_read_object_streaming() {
        assert_eq!(last_event("{ "),      Error(SyntaxError(EOFWhileParsingObject, 1, 3)));
        assert_eq!(last_event("{1"),      Error(SyntaxError(KeyMustBeAString,      1, 2)));
        assert_eq!(last_event("{ \"a\""), Error(SyntaxError(EOFWhileParsingObject, 1, 6)));
        assert_eq!(last_event("{\"a\""),  Error(SyntaxError(EOFWhileParsingObject, 1, 5)));
        assert_eq!(last_event("{\"a\" "), Error(SyntaxError(EOFWhileParsingObject, 1, 6)));

        assert_eq!(last_event("{\"a\" 1"),   Error(SyntaxError(ExpectedColon,         1, 6)));
        assert_eq!(last_event("{\"a\":"),    Error(SyntaxError(EOFWhileParsingValue,  1, 6)));
        assert_eq!(last_event("{\"a\":1"),   Error(SyntaxError(EOFWhileParsingObject, 1, 7)));
        assert_eq!(last_event("{\"a\":1 1"), Error(SyntaxError(InvalidSyntax,         1, 8)));
        assert_eq!(last_event("{\"a\":1,"),  Error(SyntaxError(EOFWhileParsingObject, 1, 8)));

        assert_stream_equal(
            "{}",
            box [(ObjectStart, box []), (ObjectEnd, box [])]
        );
        assert_stream_equal(
            "{\"a\": 3}",
            box [
                (ObjectStart,        box []),
                  (F64Value(3.0), box [Key("a")]),
                (ObjectEnd,          box []),
            ]
        );
        assert_stream_equal(
            "{ \"a\": null, \"b\" : true }",
            box [
                (ObjectStart,           box []),
                  (NullValue,           box [Key("a")]),
                  (BooleanValue(true),  box [Key("b")]),
                (ObjectEnd,             box []),
            ]
        );
        assert_stream_equal(
            "{\"a\" : 1.0 ,\"b\": [ true ]}",
            box [
                (ObjectStart,           box []),
                  (F64Value(1.0),    box [Key("a")]),
                  (ListStart,           box [Key("b")]),
                    (BooleanValue(true),box [Key("b"), Index(0)]),
                  (ListEnd,             box [Key("b")]),
                (ObjectEnd,             box []),
            ]
        );
        assert_stream_equal(
            r#"{
                "a": 1.0,
                "b": [
                    true,
                    "foo\nbar",
                    { "c": {"d": null} }
                ]
            }"#,
            ~[
                (ObjectStart,                   ~[]),
                  (F64Value(1.0),            ~[Key("a")]),
                  (ListStart,                   ~[Key("b")]),
                    (BooleanValue(true),        ~[Key("b"), Index(0)]),
                    (StringValue("foo\nbar".to_string()),  ~[Key("b"), Index(1)]),
                    (ObjectStart,               ~[Key("b"), Index(2)]),
                      (ObjectStart,             ~[Key("b"), Index(2), Key("c")]),
                        (NullValue,             ~[Key("b"), Index(2), Key("c"), Key("d")]),
                      (ObjectEnd,               ~[Key("b"), Index(2), Key("c")]),
                    (ObjectEnd,                 ~[Key("b"), Index(2)]),
                  (ListEnd,                     ~[Key("b")]),
                (ObjectEnd,                     ~[]),
            ]
        );
    }
    #[test]
    #[ignore(cfg(target_word_size = "32"))] // FIXME(#14064)
    fn test_read_list_streaming() {
        assert_stream_equal(
            "[]",
            box [
                (ListStart, box []),
                (ListEnd,   box []),
            ]
        );
        assert_stream_equal(
            "[ ]",
            box [
                (ListStart, box []),
                (ListEnd,   box []),
            ]
        );
        assert_stream_equal(
            "[true]",
            box [
                (ListStart,              box []),
                    (BooleanValue(true), box [Index(0)]),
                (ListEnd,                box []),
            ]
        );
        assert_stream_equal(
            "[ false ]",
            box [
                (ListStart,               box []),
                    (BooleanValue(false), box [Index(0)]),
                (ListEnd,                 box []),
            ]
        );
        assert_stream_equal(
            "[null]",
            box [
                (ListStart,     box []),
                    (NullValue, box [Index(0)]),
                (ListEnd,       box []),
            ]
        );
        assert_stream_equal(
            "[3, 1]",
            box [
                (ListStart,     box []),
                    (F64Value(3.0), box [Index(0)]),
                    (F64Value(1.0), box [Index(1)]),
                (ListEnd,       box []),
            ]
        );
        assert_stream_equal(
            "\n[3, 2]\n",
            box [
                (ListStart,     box []),
                    (F64Value(3.0), box [Index(0)]),
                    (F64Value(2.0), box [Index(1)]),
                (ListEnd,       box []),
            ]
        );
        assert_stream_equal(
            "[2, [4, 1]]",
            box [
                (ListStart,                 box []),
                    (F64Value(2.0),      box [Index(0)]),
                    (ListStart,             box [Index(1)]),
                        (F64Value(4.0),  box [Index(1), Index(0)]),
                        (F64Value(1.0),  box [Index(1), Index(1)]),
                    (ListEnd,               box [Index(1)]),
                (ListEnd,                   box []),
            ]
        );

        assert_eq!(last_event("["), Error(SyntaxError(EOFWhileParsingValue, 1,  2)));

        assert_eq!(from_str("["),     Err(SyntaxError(EOFWhileParsingValue, 1, 2)));
        assert_eq!(from_str("[1"),    Err(SyntaxError(EOFWhileParsingList,  1, 3)));
        assert_eq!(from_str("[1,"),   Err(SyntaxError(EOFWhileParsingValue, 1, 4)));
        assert_eq!(from_str("[1,]"),  Err(SyntaxError(InvalidSyntax,        1, 4)));
        assert_eq!(from_str("[6 7]"), Err(SyntaxError(InvalidSyntax,        1, 4)));

    }
    #[test]
    fn test_trailing_characters_streaming() {
        assert_eq!(last_event("nulla"),  Error(SyntaxError(TrailingCharacters, 1, 5)));
        assert_eq!(last_event("truea"),  Error(SyntaxError(TrailingCharacters, 1, 5)));
        assert_eq!(last_event("falsea"), Error(SyntaxError(TrailingCharacters, 1, 6)));
        assert_eq!(last_event("1a"),     Error(SyntaxError(TrailingCharacters, 1, 2)));
        assert_eq!(last_event("[]a"),    Error(SyntaxError(TrailingCharacters, 1, 3)));
        assert_eq!(last_event("{}a"),    Error(SyntaxError(TrailingCharacters, 1, 3)));
    }
    #[test]
    fn test_read_identifiers_streaming() {
        assert_eq!(Parser::new("null".chars()).next(), Some(NullValue));
        assert_eq!(Parser::new("true".chars()).next(), Some(BooleanValue(true)));
        assert_eq!(Parser::new("false".chars()).next(), Some(BooleanValue(false)));

        assert_eq!(last_event("n"),    Error(SyntaxError(InvalidSyntax, 1, 2)));
        assert_eq!(last_event("nul"),  Error(SyntaxError(InvalidSyntax, 1, 4)));
        assert_eq!(last_event("t"),    Error(SyntaxError(InvalidSyntax, 1, 2)));
        assert_eq!(last_event("truz"), Error(SyntaxError(InvalidSyntax, 1, 4)));
        assert_eq!(last_event("f"),    Error(SyntaxError(InvalidSyntax, 1, 2)));
        assert_eq!(last_event("faz"),  Error(SyntaxError(InvalidSyntax, 1, 3)));
    }

    #[test]
    fn test_stack() {
        let mut stack = Stack::new();

        assert!(stack.is_empty());
        assert!(stack.len() == 0);
        assert!(!stack.last_is_index());

        stack.push_index(0);
        stack.bump_index();

        assert!(stack.len() == 1);
        assert!(stack.is_equal_to([Index(1)]));
        assert!(stack.starts_with([Index(1)]));
        assert!(stack.ends_with([Index(1)]));
        assert!(stack.last_is_index());
        assert!(stack.get(0) == Index(1));

        stack.push_key("foo".to_string());

        assert!(stack.len() == 2);
        assert!(stack.is_equal_to([Index(1), Key("foo")]));
        assert!(stack.starts_with([Index(1), Key("foo")]));
        assert!(stack.starts_with([Index(1)]));
        assert!(stack.ends_with([Index(1), Key("foo")]));
        assert!(stack.ends_with([Key("foo")]));
        assert!(!stack.last_is_index());
        assert!(stack.get(0) == Index(1));
        assert!(stack.get(1) == Key("foo"));

        stack.push_key("bar".to_string());

        assert!(stack.len() == 3);
        assert!(stack.is_equal_to([Index(1), Key("foo"), Key("bar")]));
        assert!(stack.starts_with([Index(1)]));
        assert!(stack.starts_with([Index(1), Key("foo")]));
        assert!(stack.starts_with([Index(1), Key("foo"), Key("bar")]));
        assert!(stack.ends_with([Key("bar")]));
        assert!(stack.ends_with([Key("foo"), Key("bar")]));
        assert!(stack.ends_with([Index(1), Key("foo"), Key("bar")]));
        assert!(!stack.last_is_index());
        assert!(stack.get(0) == Index(1));
        assert!(stack.get(1) == Key("foo"));
        assert!(stack.get(2) == Key("bar"));

        stack.pop();

        assert!(stack.len() == 2);
        assert!(stack.is_equal_to([Index(1), Key("foo")]));
        assert!(stack.starts_with([Index(1), Key("foo")]));
        assert!(stack.starts_with([Index(1)]));
        assert!(stack.ends_with([Index(1), Key("foo")]));
        assert!(stack.ends_with([Key("foo")]));
        assert!(!stack.last_is_index());
        assert!(stack.get(0) == Index(1));
        assert!(stack.get(1) == Key("foo"));
    }
*/
}

#[cfg(test)]
mod bench {
    use std::collections::TreeMap;
    use std::string;
    use serialize;
    use test::Bencher;

    use super::{Json, Null, Boolean, Integer, Floating, String, List, Object};
    use super::{Parser, from_str};
    use de;

    macro_rules! treemap {
        ($($k:expr => $v:expr),*) => ({
            let mut _m = ::std::collections::TreeMap::new();
            $(_m.insert($k, $v);)*
            _m
        })
    }

    fn json_str(count: uint) -> string::String {
        let mut src = "[".to_string();
        for _ in range(0, count) {
            src.push_str(r#"{"a":true,"b":null,"c":3.1415,"d":"Hello world","e":[1,2,3]},"#);
        }
        src.push_str("{}]");
        src
    }

    fn pretty_json_str(count: uint) -> string::String {
        let mut src = "[\n".to_string();
        for _ in range(0, count) {
            src.push_str(
                concat!(
                    "  {\n",
                    "    \"a\": true,\n",
                    "    \"b\": null,\n",
                    "    \"c\": 3.1415,\n",
                    "    \"d\": \"Hello world\",\n",
                    "    \"e\": [\n",
                    "      1,\n",
                    "      2,\n",
                    "      3\n",
                    "    ]\n",
                    "  },\n"
                )
            );
        }
        src.push_str("  {}\n]");
        src
    }

    fn encoder_json(count: uint) -> serialize::json::Json {
        use serialize::json;

        let mut list = vec!();
        for _ in range(0, count) {
            list.push(json::Object(treemap!(
                "a".to_string() => json::Boolean(true),
                "b".to_string() => json::Null,
                "c".to_string() => json::F64(3.1415),
                "d".to_string() => json::String("Hello world".to_string()),
                "e".to_string() => json::List(vec!(
                    json::U64(1),
                    json::U64(2),
                    json::U64(3)
                ))
            )));
        }
        list.push(json::Object(TreeMap::new()));
        json::List(list)
    }

    fn serializer_json(count: uint) -> Json {
        let mut list = vec!();
        for _ in range(0, count) {
            list.push(Object(treemap!(
                "a".to_string() => Boolean(true),
                "b".to_string() => Null,
                "c".to_string() => Floating(3.1415),
                "d".to_string() => String("Hello world".to_string()),
                "e".to_string() => List(vec!(
                    Integer(1),
                    Integer(2),
                    Integer(3)
                ))
            )));
        }
        list.push(Object(TreeMap::new()));
        List(list)
    }

    fn bench_encoder(b: &mut Bencher, count: uint) {
        let src = json_str(count);
        let json = encoder_json(count);

        b.iter(|| {
            assert_eq!(json.to_string(), src);
        });
    }

    fn bench_encoder_pretty(b: &mut Bencher, count: uint) {
        let src = pretty_json_str(count);
        let json = encoder_json(count);

        b.iter(|| {
            assert_eq!(json.to_pretty_str(), src);
        });
    }

    fn bench_serializer(b: &mut Bencher, count: uint) {
        let src = json_str(count);
        let json = serializer_json(count);

        b.iter(|| {
            assert_eq!(json.to_string(), src);
        });
    }

    fn bench_serializer_pretty(b: &mut Bencher, count: uint) {
        let src = pretty_json_str(count);
        let json = serializer_json(count);

        b.iter(|| {
            assert_eq!(json.to_pretty_string(), src);
        });
    }

    fn bench_decoder(b: &mut Bencher, count: uint) {
        let src = json_str(count);
        let json = encoder_json(count);
        b.iter(|| {
            assert_eq!(json, serialize::json::from_str(src.as_slice()).unwrap());
        });
    }

    fn bench_deserializer(b: &mut Bencher, count: uint) {
        let src = json_str(count);
        let json = encoder_json(count);
        b.iter(|| {
            assert_eq!(json, serialize::json::from_str(src.as_slice()).unwrap());
        });
    }

    fn bench_decoder_streaming(b: &mut Bencher, count: uint) {
        use serialize::json;

        let src = json_str(count);

        b.iter( || {
            let mut parser = json::Parser::new(src.as_slice().chars());
            assert_eq!(parser.next(), Some(json::ListStart));
            for _ in range(0, count) {
                assert_eq!(parser.next(), Some(json::ObjectStart));

                assert_eq!(parser.next(), Some(json::BooleanValue(true)));
                assert_eq!(parser.stack().top(), Some(json::Key("a")));

                assert_eq!(parser.next(), Some(json::NullValue));
                assert_eq!(parser.stack().top(), Some(json::Key("b")));

                assert_eq!(parser.next(), Some(json::F64Value(3.1415)));
                assert_eq!(parser.stack().top(), Some(json::Key("c")));

                assert_eq!(parser.next(), Some(json::StringValue("Hello world".to_string())));
                assert_eq!(parser.stack().top(), Some(json::Key("d")));

                assert_eq!(parser.next(), Some(json::ListStart));
                assert_eq!(parser.stack().top(), Some(json::Key("e")));
                assert_eq!(parser.next(), Some(json::U64Value(1)));
                assert_eq!(parser.next(), Some(json::U64Value(2)));
                assert_eq!(parser.next(), Some(json::U64Value(3)));
                assert_eq!(parser.next(), Some(json::ListEnd));

                assert_eq!(parser.next(), Some(json::ObjectEnd));
            }
            assert_eq!(parser.next(), Some(json::ObjectStart));
            assert_eq!(parser.next(), Some(json::ObjectEnd));
            assert_eq!(parser.next(), Some(json::ListEnd));
            assert_eq!(parser.next(), None);
        });
    }

    fn bench_deserializer_streaming(b: &mut Bencher, count: uint) {
        let src = json_str(count);

        b.iter( || {
            let mut parser = Parser::new(src.as_slice().bytes());

            assert_eq!(parser.next(), Some(Ok(de::SeqStart(0))));
            for _ in range(0, count) {
                assert_eq!(parser.next(), Some(Ok(de::MapStart(0))));

                assert_eq!(parser.next(), Some(Ok(de::String("a".to_string()))));
                assert_eq!(parser.next(), Some(Ok(de::Bool(true))));

                assert_eq!(parser.next(), Some(Ok(de::String("b".to_string()))));
                assert_eq!(parser.next(), Some(Ok(de::Null)));

                assert_eq!(parser.next(), Some(Ok(de::String("c".to_string()))));
                assert_eq!(parser.next(), Some(Ok(de::F64(3.1415))));

                assert_eq!(parser.next(), Some(Ok(de::String("d".to_string()))));
                assert_eq!(parser.next(), Some(Ok(de::String("Hello world".to_string()))));

                assert_eq!(parser.next(), Some(Ok(de::String("e".to_string()))));
                assert_eq!(parser.next(), Some(Ok(de::SeqStart(0))));
                assert_eq!(parser.next(), Some(Ok(de::I64(1))));
                assert_eq!(parser.next(), Some(Ok(de::I64(2))));
                assert_eq!(parser.next(), Some(Ok(de::I64(3))));
                assert_eq!(parser.next(), Some(Ok(de::End)));

                assert_eq!(parser.next(), Some(Ok(de::End)));
            }
            assert_eq!(parser.next(), Some(Ok(de::MapStart(0))));
            assert_eq!(parser.next(), Some(Ok(de::End)));
            assert_eq!(parser.next(), Some(Ok(de::End)));
            assert_eq!(parser.next(), None);

            loop {
                match parser.next() {
                    None => return,
                    Some(Ok(_)) => { }
                    Some(Err(err)) => { panic!("error: {}", err); }
                }
            }
        });
    }

    #[bench]
    fn bench_encoder_001(b: &mut Bencher) {
        bench_encoder(b, 1)
    }

    #[bench]
    fn bench_encoder_500(b: &mut Bencher) {
        bench_encoder(b, 500)
    }

    #[bench]
    fn bench_encoder_001_pretty(b: &mut Bencher) {
        bench_encoder_pretty(b, 1)
    }

    #[bench]
    fn bench_encoder_500_pretty(b: &mut Bencher) {
        bench_encoder_pretty(b, 500)
    }

    #[bench]
    fn bench_serializer_001(b: &mut Bencher) {
        bench_serializer(b, 1)
    }

    #[bench]
    fn bench_serializer_500(b: &mut Bencher) {
        bench_serializer(b, 500)
    }
    #[bench]
    fn bench_serializer_001_pretty(b: &mut Bencher) {
        bench_serializer_pretty(b, 1)
    }

    #[bench]
    fn bench_serializer_500_pretty(b: &mut Bencher) {
        bench_serializer_pretty(b, 500)
    }

    #[bench]
    fn bench_decoder_001(b: &mut Bencher) {
        bench_decoder(b, 1)
    }

    #[bench]
    fn bench_decoder_500(b: &mut Bencher) {
        bench_decoder(b, 500)
    }

    #[bench]
    fn bench_deserializer_001(b: &mut Bencher) {
        bench_deserializer(b, 1)
    }

    #[bench]
    fn bench_deserializer_500(b: &mut Bencher) {
        bench_deserializer(b, 500)
    }

    #[bench]
    fn bench_decoder_001_streaming(b: &mut Bencher) {
        bench_decoder_streaming(b, 1)
    }

    #[bench]
    fn bench_decoder_500_streaming(b: &mut Bencher) {
        bench_decoder_streaming(b, 500)
    }

    #[bench]
    fn bench_deserializer_001_streaming(b: &mut Bencher) {
        bench_deserializer_streaming(b, 1)
    }

    #[bench]
    fn bench_deserializer_500_streaming(b: &mut Bencher) {
        bench_deserializer_streaming(b, 500)
    }
}
