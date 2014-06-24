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
#![allow(missing_doc)]

/*!
JSON parsing and serialization

# What is JSON?

JSON (JavaScript Object Notation) is a way to write data in Javascript.
Like XML it allows one to encode structured data in a text format that can be read by humans easily.
Its native compatibility with JavaScript and its simple syntax make it used widely.

Json data are encoded in a form of "key":"value".
Data types that can be encoded are JavaScript types :
boolean (`true` or `false`), number (`f64`), string, array, object, null.
An object is a series of string keys mapping to values, in `"key": value` format.
Arrays are enclosed in square brackets ([ ... ]) and objects in curly brackets ({ ... }).
A simple JSON document encoding a person, his/her age, address and phone numbers could look like:

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

# Rust Type-based Encoding and Decoding

Rust provides a mechanism for low boilerplate encoding & decoding
of values to and from JSON via the serialization API.
To be able to encode a piece of data, it must implement the `serialize::Encodable` trait.
To be able to decode a piece of data, it must implement the `serialize::Decodable` trait.
The Rust compiler provides an annotation to automatically generate
the code for these traits: `#[deriving(Decodable, Encodable)]`

To encode using Encodable :

```rust
use std::io;
use serialize::{json, Encodable};

 #[deriving(Encodable)]
 pub struct TestStruct   {
    data_str: String,
 }

fn main() {
    let to_encode_object = TestStruct{data_str:"example of string to encode".to_string()};
    let mut m = io::MemWriter::new();
    {
        let mut serializer = json::Serializer::new(m.by_ref());
        match to_encode_object.encode(&mut serializer) {
            Ok(()) => (),
            Err(e) => fail!("json encoding error: {}", e)
        };
    }
}
```

Two wrapper functions are provided to encode a Encodable object
into a string (String) or buffer (~[u8]): `str_encode(&m)` and `buffer_encode(&m)`.

```rust
use serialize::json;
let to_encode_object = "example of string to encode".to_string();
let encoded_str: String = json::Serializer::str_encode(&to_encode_object);
```

JSON API provide an enum `json::Json` and a trait `ToJson` to encode object.
The trait `ToJson` encode object into a container `json::Json` and the API provide writer
to encode them into a stream or a string ...

When using `ToJson` the `Encodable` trait implementation is not mandatory.

A basic `ToJson` example using a TreeMap of attribute name / attribute value:


```rust
use std::collections::TreeMap;
use serialize::json;
use serialize::json::ToJson;

pub struct MyStruct  {
    attr1: u8,
    attr2: String,
}

impl ToJson for MyStruct {
    fn to_json( &self ) -> json::Json {
        let mut d = box TreeMap::new();
        d.insert("attr1".to_string(), self.attr1.to_json());
        d.insert("attr2".to_string(), self.attr2.to_json());
        json::Object(d)
    }
}

fn main() {
    let test2: MyStruct = MyStruct {attr1: 1, attr2:"test".to_string()};
    let tjson: json::Json = test2.to_json();
    let json_str: String = tjson.to_str().into_string();
}
```

To decode a JSON string using `Decodable` trait :

```rust
extern crate serialize;
use serialize::{json, Decodable};

#[deriving(Decodable)]
pub struct MyStruct  {
     attr1: u8,
     attr2: String,
}

fn main() {
    let json_str_to_decode: String =
            "{\"attr1\":1,\"attr2\":\"toto\"}".to_string();
    let json_object = json::from_str(json_str_to_decode.as_slice());
    let mut decoder = json::Decoder::new(json_object.unwrap());
    let decoded_object: MyStruct = match Decodable::decode(&mut decoder) {
        Ok(v) => v,
        Err(e) => fail!("Decoding error: {}", e)
    }; // create the final object
}
```

# Examples of use

## Using Autoserialization

Create a struct called TestStruct1 and serialize and deserialize it to and from JSON
using the serialization API, using the derived serialization code.

```rust
extern crate serialize;
use serialize::{json, Encodable, Decodable};

 #[deriving(Decodable, Encodable)] //generate Decodable, Encodable impl.
 pub struct TestStruct1  {
    data_int: u8,
    data_str: String,
    data_vector: Vec<u8>,
 }

// To serialize use the `json::str_encode` to encode an object in a string.
// It calls the generated `Encodable` impl.
fn main() {
    let to_encode_object = TestStruct1
         {data_int: 1, data_str:"toto".to_string(), data_vector:vec![2,3,4,5]};
    let encoded_str: String = json::Serializer::str_encode(&to_encode_object);

    // To deserialize use the `json::from_str` and `json::Decoder`

    let json_object = json::from_str(encoded_str.as_slice());
    let mut decoder = json::Decoder::new(json_object.unwrap());
    let decoded1: TestStruct1 = Decodable::decode(&mut decoder).unwrap(); // create the final object
}
```

## Using `ToJson`

This example use the ToJson impl to deserialize the JSON string.
Example of `ToJson` trait implementation for TestStruct1.

```rust
use std::collections::TreeMap;
use serialize::json::ToJson;
use serialize::{json, Encodable, Decodable};

#[deriving(Decodable, Encodable)] // generate Decodable, Encodable impl.
pub struct TestStruct1  {
    data_int: u8,
    data_str: String,
    data_vector: Vec<u8>,
}

impl ToJson for TestStruct1 {
    fn to_json( &self ) -> json::Json {
        let mut d = box TreeMap::new();
        d.insert("data_int".to_string(), self.data_int.to_json());
        d.insert("data_str".to_string(), self.data_str.to_json());
        d.insert("data_vector".to_string(), self.data_vector.to_json());
        json::Object(d)
    }
}

fn main() {
    // Serialization using our impl of to_json

    let test2: TestStruct1 = TestStruct1 {data_int: 1, data_str:"toto".to_string(),
                                          data_vector:vec![2,3,4,5]};
    let tjson: json::Json = test2.to_json();
    let json_str: String = tjson.to_str().into_string();

    // Deserialize like before.

    let mut decoder =
        json::Decoder::new(json::from_str(json_str.as_slice()).unwrap());
    // create the final object
    let decoded2: TestStruct1 = Decodable::decode(&mut decoder).unwrap();
}
```

*/

use std::char;
use std::collections::{HashMap, TreeMap};
use std::collections::treemap;
use std::fmt;
use std::io::MemWriter;
use std::io;
use std::num;
use std::str::ScalarValue;
use std::str;
use std::string::String;
use std::vec::Vec;
use std::vec;

use de;
use ser::Serializable;
use ser;

/// Represents a json value
#[deriving(Clone, PartialEq)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    List(List),
    Object(Object),
}

pub type List = Vec<Json>;
pub type Object = TreeMap<String, Json>;

impl Json {
    /// Encodes a json value into an io::writer.  Uses a single line.
    pub fn to_writer<W: Writer>(&self, wr: W) -> EncodeResult {
        let mut serializer = Serializer::new(wr);
        self.serialize(&mut serializer)
    }

    /// Encodes a json value into an io::writer.
    /// Pretty-prints in a more readable format.
    pub fn to_pretty_writer<W: Writer>(&self, wr: W) -> EncodeResult {
        let mut serializer = PrettySerializer::new(wr);
        self.serialize(&mut serializer)
    }

    /// Encodes a json value into a string
    pub fn to_pretty_str(&self) -> String {
        let mut wr = MemWriter::new();
        self.to_pretty_writer(wr.by_ref()).unwrap();
        str::from_utf8(wr.unwrap().as_slice()).unwrap().to_string()
    }

     /// If the Json value is an Object, returns the value associated with the provided key.
    /// Otherwise, returns None.
    pub fn find<'a>(&'a self, key: &String) -> Option<&'a Json>{
        match self {
            &Object(ref map) => map.find(key),
            _ => None
        }
    }

    /// Attempts to get a nested Json Object for each key in `keys`.
    /// If any key is found not to exist, find_path will return None.
    /// Otherwise, it will return the Json value associated with the final key.
    pub fn find_path<'a>(&'a self, keys: &[&String]) -> Option<&'a Json>{
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
    pub fn search<'a>(&'a self, key: &String) -> Option<&'a Json> {
        match self {
            &Object(ref map) => {
                match map.find(key) {
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
    pub fn as_object<'a>(&'a self) -> Option<&'a Object> {
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
    pub fn as_list<'a>(&'a self) -> Option<&'a List> {
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

    /// Returns true if the Json value is a Number. Returns false otherwise.
    pub fn is_number(&self) -> bool {
        self.as_number().is_some()
    }

    /// If the Json value is a Number, returns the associated f64.
    /// Returns None otherwise.
    pub fn as_number(&self) -> Option<f64> {
        match *self {
            Number(n) => Some(n),
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

impl fmt::Show for Json {
    /// Encodes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_writer(f as &mut Writer).map_err(|_| fmt::WriteError)
    }
}

impl ser::Serializable for Json {
    #[inline]
    fn serialize<
        S: ser::Serializer<E>,
        E
    >(&self, s: &mut S) -> Result<(), E> {
        match *self {
            Null => {
                ().serialize(s)
            }
            Boolean(v) => {
                v.serialize(s)
            }
            Number(v) => {
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

impl de::Deserializable for Json {
    #[inline]
    fn deserialize_token<
        D: de::Deserializer<E>,
        E
    >(d: &mut D, token: de::Token) -> Result<Json, E> {
        match token {
            de::Null => Ok(Null),
            de::Bool(x) => Ok(Boolean(x)),
            de::Int(x) => Ok(Number(x as f64)),
            de::I8(x) => Ok(Number(x as f64)),
            de::I16(x) => Ok(Number(x as f64)),
            de::I32(x) => Ok(Number(x as f64)),
            de::I64(x) => Ok(Number(x as f64)),
            de::Uint(x) => Ok(Number(x as f64)),
            de::U8(x) => Ok(Number(x as f64)),
            de::U16(x) => Ok(Number(x as f64)),
            de::U32(x) => Ok(Number(x as f64)),
            de::U64(x) => Ok(Number(x as f64)),
            de::F32(x) => Ok(Number(x as f64)),
            de::F64(x) => Ok(Number(x)),
            de::Char(x) => Ok(String(x.to_str())),
            de::Str(x) => Ok(String(x.to_str())),
            de::String(x) => Ok(String(x)),
            de::Option(false) => Ok(Null),
            de::Option(true) => de::Deserializable::deserialize(d),
            de::TupleStart(_) | de::SeqStart(_) => {
                let list = try!(de::Deserializable::deserialize_token(d, token));
                Ok(List(list))
            }
            de::StructStart(_, _) | de::MapStart(_) => {
                let object = try!(de::Deserializable::deserialize_token(d, token));
                Ok(Object(object))
            }
            de::EnumStart(_, name, len) => {
                let token = de::SeqStart(len);
                let fields: Vec<Json> = try!(de::Deserializable::deserialize_token(d, token));
                let mut object = TreeMap::new();
                object.insert(name.to_string(), List(fields));
                Ok(Object(object))
            }
            de::End => d.syntax_error(),
        }
    }
}

enum JsonDeserializerState {
    JsonDeserializerValueState(Json),
    JsonDeserializerListState(vec::MoveItems<Json>),
    JsonDeserializerObjectState(treemap::MoveEntries<String, Json>),
    JsonDeserializerEndState,
}

pub struct JsonDeserializer {
    stack: Vec<JsonDeserializerState>,
}

impl JsonDeserializer {
    /// Creates a new decoder instance for decoding the specified JSON value.
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
                        Number(x) => de::F64(x),
                        String(x) => de::String(x),
                        List(x) => {
                            let len = x.len();
                            self.stack.push(JsonDeserializerListState(x.move_iter()));
                            de::SeqStart(len)
                        }
                        Object(x) => {
                            let len = x.len();
                            self.stack.push(JsonDeserializerObjectState(x.move_iter()));
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
    fn end_of_stream_error<T>(&self) -> Result<T, ParserError> {
        Err(SyntaxError(EOFWhileParsingValue, 0, 0))
    }

    fn syntax_error<T>(&self) -> Result<T, ParserError> {
        Err(SyntaxError(InvalidSyntax, 0, 0))
    }

    // Special case treating options as a nullable value.
    #[inline]
    fn expect_option<
        U: de::Deserializable
    >(&mut self, token: de::Token) -> Result<Option<U>, ParserError> {
        match token {
            de::Null => Ok(None),
            token => {
                let value: U = try!(de::Deserializable::deserialize_token(self, token));
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
                    None => { fail!("state machine error, state stack empty"); }
                };

                let mut iter = match state {
                    JsonDeserializerObjectState(iter) => iter,
                    _ => { fail!("state machine error, expected an object"); }
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

                for field in fields.move_iter().rev() {
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
}

/// The errors that can arise while parsing a JSON stream.
#[deriving(Clone, PartialEq)]
pub enum ErrorCode {
    EOFWhileParsingList,
    EOFWhileParsingObject,
    EOFWhileParsingString,
    EOFWhileParsingValue,
    ExpectedColon,
    InvalidEscape,
    InvalidNumber,
    InvalidSyntax,
    InvalidUnicodeCodePoint,
    KeyMustBeAString,
    LoneLeadingSurrogateInHexEscape,
    MissingField,
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
    ExpectedError(String, String),
    MissingFieldError(String),
    UnknownVariantError(String),
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

/// Returns a readable error string for a given error code.
pub fn error_str(error: ErrorCode) -> &'static str {
    return match error {
        EOFWhileParsingList => "EOF While parsing list",
        EOFWhileParsingObject => "EOF While parsing object",
        EOFWhileParsingString => "EOF While parsing string",
        EOFWhileParsingValue => "EOF While parsing value",
        ExpectedColon => "expected `:`",
        InvalidEscape => "invalid escape",
        InvalidNumber => "invalid number",
        InvalidSyntax => "invalid syntax",
        InvalidUnicodeCodePoint => "invalid unicode code point",
        KeyMustBeAString => "key must be a string",
        LoneLeadingSurrogateInHexEscape => "lone leading surrogate in hex escape",
        MissingField => "missing variant",
        NotFourDigit => "invalid \\u escape (not four digits)",
        NotUtf8 => "contents not utf-8",
        TrailingCharacters => "trailing characters",
        UnexpectedEndOfHexEscape => "unexpected end of hex escape",
        UnknownVariant => "unknown variant",
        UnrecognizedHex => "invalid \\u escape (unrecognized hex)",
    }
}

impl fmt::Show for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        error_str(*self).fmt(f)
    }
}

/*
fn io_error_to_error(io: io::IoError) -> ParserError {
    IoError(io.kind, io.desc)
}
*/

pub type EncodeResult = io::IoResult<()>;

fn escape_str<W: Writer>(wr: &mut W, s: &str) -> Result<(), io::IoError> {
    try!(wr.write_str("\""));
    for byte in s.bytes() {
        match byte {
            b'"' => try!(wr.write_str("\\\"")),
            b'\\' => try!(wr.write_str("\\\\")),
            b'\x08' => try!(wr.write_str("\\b")),
            b'\x0c' => try!(wr.write_str("\\f")),
            b'\n' => try!(wr.write_str("\\n")),
            b'\r' => try!(wr.write_str("\\r")),
            b'\t' => try!(wr.write_str("\\t")),
            _ => try!(wr.write_u8(byte)),
        }
    }
    wr.write_str("\"")
}

fn spaces<W: Writer>(wr: &mut W, n: uint) -> Result<(), io::IoError> {
    for _ in range(0, n) {
        try!(wr.write_str(" "));
    }
    Ok(())
}

#[deriving(Show)]
enum SerializerState {
    ValueState,
    TupleState,
    StructState,
    EnumState,
}

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
}

impl<W: Writer> ser::Serializer<io::IoError> for Serializer<W> {
    #[inline]
    fn serialize_null(&mut self) -> Result<(), io::IoError> {
        self.wr.write_str("null")
    }

    #[inline]
    fn serialize_bool(&mut self, v: bool) -> Result<(), io::IoError> {
        if v {
            self.wr.write_str("true")
        } else {
            self.wr.write_str("false")
        }
    }

    #[inline]
    fn serialize_int(&mut self, v: int) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_i8(&mut self, v: i8) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_i16(&mut self, v: i16) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_i32(&mut self, v: i32) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_i64(&mut self, v: i64) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_uint(&mut self, v: uint) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_u8(&mut self, v: u8) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_u16(&mut self, v: u16) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_u32(&mut self, v: u32) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_u64(&mut self, v: u64) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_f32(&mut self, v: f32) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_f64(&mut self, v: f64) -> Result<(), io::IoError> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_char(&mut self, v: char) -> Result<(), io::IoError> {
        self.serialize_str(str::from_char(v).as_slice())
    }

    #[inline]
    fn serialize_str(&mut self, v: &str) -> Result<(), io::IoError> {
        escape_str(&mut self.wr, v)
    }

    #[inline]
    fn serialize_tuple_start(&mut self, _len: uint) -> Result<(), io::IoError> {
        self.first = true;
        self.wr.write_str("[")
    }

    #[inline]
    fn serialize_tuple_sep<
        T: Serializable
    >(&mut self, value: &T) -> Result<(), io::IoError> {
        if self.first {
            self.first = false;
        } else {
            try!(self.wr.write_str(","));
        }
        value.serialize(self)
    }

    #[inline]
    fn serialize_tuple_end(&mut self) -> Result<(), io::IoError> {
        self.wr.write_str("]")
    }

    #[inline]
    fn serialize_struct_start(&mut self, _name: &str, _len: uint) -> Result<(), io::IoError> {
        self.first = true;
        self.wr.write_str("{")
    }

    #[inline]
    fn serialize_struct_sep<
        T: Serializable
    >(&mut self, name: &str, value: &T) -> Result<(), io::IoError> {
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
    fn serialize_struct_end(&mut self) -> Result<(), io::IoError> {
        self.wr.write_str("}")
    }

    #[inline]
    fn serialize_enum_start(&mut self, _name: &str, variant: &str, _len: uint) -> Result<(), io::IoError> {
        self.first = true;
        try!(self.wr.write_str("{"));
        try!(self.serialize_str(variant));
        self.wr.write_str(":[")
    }

    #[inline]
    fn serialize_enum_sep<
        T: Serializable
    >(&mut self, value: &T) -> Result<(), io::IoError> {
        if self.first {
            self.first = false;
        } else {
            try!(self.wr.write_str(","));
        }
        value.serialize(self)
    }

    #[inline]
    fn serialize_enum_end(&mut self) -> Result<(), io::IoError> {
        self.wr.write_str("]}")
    }

    #[inline]
    fn serialize_option<
        T: Serializable
    >(&mut self, v: &Option<T>) -> Result<(), io::IoError> {
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
        T: Serializable,
        Iter: Iterator<T>
    >(&mut self, mut iter: Iter) -> Result<(), io::IoError> {
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
        K: Serializable,
        V: Serializable,
        Iter: Iterator<(K, V)>
    >(&mut self, mut iter: Iter) -> Result<(), io::IoError> {
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

    #[inline]
    fn serialize_sep(&mut self) -> Result<(), io::IoError> {
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
    fn serialize_end(&mut self, s: &str) -> Result<(), io::IoError> {
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
    fn serialize_null(&mut self) -> Result<(), io::IoError> {
        self.wr.write_str("null")
    }

    #[inline]
    fn serialize_bool(&mut self, v: bool) -> Result<(), io::IoError> {
        if v {
            self.wr.write_str("true")
        } else {
            self.wr.write_str("false")
        }
    }

    #[inline]
    fn serialize_int(&mut self, v: int) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_i8(&mut self, v: i8) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_i16(&mut self, v: i16) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_i32(&mut self, v: i32) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_i64(&mut self, v: i64) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_uint(&mut self, v: uint) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_u8(&mut self, v: u8) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_u16(&mut self, v: u16) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_u32(&mut self, v: u32) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_u64(&mut self, v: u64) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_f32(&mut self, v: f32) -> Result<(), io::IoError> {
        self.serialize_f64(v as f64)
    }

    #[inline]
    fn serialize_f64(&mut self, v: f64) -> Result<(), io::IoError> {
        write!(self.wr, "{}", v)
    }

    #[inline]
    fn serialize_char(&mut self, v: char) -> Result<(), io::IoError> {
        self.serialize_str(str::from_char(v).as_slice())
    }

    #[inline]
    fn serialize_str(&mut self, v: &str) -> Result<(), io::IoError> {
        escape_str(&mut self.wr, v)
    }

    #[inline]
    fn serialize_tuple_start(&mut self, _len: uint) -> Result<(), io::IoError> {
        self.first = true;
        self.wr.write_str("[")
    }

    #[inline]
    fn serialize_tuple_sep<
        T: Serializable
    >(&mut self, value: &T) -> Result<(), io::IoError> {
        try!(self.serialize_sep());
        value.serialize(self)
    }

    #[inline]
    fn serialize_tuple_end(&mut self) -> Result<(), io::IoError> {
        self.serialize_end("]")
    }

    #[inline]
    fn serialize_struct_start(&mut self, _name: &str, _len: uint) -> Result<(), io::IoError> {
        self.first = true;
        self.wr.write_str("{")
    }

    #[inline]
    fn serialize_struct_sep<
        T: Serializable
    >(&mut self, name: &str, value: &T) -> Result<(), io::IoError> {
        try!(self.serialize_sep());
        try!(self.serialize_str(name));
        try!(self.wr.write_str(": "));
        value.serialize(self)
    }

    #[inline]
    fn serialize_struct_end(&mut self) -> Result<(), io::IoError> {
        self.serialize_end("}")
    }

    #[inline]
    fn serialize_enum_start(&mut self, _name: &str, variant: &str, _len: uint) -> Result<(), io::IoError> {
        self.first = true;
        try!(self.wr.write_str("{"));
        try!(self.serialize_sep());
        try!(self.serialize_str(variant));
        self.first = true;
        self.wr.write_str(": [")
    }

    #[inline]
    fn serialize_enum_sep<
        T: Serializable
    >(&mut self, value: &T) -> Result<(), io::IoError> {
        try!(self.serialize_sep());
        value.serialize(self)
    }

    #[inline]
    fn serialize_enum_end(&mut self) -> Result<(), io::IoError> {
        try!(self.serialize_tuple_end());
        self.serialize_struct_end()
    }

    #[inline]
    fn serialize_option<
        T: Serializable
    >(&mut self, v: &Option<T>) -> Result<(), io::IoError> {
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
        T: Serializable,
        Iter: Iterator<T>
    >(&mut self, mut iter: Iter) -> Result<(), io::IoError> {
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
        K: Serializable,
        V: Serializable,
        Iter: Iterator<(K, V)>
    >(&mut self, mut iter: Iter) -> Result<(), io::IoError> {
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
pub fn to_vec<T: ser::Serializable>(value: &T) -> Vec<u8> {
    let mut wr = MemWriter::new();
    {
        let mut serializer = Serializer::new(wr.by_ref());
        value.serialize(&mut serializer).unwrap();
    }
    wr.unwrap()
}

/// Encode the specified struct into a json `String` buffer.
pub fn to_str<T: ser::Serializable>(value: &T) -> Result<String, Vec<u8>> {
    let buf = to_vec(value);
    String::from_utf8(buf)
}

/// Encode the specified struct into a json `[u8]` buffer.
pub fn to_pretty_vec<T: ser::Serializable>(value: &T) -> Vec<u8> {
    let mut wr = MemWriter::new();
    {
        let mut serializer = PrettySerializer::new(wr.by_ref());
        value.serialize(&mut serializer).unwrap();
    }
    wr.unwrap()
}

/// Encode the specified struct into a json `String` buffer.
pub fn to_pretty_str<T: ser::Serializable>(value: &T) -> Result<String, Vec<u8>> {
    let buf = to_pretty_vec(value);
    String::from_utf8(buf)
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
    ParseObjectKey,
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
          _ => { fail!(); }
        };
        *self.stack.get_mut(len - 1) = InternalIndex(idx);
    }
}
*/

/// A streaming JSON parser implemented as an iterator of JsonEvent, consuming
/// an iterator of char.
pub struct Parser<T> {
    rdr: T,
    ch: Option<char>,
    line: uint,
    col: uint,
    // A state machine is kept to make it possible to interupt and resume parsing.
    state_stack: Vec<ParserState>,
}

impl<T: Iterator<char>> Iterator<Result<de::Token, ParserError>> for Parser<T> {
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
            ParseObjectStart => Some(self.parse_object_start()),
            ParseObjectCommaOrEnd => Some(self.parse_object_comma_or_end()),
            ParseObjectKey => Some(self.parse_object_key()),
            ParseObjectValue => Some(self.parse_object_value()),
        }
    }
}

impl<T: Iterator<char>> Parser<T> {
    /// Creates the JSON parser.
    pub fn new(rdr: T) -> Parser<T> {
        let mut p = Parser {
            rdr: rdr,
            ch: Some('\x00'),
            line: 1,
            col: 0,
            state_stack: vec!(ParseValue),
        };
        p.bump();
        return p;
    }

    fn eof(&self) -> bool { self.ch.is_none() }
    fn ch_or_null(&self) -> char { self.ch.unwrap_or('\x00') }
    fn bump(&mut self) {
        self.ch = self.rdr.next();

        if self.ch_is('\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.bump();
        self.ch
    }
    fn ch_is(&self, c: char) -> bool {
        self.ch == Some(c)
    }

    fn error<T>(&self, reason: ErrorCode) -> Result<T, ParserError> {
        Err(SyntaxError(reason, self.line, self.col))
    }

    fn parse_whitespace(&mut self) {
        while self.ch_is(' ') ||
              self.ch_is('\n') ||
              self.ch_is('\t') ||
              self.ch_is('\r') { self.bump(); }
    }

    fn parse_number(&mut self) -> Result<f64, ParserError> {
        let mut neg = 1.0;

        if self.ch_is('-') {
            self.bump();
            neg = -1.0;
        }

        let mut res = match self.parse_integer() {
          Ok(res) => res,
          Err(e) => return Err(e)
        };

        if self.ch_is('.') {
            match self.parse_decimal(res) {
              Ok(r) => res = r,
              Err(e) => return Err(e)
            }
        }

        if self.ch_is('e') || self.ch_is('E') {
            match self.parse_exponent(res) {
              Ok(r) => res = r,
              Err(e) => return Err(e)
            }
        }

        Ok(neg * res)
    }

    fn parse_integer(&mut self) -> Result<f64, ParserError> {
        let mut res = 0.0;

        match self.ch_or_null() {
            '0' => {
                self.bump();

                // There can be only one leading '0'.
                match self.ch_or_null() {
                    '0' .. '9' => return self.error(InvalidNumber),
                    _ => ()
                }
            },
            '1' .. '9' => {
                while !self.eof() {
                    match self.ch_or_null() {
                        c @ '0' .. '9' => {
                            res *= 10.0;
                            res += ((c as int) - ('0' as int)) as f64;
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

    fn parse_decimal(&mut self, res: f64) -> Result<f64, ParserError> {
        self.bump();

        // Make sure a digit follows the decimal place.
        match self.ch_or_null() {
            '0' .. '9' => (),
             _ => return self.error(InvalidNumber)
        }

        let mut res = res;
        let mut dec = 1.0;
        while !self.eof() {
            match self.ch_or_null() {
                c @ '0' .. '9' => {
                    dec /= 10.0;
                    res += (((c as int) - ('0' as int)) as f64) * dec;
                    self.bump();
                }
                _ => break,
            }
        }

        Ok(res)
    }

    fn parse_exponent(&mut self, mut res: f64) -> Result<f64, ParserError> {
        self.bump();

        let mut exp = 0u;
        let mut neg_exp = false;

        if self.ch_is('+') {
            self.bump();
        } else if self.ch_is('-') {
            self.bump();
            neg_exp = true;
        }

        // Make sure a digit follows the exponent place.
        match self.ch_or_null() {
            '0' .. '9' => (),
            _ => return self.error(InvalidNumber)
        }
        while !self.eof() {
            match self.ch_or_null() {
                c @ '0' .. '9' => {
                    exp *= 10;
                    exp += (c as uint) - ('0' as uint);

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

    fn decode_hex_escape(&mut self) -> Result<u16, ParserError> {
        let mut i = 0u;
        let mut n = 0u16;
        while i < 4u && !self.eof() {
            self.bump();
            n = match self.ch_or_null() {
                c @ '0' .. '9' => n * 16_u16 + ((c as u16) - ('0' as u16)),
                'a' | 'A' => n * 16_u16 + 10_u16,
                'b' | 'B' => n * 16_u16 + 11_u16,
                'c' | 'C' => n * 16_u16 + 12_u16,
                'd' | 'D' => n * 16_u16 + 13_u16,
                'e' | 'E' => n * 16_u16 + 14_u16,
                'f' | 'F' => n * 16_u16 + 15_u16,
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

    fn parse_str(&mut self) -> Result<String, ParserError> {
        let mut escape = false;
        let mut res = String::new();

        loop {
            self.bump();
            if self.eof() {
                return self.error(EOFWhileParsingString);
            }

            if escape {
                match self.ch_or_null() {
                    '"' => res.push_char('"'),
                    '\\' => res.push_char('\\'),
                    '/' => res.push_char('/'),
                    'b' => res.push_char('\x08'),
                    'f' => res.push_char('\x0c'),
                    'n' => res.push_char('\n'),
                    'r' => res.push_char('\r'),
                    't' => res.push_char('\t'),
                    'u' => match try!(self.decode_hex_escape()) {
                        0xDC00 .. 0xDFFF => return self.error(LoneLeadingSurrogateInHexEscape),

                        // Non-BMP characters are encoded as a sequence of
                        // two hex escapes, representing UTF-16 surrogates.
                        n1 @ 0xD800 .. 0xDBFF => {
                            let c1 = self.next_char();
                            let c2 = self.next_char();
                            match (c1, c2) {
                                (Some('\\'), Some('u')) => (),
                                _ => return self.error(UnexpectedEndOfHexEscape),
                            }

                            let buf = [n1, try!(self.decode_hex_escape())];
                            match str::utf16_items(buf.as_slice()).next() {
                                Some(ScalarValue(c)) => res.push_char(c),
                                _ => return self.error(LoneLeadingSurrogateInHexEscape),
                            }
                        }

                        n => match char::from_u32(n as u32) {
                            Some(c) => res.push_char(c),
                            None => return self.error(InvalidUnicodeCodePoint),
                        },
                    },
                    _ => return self.error(InvalidEscape),
                }
                escape = false;
            } else if self.ch_is('\\') {
                escape = true;
            } else {
                match self.ch {
                    Some('"') => {
                        self.bump();
                        return Ok(res);
                    },
                    Some(c) => res.push_char(c),
                    None => unreachable!()
                }
            }
        }
    }

    fn parse_list_start(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.ch_is(']') {
            self.bump();
            Ok(de::End)
        } else {
            self.state_stack.push(ParseListCommaOrEnd);
            self.parse_value()
        }
    }

    fn parse_list_comma_or_end(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.ch_is(',') {
            self.bump();
            self.state_stack.push(ParseListCommaOrEnd);
            self.parse_value()
        } else if self.ch_is(']') {
            self.bump();
            Ok(de::End)
        } else if self.eof() {
            self.error_event(EOFWhileParsingList)
        } else {
            self.error_event(InvalidSyntax)
        }
    }

    fn parse_object_start(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.ch_is('}') {
            self.bump();
            Ok(de::End)
        } else {
            self.parse_object_key()
        }
    }

    fn parse_object_comma_or_end(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.ch_is(',') {
            self.bump();
            self.parse_object_key()
        } else if self.ch_is('}') {
            self.bump();
            Ok(de::End)
        } else if self.eof() {
            self.error_event(EOFWhileParsingObject)
        } else {
            self.error_event(InvalidSyntax)
        }
    }

    fn parse_object_key(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        self.state_stack.push(ParseObjectValue);

        if self.eof() {
            return self.error_event(EOFWhileParsingString);
        }

        match self.ch_or_null() {
            '"' => {
                let s = try!(self.parse_str());
                Ok(de::String(s))
            }
            _ => self.error_event(KeyMustBeAString),
        }
    }

    fn parse_object_value(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.ch_is(':') {
            self.bump();
            self.state_stack.push(ParseObjectCommaOrEnd);
            self.parse_value()
        } else if self.eof() {
            self.error_event(EOFWhileParsingObject)
        } else {
            self.error_event(ExpectedColon)
        }
    }

    fn parse_value(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.eof() {
            return self.error_event(EOFWhileParsingValue);
        }

        match self.ch_or_null() {
            'n' => self.parse_ident("ull", de::Null),
            't' => self.parse_ident("rue", de::Bool(true)),
            'f' => self.parse_ident("alse", de::Bool(false)),
            '0' .. '9' | '-' => {
                let number = try!(self.parse_number());
                Ok(de::F64(number))
            }
            '"' => {
                let s = try!(self.parse_str());
                Ok(de::String(s))
            }
            '[' => {
                self.bump();
                self.state_stack.push(ParseListStart);
                Ok(de::SeqStart(0))
            }
            '{' => {
                self.bump();
                self.state_stack.push(ParseObjectStart);
                Ok(de::MapStart(0))
            }
            _ => {
                self.error_event(InvalidSyntax)
            }
        }
    }

    fn parse_ident(&mut self, ident: &str, token: de::Token) -> Result<de::Token, ParserError> {
        if ident.chars().all(|c| Some(c) == self.next_char()) {
            self.bump();
            Ok(token)
        } else {
            self.error_event(InvalidSyntax)
        }
    }

    fn error_event(&mut self, reason: ErrorCode) -> Result<de::Token, ParserError> {
        self.state_stack.clear();
        Err(SyntaxError(reason, self.line, self.col))
    }
}

impl<T: Iterator<char>> de::Deserializer<ParserError> for Parser<T> {
    fn end_of_stream_error<U>(&self) -> Result<U, ParserError> {
        Err(SyntaxError(EOFWhileParsingValue, self.line, self.col))
    }

    fn syntax_error<U>(&self) -> Result<U, ParserError> {
        Err(SyntaxError(InvalidSyntax, self.line, self.col))
    }

    // Special case treating options as a nullable value.
    #[inline]
    fn expect_option<
        U: de::Deserializable
    >(&mut self, token: de::Token) -> Result<Option<U>, ParserError> {
        match token {
            de::Null => Ok(None),
            token => {
                let value: U = try!(de::Deserializable::deserialize_token(self, token));
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
            _ => { return self.error(InvalidSyntax); }
        };

        // Enums only have one field in them, which is the variant name.
        let variant = match try!(self.expect_token()) {
            de::String(variant) => variant,
            _ => { return self.error(InvalidSyntax); }
        };

        // The variant's field is a list of the values.
        match try!(self.expect_token()) {
            de::SeqStart(_) => { }
            _ => { return self.error(InvalidSyntax); }
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
                    _ => self.error(InvalidSyntax),
                }
            }
            _ => self.error(InvalidSyntax),
        }
    }
}

/// Decodes a json value from an `Iterator<Char>`.
pub fn from_iter<
    Iter: Iterator<char>,
    T: de::Deserializable
>(iter: Iter) -> Result<T, ParserError> {
    let mut parser = Parser::new(iter);
    let value = try!(de::Deserializable::deserialize(&mut parser));

    // Make sure the whole stream has been consumed.
    match parser.next() {
        Some(Ok(_token)) => parser.error(TrailingCharacters),
        Some(Err(err)) => Err(err),
        None => Ok(value),
    }
}

/// Decodes a json value from a string
pub fn from_str<
    T: de::Deserializable
>(s: &str) -> Result<T, BuilderError> {
    from_iter(s.chars())
}

/// Decodes a json value from a `Json`.
pub fn from_json<
    T: de::Deserializable
>(json: Json) -> Result<T, ParserError> {
    let mut d = JsonDeserializer::new(json);
    de::Deserializable::deserialize(&mut d)
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

/// Test if two json values are less than one another
impl PartialOrd for Json {
    fn lt(&self, other: &Json) -> bool {
        match *self {
            Number(f0) => {
                match *other {
                    Number(f1) => f0 < f1,
                    String(_) | Boolean(_) | List(_) | Object(_) |
                    Null => true
                }
            }

            String(ref s0) => {
                match *other {
                    Number(_) => false,
                    String(ref s1) => s0 < s1,
                    Boolean(_) | List(_) | Object(_) | Null => true
                }
            }

            Boolean(b0) => {
                match *other {
                    Number(_) | String(_) => false,
                    Boolean(b1) => b0 < b1,
                    List(_) | Object(_) | Null => true
                }
            }

            List(ref l0) => {
                match *other {
                    Number(_) | String(_) | Boolean(_) => false,
                    List(ref l1) => (*l0) < (*l1),
                    Object(_) | Null => true
                }
            }

            Object(ref d0) => {
                match *other {
                    Number(_) | String(_) | Boolean(_) | List(_) => false,
                    Object(ref d1) => d0 < d1,
                    Null => true
                }
            }

            Null => {
                match *other {
                    Number(_) | String(_) | Boolean(_) | List(_) |
                    Object(_) =>
                        false,
                    Null => true
                }
            }
        }
    }
}

/// A trait for converting values to JSON
pub trait ToJson {
    /// Converts the value of `self` to an instance of JSON
    fn to_json(&self) -> Json;
}

impl ToJson for Json {
    fn to_json(&self) -> Json { (*self).clone() }
}

impl ToJson for int {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for i8 {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for i16 {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for i32 {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for i64 {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for uint {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for u8 {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for u16 {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for u32 {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for u64 {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for f32 {
    fn to_json(&self) -> Json { Number(*self as f64) }
}

impl ToJson for f64 {
    fn to_json(&self) -> Json { Number(*self) }
}

impl ToJson for bool {
    fn to_json(&self) -> Json { Boolean(*self) }
}

impl<'a> ToJson for &'a str {
    fn to_json(&self) -> Json { String(self.to_string()) }
}

impl ToJson for String {
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
            #[allow(uppercase_variables)]
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

impl_to_json_tuple! { T0, T1, } // T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

impl<A:ToJson> ToJson for Vec<A> {
    fn to_json(&self) -> Json { List(self.iter().map(|elt| elt.to_json()).collect()) }
}

impl<A:ToJson> ToJson for TreeMap<String, A> {
    fn to_json(&self) -> Json {
        let mut d = TreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_json());
        }
        Object(d)
    }
}

impl<A:ToJson> ToJson for HashMap<String, A> {
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
    use std::collections::TreeMap;

    use super::{Json, Null, Boolean, Number, String, List, Object};
    use super::{ParserError, from_iter, from_str};
    use super::{from_json, ToJson};
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
    };
    use de;
    use ser::Serializable;
    use ser;

    macro_rules! treemap {
        ($($k:expr => $v:expr),*) => ({
            let mut _m = ::std::collections::TreeMap::new();
            $(_m.insert($k, $v);)*
            _m
        })
    }

    #[deriving(PartialEq, Show)]
    enum Animal {
        Dog,
        Frog(String, Vec<int>)
    }

    impl ser::Serializable for Animal {
        #[inline]
        fn serialize<
            S: ser::Serializer<E>,
            E
        >(&self, s: &mut S) -> Result<(), E> {
            match *self {
                Dog => {
                    try!(s.serialize_enum_start("Animal", "Dog", 0));
                    s.serialize_enum_end()
                }
                Frog(ref x0, ref x1) => {
                    try!(s.serialize_enum_start("Animal", "Frog", 2));

                    try!(s.serialize_enum_sep(x0));
                    try!(s.serialize_enum_sep(x1));

                    s.serialize_enum_end()
                }
            }
        }
    }

    impl de::Deserializable for Animal {
        #[inline]
        fn deserialize_token<
            D: de::Deserializer<E>,
            E
        >(d: &mut D, token: de::Token) -> Result<Animal, E> {
            match try!(d.expect_enum_start(token, "Animal", ["Dog", "Frog"])) {
                0 => {
                    try!(d.expect_enum_end());
                    Ok(Dog)
                }
                1 => {
                    let x0 = try!(de::Deserializable::deserialize(d));
                    let x1 = try!(de::Deserializable::deserialize(d));

                    try!(d.expect_enum_end());

                    Ok(Frog(x0, x1))
                }
                _ => d.syntax_error(),
            }
        }
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
    struct Inner {
        a: (),
        b: uint,
        c: Vec<String>,
    }

    impl ser::Serializable for Inner {
        #[inline]
        fn serialize<
            S: ser::Serializer<E>,
            E
        >(&self, s: &mut S) -> Result<(), E> {
            try!(s.serialize_struct_start("Inner", 3));

            try!(s.serialize_struct_sep("a", &self.a));
            try!(s.serialize_struct_sep("b", &self.b));
            try!(s.serialize_struct_sep("c", &self.c));

            s.serialize_struct_end()
        }
    }

    impl de::Deserializable for Inner {
        #[inline]
        fn deserialize_token<
            D: de::Deserializer<E>, E
        >(d: &mut D, token: de::Token) -> Result<Inner, E> {
            match token {
                de::StructStart("Inner", _) |
                de::MapStart(_) => {
                    let mut a = None;
                    let mut b = None;
                    let mut c = None;

                    loop {
                        match try!(d.expect_token()) {
                            de::End => { break; }
                            de::Str(name) => {
                                match name {
                                    "a" => {
                                        a = Some(try!(de::Deserializable::deserialize(d)));
                                    }
                                    "b" => {
                                        b = Some(try!(de::Deserializable::deserialize(d)));
                                    }
                                    "c" => {
                                        c = Some(try!(de::Deserializable::deserialize(d)));
                                    }
                                    _ => { }
                                }
                            }
                            de::String(ref name) => {
                                match name.as_slice() {
                                    "a" => {
                                        a = Some(try!(de::Deserializable::deserialize(d)));
                                    }
                                    "b" => {
                                        b = Some(try!(de::Deserializable::deserialize(d)));
                                    }
                                    "c" => {
                                        c = Some(try!(de::Deserializable::deserialize(d)));
                                    }
                                    _ => { }
                                }
                            }
                            _ => { return d.syntax_error(); }
                        }
                    }

                    match (a, b, c) {
                        (Some(a), Some(b), Some(c)) => {
                            Ok(Inner { a: a, b: b, c: c })
                        }
                        _ => d.syntax_error(),
                    }
                }
                _ => d.syntax_error(),
            }
        }
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
    struct Outer {
        inner: Vec<Inner>,
    }

    impl ser::Serializable for Outer {
        #[inline]
        fn serialize<
            S: ser::Serializer<E>,
            E
        >(&self, s: &mut S) -> Result<(), E> {
            try!(s.serialize_struct_start("Outer", 1));

            try!(s.serialize_struct_sep("inner", &self.inner));

            s.serialize_struct_end()
        }
    }

    impl de::Deserializable for Outer {
        #[inline]
        fn deserialize_token<
            D: de::Deserializer<E>,
            E
        >(d: &mut D, token: de::Token) -> Result<Outer, E> {
            match token {
                de::StructStart("Outer", _) |
                de::MapStart(_) => {
                    let mut inner = None;

                    loop {
                        match try!(d.expect_token()) {
                            de::End => { break; }
                            de::Str(name) => {
                                match name {
                                    "inner" => {
                                        inner = Some(try!(de::Deserializable::deserialize(d)));
                                    }
                                    _ => { }
                                }
                            }
                            de::String(ref name) => {
                                match name.as_slice() {
                                    "inner" => {
                                        inner = Some(try!(de::Deserializable::deserialize(d)));
                                    }
                                    _ => { }
                                }
                            }
                            _ => { return d.syntax_error(); }
                        }
                    }

                    match inner {
                        Some(inner) => {
                            Ok(Outer { inner: inner })
                        }
                        _ => d.syntax_error(),
                    }
                }
                _ => d.syntax_error(),
            }
        }
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
        T: PartialEq + Show + ToJson + ser::Serializable
    >(errors: &[(T, &str)]) {
        for &(ref value, out) in errors.iter() {
            let out = out.to_string();

            let s = super::to_str(value).unwrap();
            assert_eq!(s, out);

            let s = super::to_str(&value.to_json()).unwrap();
            assert_eq!(s, out);
        }
    }

    fn test_pretty_encode_ok<
        T: PartialEq + Show + ToJson + ser::Serializable
    >(errors: &[(T, &str)]) {
        for &(ref value, out) in errors.iter() {
            let out = out.to_string();

            let s = super::to_pretty_str(value).unwrap();
            assert_eq!(s, out);

            let s = super::to_pretty_str(&value.to_json()).unwrap();
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
    fn test_write_number() {
        let tests = [
            (3.0, "3"),
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
            List(vec![String("foo\nbar".to_string()), Number(3.5)])]);

        test_encode_ok([
            (long_test_list, "[false,null,[\"foo\\nbar\",3.5]]"),
        ]);

        let long_test_list = List(vec![
            Boolean(false),
            Null,
            List(vec![String("foo\nbar".to_string()), Number(3.5)])]);

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
                (5,),
                "[5]",
            ),
        ]);

        test_pretty_encode_ok([
            (
                (5,),
                concat!(
                    "[\n",
                    "  5\n",
                    "]"
                ),
            ),
        ]);

        test_encode_ok([
            (
                (5, (6, "abc")),
                "[5,[6,\"abc\"]]",
            ),
        ]);

        test_pretty_encode_ok([
            (
                (5, (6, "abc")),
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
        T: Show + de::Deserializable
    >(errors: &[(&str, ParserError)]) {
        for &(s, ref err) in errors.iter() {
            let v: Result<T, ParserError> = from_iter(s.chars());
            assert_eq!(v.unwrap_err(), *err);
        }
    }

    fn test_parse_ok<
        T: PartialEq + Show + ToJson + de::Deserializable
    >(errors: &[(&str, T)]) {
        for &(s, ref value) in errors.iter() {
            let v: T = from_iter(s.chars()).unwrap();
            assert_eq!(v, *value);

            let v: Json = from_iter(s.chars()).unwrap();
            assert_eq!(v, value.to_json());
        }
    }

    fn test_json_deserialize_ok<
        T: PartialEq + Show + ToJson + de::Deserializable
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
            ("n", SyntaxError(InvalidSyntax, 1, 2)),
            ("nul", SyntaxError(InvalidSyntax, 1, 4)),
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
            ("t", SyntaxError(InvalidSyntax, 1, 2)),
            ("truz", SyntaxError(InvalidSyntax, 1, 4)),
            ("f", SyntaxError(InvalidSyntax, 1, 2)),
            ("faz", SyntaxError(InvalidSyntax, 1, 3)),
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
    fn test_parse_numbers() {
        test_parse_err::<f64>([
            ("+", SyntaxError(InvalidSyntax, 1, 1)),
            (".", SyntaxError(InvalidSyntax, 1, 1)),
            ("-", SyntaxError(InvalidNumber, 1, 2)),
            ("00", SyntaxError(InvalidNumber, 1, 2)),
            ("1.", SyntaxError(InvalidNumber, 1, 3)),
            ("1e", SyntaxError(InvalidNumber, 1, 3)),
            ("1e+", SyntaxError(InvalidNumber, 1, 4)),
            ("1a", SyntaxError(TrailingCharacters, 1, 2)),
        ]);

        test_parse_ok([
            ("3", 3.0),
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
            3.0,
            3.1,
            -1.2,
            0.4,
            0.4e5,
            0.4e15,
            0.4e-01,
        ]);
    }

    #[test]
    fn test_parse_str() {
        test_parse_err::<String>([
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
            ("[1,]", SyntaxError(InvalidSyntax, 1, 4)),
            ("[1 2]", SyntaxError(InvalidSyntax, 1, 4)),
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
            ("[3,1]", vec!(3, 1)),
            ("[ 3 , 1 ]", vec!(3, 1)),
        ]);

        test_parse_ok([
            ("[[3], [1, 2]]", vec!(vec!(3), vec!(1, 2))),
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
            vec!(3, 1),
        ]);

        test_json_deserialize_ok([
            vec!(vec!(3), vec!(1, 2)),
        ]);
    }

    #[test]
    fn test_parse_object() {
        test_parse_err::<TreeMap<String, int>>([
            ("{", SyntaxError(EOFWhileParsingString, 1, 2)),
            ("{ ", SyntaxError(EOFWhileParsingString, 1, 3)),
            ("{1", SyntaxError(KeyMustBeAString, 1, 2)),
            ("{ \"a\"", SyntaxError(EOFWhileParsingObject, 1, 6)),
            ("{\"a\"", SyntaxError(EOFWhileParsingObject, 1, 5)),
            ("{\"a\" ", SyntaxError(EOFWhileParsingObject, 1, 6)),
            ("{\"a\" 1", SyntaxError(ExpectedColon, 1, 6)),
            ("{\"a\":", SyntaxError(EOFWhileParsingValue, 1, 6)),
            ("{\"a\":1", SyntaxError(EOFWhileParsingObject, 1, 7)),
            ("{\"a\":1 1", SyntaxError(InvalidSyntax, 1, 8)),
            ("{\"a\":1,", SyntaxError(EOFWhileParsingString, 1, 8)),
            ("{}a", SyntaxError(TrailingCharacters, 1, 3)),
        ]);

        test_parse_ok([
            ("{}", treemap!()),
            ("{ }", treemap!()),
            (
                "{\"a\":3}",
                treemap!("a".to_string() => 3)
            ),
            (
                "{ \"a\" : 3 }",
                treemap!("a".to_string() => 3)
            ),
            (
                "{\"a\":3,\"b\":4}",
                treemap!("a".to_string() => 3, "b".to_string() => 4)
            ),
            (
                "{ \"a\" : 3 , \"b\" : 4 }",
                treemap!("a".to_string() => 3, "b".to_string() => 4),
            ),
        ]);

        test_parse_ok([
            (
                "{\"a\": {\"b\": 3, \"c\": 4}}",
                treemap!("a".to_string() => treemap!("b".to_string() => 3, "c".to_string() => 4)),
            ),
        ]);
    }

    #[test]
    fn test_json_deserialize_object() {
        test_json_deserialize_ok([
            treemap!(),
            treemap!("a".to_string() => 3),
            treemap!("a".to_string() => 3, "b".to_string() => 4),
        ]);

        test_json_deserialize_ok([
            treemap!("a".to_string() => treemap!("b".to_string() => 3, "c".to_string() => 4)),
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
        test_parse_err::<TreeMap<String, String>>([
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
            Ok(_) => fail!("`{}` parsed & decoded ok, expecting error `{}`",
                              to_parse, expected),
            Err(ParseError(e)) => fail!("`{}` is not valid json: {}",
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
    fn test_is_object(){
        let json_value: Json = from_str("{}").unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_as_object(){
        let json_value: Json = from_str("{}").unwrap();
        let json_object = json_value.as_object();
        assert!(json_object.is_some());
    }

    #[test]
    fn test_is_list(){
        let json_value: Json = from_str("[1, 2, 3]").unwrap();
        assert!(json_value.is_list());
    }

    #[test]
    fn test_as_list(){
        let json_value: Json = from_str("[1, 2, 3]").unwrap();
        let json_list = json_value.as_list();
        let expected_length = 3;
        assert!(json_list.is_some() && json_list.unwrap().len() == expected_length);
    }

    #[test]
    fn test_is_string(){
        let json_value: Json = from_str("\"dog\"").unwrap();
        assert!(json_value.is_string());
    }

    #[test]
    fn test_as_string(){
        let json_value: Json = from_str("\"dog\"").unwrap();
        let json_str = json_value.as_string();
        let expected_str = "dog";
        assert_eq!(json_str, Some(expected_str));
    }

    #[test]
    fn test_is_number(){
        let json_value: Json = from_str("12").unwrap();
        assert!(json_value.is_number());
    }

    #[test]
    fn test_as_number(){
        let json_value: Json = from_str("12").unwrap();
        let json_num = json_value.as_number();
        let expected_num = 12f64;
        assert!(json_num.is_some() && json_num.unwrap() == expected_num);
    }

    #[test]
    fn test_is_boolean(){
        let json_value: Json = from_str("false").unwrap();
        assert!(json_value.is_boolean());
    }

    #[test]
    fn test_as_boolean(){
        let json_value: Json = from_str("false").unwrap();
        let json_bool = json_value.as_boolean();
        let expected_bool = false;
        assert!(json_bool.is_some() && json_bool.unwrap() == expected_bool);
    }

    #[test]
    fn test_is_null(){
        let json_value: Json = from_str("null").unwrap();
        assert!(json_value.is_null());
    }

    #[test]
    fn test_as_null(){
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
                fail!("Parser stack is not equal to {}", expected_stack);
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
                  (NumberValue(3.0), box [Key("a")]),
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
                  (NumberValue(1.0),    box [Key("a")]),
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
                  (NumberValue(1.0),            ~[Key("a")]),
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
                    (NumberValue(3.0), box [Index(0)]),
                    (NumberValue(1.0), box [Index(1)]),
                (ListEnd,       box []),
            ]
        );
        assert_stream_equal(
            "\n[3, 2]\n",
            box [
                (ListStart,     box []),
                    (NumberValue(3.0), box [Index(0)]),
                    (NumberValue(2.0), box [Index(1)]),
                (ListEnd,       box []),
            ]
        );
        assert_stream_equal(
            "[2, [4, 1]]",
            box [
                (ListStart,                 box []),
                    (NumberValue(2.0),      box [Index(0)]),
                    (ListStart,             box [Index(1)]),
                        (NumberValue(4.0),  box [Index(1), Index(0)]),
                        (NumberValue(1.0),  box [Index(1), Index(1)]),
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
    use serialize;
    use test::Bencher;

    use super::{Json, Null, Boolean, Number, String, List, Object};
    use super::{Parser, from_str};
    use de;

    macro_rules! treemap {
        ($($k:expr => $v:expr),*) => ({
            let mut _m = ::std::collections::TreeMap::new();
            $(_m.insert($k, $v);)*
            _m
        })
    }

    fn json_str(count: uint) -> String {
        let mut src = "[".to_string();
        for _ in range(0, count) {
            src.push_str(r#"{"a":true,"b":null,"c":3.1415,"d":"Hello world","e":[1,2,3]},"#);
        }
        src.push_str("{}]");
        src
    }

    fn pretty_json_str(count: uint) -> String {
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
            list.push(json::Object(box treemap!(
                "a".to_string() => json::Boolean(true),
                "b".to_string() => json::Null,
                "c".to_string() => json::Number(3.1415),
                "d".to_string() => json::String("Hello world".to_string()),
                "e".to_string() => json::List(vec!(
                    json::Number(1.0),
                    json::Number(2.0),
                    json::Number(3.0)
                ))
            )));
        }
        list.push(json::Object(box TreeMap::new()));
        json::List(list)
    }

    fn serializer_json(count: uint) -> Json {
        let mut list = vec!();
        for _ in range(0, count) {
            list.push(Object(treemap!(
                "a".to_string() => Boolean(true),
                "b".to_string() => Null,
                "c".to_string() => Number(3.1415),
                "d".to_string() => String("Hello world".to_string()),
                "e".to_string() => List(vec!(
                    Number(1.0),
                    Number(2.0),
                    Number(3.0)
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
            assert_eq!(json.to_str(), src);
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
            assert_eq!(json.to_str(), src);
        });
    }

    fn bench_serializer_pretty(b: &mut Bencher, count: uint) {
        let src = pretty_json_str(count);
        let json = serializer_json(count);

        b.iter(|| {
            assert_eq!(json.to_pretty_str(), src);
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

                assert_eq!(parser.next(), Some(json::NumberValue(3.1415)));
                assert_eq!(parser.stack().top(), Some(json::Key("c")));

                assert_eq!(parser.next(), Some(json::StringValue("Hello world".to_string())));
                assert_eq!(parser.stack().top(), Some(json::Key("d")));

                assert_eq!(parser.next(), Some(json::ListStart));
                assert_eq!(parser.stack().top(), Some(json::Key("e")));
                assert_eq!(parser.next(), Some(json::NumberValue(1.0)));
                assert_eq!(parser.next(), Some(json::NumberValue(2.0)));
                assert_eq!(parser.next(), Some(json::NumberValue(3.0)));
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
            let mut parser = Parser::new(src.as_slice().chars());

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
                assert_eq!(parser.next(), Some(Ok(de::F64(1.0))));
                assert_eq!(parser.next(), Some(Ok(de::F64(2.0))));
                assert_eq!(parser.next(), Some(Ok(de::F64(3.0))));
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
                    Some(Err(err)) => { fail!("error: {}", err); }
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
