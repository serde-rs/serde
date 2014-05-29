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
        let mut encoder = json::Encoder::new(&mut m as &mut std::io::Writer);
        match to_encode_object.encode(&mut encoder) {
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
let encoded_str: String = json::Encoder::str_encode(&to_encode_object);
```

JSON API provide an enum `json::Json` and a trait `ToJson` to encode object.
The trait `ToJson` encode object into a container `json::Json` and the API provide writer
to encode them into a stream or a string ...

When using `ToJson` the `Encodable` trait implementation is not mandatory.

A basic `ToJson` example using a TreeMap of attribute name / attribute value:


```rust
extern crate collections;
extern crate serialize;

use serialize::json;
use serialize::json::ToJson;
use collections::TreeMap;

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
    let encoded_str: String = json::Encoder::str_encode(&to_encode_object);

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
extern crate serialize;
extern crate collections;

use serialize::json::ToJson;
use serialize::{json, Encodable, Decodable};
use collections::TreeMap;

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
use std::f64;
use std::fmt;
use std::io::MemWriter;
use std::io;
use std::mem::swap;
use std::num;
use std::str::ScalarValue;
use std::str;
use std::string::String;
use std::vec::Vec;
use std::vec;

use de;
use collections::{HashMap, TreeMap};
use collections::treemap;

/// Represents a json value
#[deriving(Clone, Eq, Show)]
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

impl<E, D: de::Deserializer<E>> de::Deserializable<E, D> for Json {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<Json, E> {
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
                if fields.is_empty() {
                    Ok(String(name.to_string()))
                } else {
                    let mut object = TreeMap::new();
                    object.insert("variant".to_string(), String(name.to_string()));
                    object.insert("fields".to_string(), List(fields));
                    Ok(Object(object))
                }
            }
            de::End => Err(d.syntax_error()),
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
    fn end_of_stream_error(&self) -> ParserError {
        SyntaxError(EOFWhileParsingValue, 0, 0)
    }

    fn syntax_error(&self) -> ParserError {
        SyntaxError(InvalidSyntax, 0, 0)
    }

    // Special case treating options as a nullable value.
    #[inline]
    fn expect_option<
        U: de::Deserializable<ParserError, JsonDeserializer>
    >(&mut self, token: de::Token) -> Result<Option<U>, ParserError> {
        match token {
            de::Null => Ok(None),
            token => {
                let value: U = try!(de::Deserializable::deserialize_token(self, token));
                Ok(Some(value))
            }
        }
    }

    #[inline]
    fn expect_enum_start(&mut self, token: de::Token, _name: &str, variants: &[&str]) -> Result<uint, ParserError> {
        let variant = match token {
            de::String(variant) => {
                self.stack.push(JsonDeserializerEndState);
                variant
            }
            de::MapStart(_) => {
                let state = match self.stack.pop() {
                    Some(state) => state,
                    None => { fail!("state machine error, state stack empty"); }
                };

                let mut iter = match state {
                    JsonDeserializerObjectState(iter) => iter,
                    _ => { fail!("state machine error, expected an object"); }
                };

                let mut variant = None;
                let mut fields = None;

                for (key, value) in iter {
                    if key.equiv(&"variant") {
                        match value {
                            String(v) => { variant = Some(v); }
                            value => {
                                return Err(ExpectedError("String".to_string(),
                                                         format!("{}", value)))
                            }
                        }
                    } else if key.equiv(&"fields") {
                        match value {
                            List(v) => { fields = Some(v); }
                            value => {
                                return Err(ExpectedError("List".to_string(),
                                                         format!("{}", value)))
                            }
                        }
                    }
                }

                let (variant, fields) = match (variant, fields) {
                    (Some(variant), Some(fields)) => (variant, fields),
                    (None, _) => {
                        return Err(MissingFieldError("variant".to_string()))
                    }
                    (_, None) => {
                        return Err(MissingFieldError("fields".to_string()))
                    }
                };

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


        /*


        let name = match self.stack.pop() {
            String(s) => s,
            Object(mut object) => {
                let n = match object.pop_equiv(&"variant") {
                    Some(String(s)) => s,
                    Some(value) => {
                        return Err(ExpectedError("String".to_string(),
                                                 format!("{}", value)))
                    }
                    None => {
                        return Err(MissingFieldError("variant".to_string()))
                    }
                };
                match object.pop_equiv(&"fields") {
                    Some(List(list)) => {
                        for field in list.move_iter().rev() {
                            self.stack.push(JsonDeserializerValueState(field));
                        }
                    },
                    Some(value) => {
                        return Err(ExpectedError("List".to_string(),
                                                 format!("{}", value)))
                    }
                    None => {
                        return Err(MissingFieldError("fields".to_string()))
                    }
                }
                n
            }
            json => {
                return Err(ExpectedError("String or Object".to_string(),
                                         format!("{}", json)))
            }
        };
        */

        match variants.iter().position(|v| *v == variant.as_slice()) {
            Some(idx) => Ok(idx),
            None => Err(UnknownVariantError(variant)),
        }
    }
}

/// The errors that can arise while parsing a JSON stream.
#[deriving(Clone, Eq)]
pub enum ErrorCode {
    InvalidSyntax,
    InvalidNumber,
    EOFWhileParsingObject,
    EOFWhileParsingList,
    EOFWhileParsingValue,
    EOFWhileParsingString,
    KeyMustBeAString,
    ExpectedColon,
    TrailingCharacters,
    InvalidEscape,
    InvalidUnicodeCodePoint,
    LoneLeadingSurrogateInHexEscape,
    UnexpectedEndOfHexEscape,
    UnrecognizedHex,
    NotFourDigit,
    NotUtf8,
}

#[deriving(Clone, Eq, Show)]
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
        InvalidSyntax => "invalid syntax",
        InvalidNumber => "invalid number",
        EOFWhileParsingObject => "EOF While parsing object",
        EOFWhileParsingList => "EOF While parsing list",
        EOFWhileParsingValue => "EOF While parsing value",
        EOFWhileParsingString => "EOF While parsing string",
        KeyMustBeAString => "key must be a string",
        ExpectedColon => "expected `:`",
        TrailingCharacters => "trailing characters",
        InvalidEscape => "invalid escape",
        UnrecognizedHex => "invalid \\u escape (unrecognized hex)",
        NotFourDigit => "invalid \\u escape (not four digits)",
        NotUtf8 => "contents not utf-8",
        InvalidUnicodeCodePoint => "invalid unicode code point",
        LoneLeadingSurrogateInHexEscape => "lone leading surrogate in hex escape",
        UnexpectedEndOfHexEscape => "unexpected end of hex escape",
    }
}

impl fmt::Show for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        error_str(*self).fmt(f)
    }
}


fn io_error_to_error(io: io::IoError) -> ParserError {
    IoError(io.kind, io.desc)
}

pub type EncodeResult = io::IoResult<()>;
//pub type DecodeResult<T> = Result<T, DecoderError>;

fn escape_str(s: &str) -> String {
    let mut escaped = String::from_str("\"");
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\x08' => escaped.push_str("\\b"),
            '\x0c' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push_char(c),
        }
    };
    escaped.push_char('"');
    escaped
}

fn spaces(n: uint) -> String {
    let mut ss = String::new();
    for _ in range(0, n) {
        ss.push_str(" ");
    }
    return ss
}

/*
/// A structure for implementing serialization to JSON.
pub struct Encoder<'a> {
    wr: &'a mut io::Writer,
}

impl<'a> Encoder<'a> {
    /// Creates a new JSON encoder whose output will be written to the writer
    /// specified.
    pub fn new<'a>(wr: &'a mut io::Writer) -> Encoder<'a> {
        Encoder { wr: wr }
    }

    /// Encode the specified struct into a json [u8]
    pub fn buffer_encode<T:Encodable<Encoder<'a>, io::IoError>>(to_encode_object: &T) -> Vec<u8>  {
       //Serialize the object in a string using a writer
        let mut m = MemWriter::new();
        {
            let mut encoder = Encoder::new(&mut m as &mut io::Writer);
            // MemWriter never Errs
            let _ = to_encode_object.encode(&mut encoder);
        }
        m.unwrap()
    }

    /// Encode the specified struct into a json str
    pub fn str_encode<T:Encodable<Encoder<'a>,
                        io::IoError>>(
                      to_encode_object: &T)
                      -> String {
        let buff = Encoder::buffer_encode(to_encode_object);
        str::from_utf8(buff.as_slice()).unwrap().to_string()
    }
}

impl<'a> ::Encoder<io::IoError> for Encoder<'a> {
    fn emit_nil(&mut self) -> EncodeResult { write!(self.wr, "null") }

    fn emit_uint(&mut self, v: uint) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_u64(&mut self, v: u64) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_u32(&mut self, v: u32) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_u16(&mut self, v: u16) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_u8(&mut self, v: u8) -> EncodeResult  { self.emit_f64(v as f64) }

    fn emit_int(&mut self, v: int) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_i64(&mut self, v: i64) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_i32(&mut self, v: i32) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_i16(&mut self, v: i16) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_i8(&mut self, v: i8) -> EncodeResult  { self.emit_f64(v as f64) }

    fn emit_bool(&mut self, v: bool) -> EncodeResult {
        if v {
            write!(self.wr, "true")
        } else {
            write!(self.wr, "false")
        }
    }

    fn emit_f64(&mut self, v: f64) -> EncodeResult {
        write!(self.wr, "{}", f64::to_str_digits(v, 6u))
    }
    fn emit_f32(&mut self, v: f32) -> EncodeResult { self.emit_f64(v as f64) }

    fn emit_char(&mut self, v: char) -> EncodeResult {
        self.emit_str(str::from_char(v).as_slice())
    }
    fn emit_str(&mut self, v: &str) -> EncodeResult {
        write!(self.wr, "{}", escape_str(v))
    }

    fn emit_enum(&mut self,
                 _name: &str,
                 f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult { f(self) }

    fn emit_enum_variant(&mut self,
                         name: &str,
                         _id: uint,
                         cnt: uint,
                         f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        // enums are encoded as strings or objects
        // Bunny => "Bunny"
        // Kangaroo(34,"William") => {"variant": "Kangaroo", "fields": [34,"William"]}
        if cnt == 0 {
            write!(self.wr, "{}", escape_str(name))
        } else {
            try!(write!(self.wr, "\\{\"variant\":"));
            try!(write!(self.wr, "{}", escape_str(name)));
            try!(write!(self.wr, ",\"fields\":["));
            try!(f(self));
            write!(self.wr, "]\\}")
        }
    }

    fn emit_enum_variant_arg(&mut self,
                             idx: uint,
                             f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        if idx != 0 {
            try!(write!(self.wr, ","));
        }
        f(self)
    }

    fn emit_enum_struct_variant(&mut self,
                                name: &str,
                                id: uint,
                                cnt: uint,
                                f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_enum_variant(name, id, cnt, f)
    }

    fn emit_enum_struct_variant_field(&mut self,
                                      _: &str,
                                      idx: uint,
                                      f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_enum_variant_arg(idx, f)
    }

    fn emit_struct(&mut self,
                   _: &str,
                   _: uint,
                   f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        try!(write!(self.wr, r"\{"));
        try!(f(self));
        write!(self.wr, r"\}")
    }

    fn emit_struct_field(&mut self,
                         name: &str,
                         idx: uint,
                         f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        if idx != 0 { try!(write!(self.wr, ",")); }
        try!(write!(self.wr, "{}:", escape_str(name)));
        f(self)
    }

    fn emit_tuple(&mut self, len: uint, f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_seq(len, f)
    }
    fn emit_tuple_arg(&mut self,
                      idx: uint,
                      f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_seq_elt(idx, f)
    }

    fn emit_tuple_struct(&mut self,
                         _name: &str,
                         len: uint,
                         f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_seq(len, f)
    }
    fn emit_tuple_struct_arg(&mut self,
                             idx: uint,
                             f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_seq_elt(idx, f)
    }

    fn emit_option(&mut self, f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        f(self)
    }
    fn emit_option_none(&mut self) -> EncodeResult { self.emit_nil() }
    fn emit_option_some(&mut self, f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        f(self)
    }

    fn emit_seq(&mut self, _len: uint, f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        try!(write!(self.wr, "["));
        try!(f(self));
        write!(self.wr, "]")
    }

    fn emit_seq_elt(&mut self, idx: uint, f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        if idx != 0 {
            try!(write!(self.wr, ","));
        }
        f(self)
    }

    fn emit_map(&mut self, _len: uint, f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        try!(write!(self.wr, r"\{"));
        try!(f(self));
        write!(self.wr, r"\}")
    }

    fn emit_map_elt_key(&mut self,
                        idx: uint,
                        f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        use std::str::from_utf8;
        if idx != 0 { try!(write!(self.wr, ",")) }
        // ref #12967, make sure to wrap a key in double quotes,
        // in the event that its of a type that omits them (eg numbers)
        let mut buf = MemWriter::new();
        let mut check_encoder = Encoder::new(&mut buf);
        try!(f(&mut check_encoder));
        let buf = buf.unwrap();
        let out = from_utf8(buf.as_slice()).unwrap();
        let needs_wrapping = out.char_at(0) != '"' &&
            out.char_at_reverse(out.len()) != '"';
        if needs_wrapping { try!(write!(self.wr, "\"")); }
        try!(f(self));
        if needs_wrapping { try!(write!(self.wr, "\"")); }
        Ok(())
    }

    fn emit_map_elt_val(&mut self,
                        _idx: uint,
                        f: |&mut Encoder<'a>| -> EncodeResult) -> EncodeResult {
        try!(write!(self.wr, ":"));
        f(self)
    }
}

/// Another encoder for JSON, but prints out human-readable JSON instead of
/// compact data
pub struct PrettyEncoder<'a> {
    wr: &'a mut io::Writer,
    indent: uint,
}

impl<'a> PrettyEncoder<'a> {
    /// Creates a new encoder whose output will be written to the specified writer
    pub fn new<'a>(wr: &'a mut io::Writer) -> PrettyEncoder<'a> {
        PrettyEncoder {
            wr: wr,
            indent: 0,
        }
    }
}

impl<'a> ::Encoder<io::IoError> for PrettyEncoder<'a> {
    fn emit_nil(&mut self) -> EncodeResult { write!(self.wr, "null") }

    fn emit_uint(&mut self, v: uint) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_u64(&mut self, v: u64) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_u32(&mut self, v: u32) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_u16(&mut self, v: u16) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_u8(&mut self, v: u8) -> EncodeResult { self.emit_f64(v as f64) }

    fn emit_int(&mut self, v: int) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_i64(&mut self, v: i64) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_i32(&mut self, v: i32) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_i16(&mut self, v: i16) -> EncodeResult { self.emit_f64(v as f64) }
    fn emit_i8(&mut self, v: i8) -> EncodeResult { self.emit_f64(v as f64) }

    fn emit_bool(&mut self, v: bool) -> EncodeResult {
        if v {
            write!(self.wr, "true")
        } else {
            write!(self.wr, "false")
        }
    }

    fn emit_f64(&mut self, v: f64) -> EncodeResult {
        write!(self.wr, "{}", f64::to_str_digits(v, 6u))
    }
    fn emit_f32(&mut self, v: f32) -> EncodeResult {
        self.emit_f64(v as f64)
    }

    fn emit_char(&mut self, v: char) -> EncodeResult {
        self.emit_str(str::from_char(v).as_slice())
    }
    fn emit_str(&mut self, v: &str) -> EncodeResult {
        write!(self.wr, "{}", escape_str(v))
    }

    fn emit_enum(&mut self,
                 _name: &str,
                 f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        f(self)
    }

    fn emit_enum_variant(&mut self,
                         name: &str,
                         _: uint,
                         cnt: uint,
                         f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        if cnt == 0 {
            write!(self.wr, "{}", escape_str(name))
        } else {
            self.indent += 2;
            try!(write!(self.wr, "[\n{}{},\n", spaces(self.indent),
                          escape_str(name)));
            try!(f(self));
            self.indent -= 2;
            write!(self.wr, "\n{}]", spaces(self.indent))
        }
    }

    fn emit_enum_variant_arg(&mut self,
                             idx: uint,
                             f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        if idx != 0 {
            try!(write!(self.wr, ",\n"));
        }
        try!(write!(self.wr, "{}", spaces(self.indent)));
        f(self)
    }

    fn emit_enum_struct_variant(&mut self,
                                name: &str,
                                id: uint,
                                cnt: uint,
                                f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_enum_variant(name, id, cnt, f)
    }

    fn emit_enum_struct_variant_field(&mut self,
                                      _: &str,
                                      idx: uint,
                                      f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_enum_variant_arg(idx, f)
    }


    fn emit_struct(&mut self,
                   _: &str,
                   len: uint,
                   f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        if len == 0 {
            write!(self.wr, "\\{\\}")
        } else {
            try!(write!(self.wr, "\\{"));
            self.indent += 2;
            try!(f(self));
            self.indent -= 2;
            write!(self.wr, "\n{}\\}", spaces(self.indent))
        }
    }

    fn emit_struct_field(&mut self,
                         name: &str,
                         idx: uint,
                         f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        if idx == 0 {
            try!(write!(self.wr, "\n"));
        } else {
            try!(write!(self.wr, ",\n"));
        }
        try!(write!(self.wr, "{}{}: ", spaces(self.indent), escape_str(name)));
        f(self)
    }

    fn emit_tuple(&mut self,
                  len: uint,
                  f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_seq(len, f)
    }
    fn emit_tuple_arg(&mut self,
                      idx: uint,
                      f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_seq_elt(idx, f)
    }

    fn emit_tuple_struct(&mut self,
                         _: &str,
                         len: uint,
                         f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_seq(len, f)
    }
    fn emit_tuple_struct_arg(&mut self,
                             idx: uint,
                             f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        self.emit_seq_elt(idx, f)
    }

    fn emit_option(&mut self, f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        f(self)
    }
    fn emit_option_none(&mut self) -> EncodeResult { self.emit_nil() }
    fn emit_option_some(&mut self, f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        f(self)
    }

    fn emit_seq(&mut self,
                len: uint,
                f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        if len == 0 {
            write!(self.wr, "[]")
        } else {
            try!(write!(self.wr, "["));
            self.indent += 2;
            try!(f(self));
            self.indent -= 2;
            write!(self.wr, "\n{}]", spaces(self.indent))
        }
    }

    fn emit_seq_elt(&mut self,
                    idx: uint,
                    f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        if idx == 0 {
            try!(write!(self.wr, "\n"));
        } else {
            try!(write!(self.wr, ",\n"));
        }
        try!(write!(self.wr, "{}", spaces(self.indent)));
        f(self)
    }

    fn emit_map(&mut self,
                len: uint,
                f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        if len == 0 {
            write!(self.wr, "\\{\\}")
        } else {
            try!(write!(self.wr, "\\{"));
            self.indent += 2;
            try!(f(self));
            self.indent -= 2;
            write!(self.wr, "\n{}\\}", spaces(self.indent))
        }
    }

    fn emit_map_elt_key(&mut self,
                        idx: uint,
                        f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        use std::str::from_utf8;
        if idx == 0 {
            try!(write!(self.wr, "\n"));
        } else {
            try!(write!(self.wr, ",\n"));
        }
        try!(write!(self.wr, "{}", spaces(self.indent)));
        // ref #12967, make sure to wrap a key in double quotes,
        // in the event that its of a type that omits them (eg numbers)
        let mut buf = MemWriter::new();
        let mut check_encoder = PrettyEncoder::new(&mut buf);
        try!(f(&mut check_encoder));
        let buf = buf.unwrap();
        let out = from_utf8(buf.as_slice()).unwrap();
        let needs_wrapping = out.char_at(0) != '"' &&
            out.char_at_reverse(out.len()) != '"';
        if needs_wrapping { try!(write!(self.wr, "\"")); }
        try!(f(self));
        if needs_wrapping { try!(write!(self.wr, "\"")); }
        Ok(())
    }

    fn emit_map_elt_val(&mut self,
                        _idx: uint,
                        f: |&mut PrettyEncoder<'a>| -> EncodeResult) -> EncodeResult {
        try!(write!(self.wr, ": "));
        f(self)
    }
}

impl<E: ::Encoder<S>, S> Encodable<E, S> for Json {
    fn encode(&self, e: &mut E) -> Result<(), S> {
        match *self {
            Number(v) => v.encode(e),
            String(ref v) => v.encode(e),
            Boolean(v) => v.encode(e),
            List(ref v) => v.encode(e),
            Object(ref v) => v.encode(e),
            Null => e.emit_nil(),
        }
    }
}

impl Json {
    /// Encodes a json value into an io::writer.  Uses a single line.
    pub fn to_writer(&self, wr: &mut io::Writer) -> EncodeResult {
        let mut encoder = Encoder::new(wr);
        self.encode(&mut encoder)
    }

    /// Encodes a json value into an io::writer.
    /// Pretty-prints in a more readable format.
    pub fn to_pretty_writer(&self, wr: &mut io::Writer) -> EncodeResult {
        let mut encoder = PrettyEncoder::new(wr);
        self.encode(&mut encoder)
    }

    /// Encodes a json value into a string
    pub fn to_pretty_str(&self) -> String {
        let mut s = MemWriter::new();
        self.to_pretty_writer(&mut s as &mut io::Writer).unwrap();
        str::from_utf8(s.unwrap().as_slice()).unwrap().to_string()
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
        match self {
            &Object(ref map) => Some(&**map),
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
        match self {
            &List(ref list) => Some(&*list),
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
        match self {
            &Number(n) => Some(n),
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
        match self {
            &Boolean(b) => Some(b),
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
        match self {
            &Null => Some(()),
            _ => None
        }
    }
}
*/

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

#[deriving(Eq, Show)]
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

/// A streaming JSON parser implemented as an iterator of JsonEvent, consuming
/// an iterator of char.
pub struct Parser<T> {
    rdr: T,
    ch: Option<char>,
    line: uint,
    col: uint,
    // A state machine is kept to make it possible to interupt and resume parsing.
    state: Vec<ParserState>,
}

impl<T: Iterator<char>> Iterator<Result<de::Token, ParserError>> for Parser<T> {
    #[inline]
    fn next(&mut self) -> Option<Result<de::Token, ParserError>> {
        let state = match self.state.pop() {
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
            state: vec!(ParseValue),
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
            self.state.push(ParseListCommaOrEnd);
            self.parse_value()
        }
    }

    fn parse_list_comma_or_end(&mut self) -> Result<de::Token, ParserError> {
        self.parse_whitespace();

        if self.ch_is(',') {
            self.bump();
            self.state.push(ParseListCommaOrEnd);
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

        self.state.push(ParseObjectValue);

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
            self.state.push(ParseObjectCommaOrEnd);
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
                self.state.push(ParseListStart);
                Ok(de::SeqStart(0))
            }
            '{' => {
                self.bump();
                self.state.push(ParseObjectStart);
                Ok(de::MapStart(0))
            }
            _ => {
                self.error_event(InvalidSyntax)
            }
        }
    }

    fn parse_ident(&mut self, ident: &str, value: de::Token) -> Result<de::Token, ParserError> {
        if ident.chars().all(|c| Some(c) == self.next_char()) {
            self.bump();
            Ok(value)
        } else {
            Err(SyntaxError(InvalidSyntax, self.line, self.col))
        }
    }

    fn error_event(&mut self, reason: ErrorCode) -> Result<de::Token, ParserError> {
        self.state.clear();
        Err(SyntaxError(reason, self.line, self.col))
    }
}

impl<T: Iterator<char>> de::Deserializer<ParserError> for Parser<T> {
    fn end_of_stream_error(&self) -> ParserError {
        SyntaxError(EOFWhileParsingValue, self.line, self.col)
    }

    fn syntax_error(&self) -> ParserError {
        SyntaxError(InvalidSyntax, self.line, self.col)
    }

    // Special case treating options as a nullable value.
    #[inline]
    fn expect_option<
        U: de::Deserializable<ParserError, Parser<T>>
    >(&mut self, token: de::Token) -> Result<Option<U>, ParserError> {
        match token {
            de::Null => Ok(None),
            token => {
                let value: U = try!(de::Deserializable::deserialize_token(self, token));
                Ok(Some(value))
            }
        }
    }

    /*
    #[inline]
    fn expect_enum_start(&mut self, token: de::Token, _name: &str, variants: &[&str]) -> Result<uint, ParserError> {
        match token {
            Str(name) =>

        }

        let name = match self.pop() {
            String(s) => s,
            Object(mut o) => {
                let n = match o.pop(&"variant".to_string()) {
                    Some(String(s)) => s,
                    Some(val) => {
                        return Err(ExpectedError("String".to_string(),
                                                 format!("{}", val)))
                    }
                    None => {
                        return Err(MissingFieldError("variant".to_string()))
                    }
                };
                match o.pop(&"fields".to_string()) {
                    Some(List(l)) => {
                        for field in l.move_iter().rev() {
                            self.stack.push(field.clone());
                        }
                    },
                    Some(val) => {
                        return Err(ExpectedError("List".to_string(),
                                                 format!("{}", val)))
                    }
                    None => {
                        return Err(MissingFieldError("fields".to_string()))
                    }
                }
                n
            }
            json => {
                return Err(ExpectedError("String or Object".to_string(),
                                         format!("{}", json)))
            }
        };
        let idx = match names.iter()
                             .position(|n| {
                                 str::eq_slice(*n, name.as_slice())
                             }) {
            Some(idx) => idx,
            None => return Err(UnknownVariantError(name))
        };
        f(self, idx)

    }
    */
}

/*
/// A Builder consumes a json::Parser to create a generic Json structure.
pub struct Builder<T> {
    parser: Parser<T>,
    token: Option<JsonEvent>,
}

impl<T: Iterator<char>> Builder<T> {
    /// Create a JSON Builder.
    pub fn new(src: T) -> Builder<T> {
        Builder {
            parser: Parser::new(src),
            token: None,
        }
    }

    // Decode a Json value from a Parser.
    pub fn build(&mut self) -> Result<Json, BuilderError> {
        self.bump();
        let result = self.build_value();
        self.bump();
        match self.token {
            None => {}
            Some(Error(e)) => { return Err(e); }
            ref tok => { fail!("unexpected token {}", tok.clone()); }
        }
        return result;
    }

    fn bump(&mut self) {
        self.token = self.parser.next();
    }

    fn build_value(&mut self) -> Result<Json, BuilderError> {
        return match self.token {
            Some(NullValue) => { Ok(Null) }
            Some(NumberValue(n)) => { Ok(Number(n)) }
            Some(BooleanValue(b)) => { Ok(Boolean(b)) }
            Some(StringValue(ref mut s)) => {
                let mut temp = String::new();
                swap(s, &mut temp);
                Ok(String(temp))
            }
            Some(Error(e)) => { Err(e) }
            Some(ListStart) => { self.build_list() }
            Some(ObjectStart) => { self.build_object() }
            Some(ObjectEnd) => { self.parser.error(InvalidSyntax) }
            Some(ListEnd) => { self.parser.error(InvalidSyntax) }
            None => { self.parser.error(EOFWhileParsingValue) }
        }
    }

    fn build_list(&mut self) -> Result<Json, BuilderError> {
        self.bump();
        let mut values = Vec::new();

        loop {
            if self.token == Some(ListEnd) {
                return Ok(List(values.move_iter().collect()));
            }
            match self.build_value() {
                Ok(v) => values.push(v),
                Err(e) => { return Err(e) }
            }
            self.bump();
        }
    }

    fn build_object(&mut self) -> Result<Json, BuilderError> {
        self.bump();

        let mut values = box TreeMap::new();

        while self.token != None {
            match self.token {
                Some(ObjectEnd) => { return Ok(Object(values)); }
                Some(Error(e)) => { return Err(e); }
                None => { break; }
                _ => {}
            }
            let key = match self.parser.stack().top() {
                Some(Key(k)) => { k.to_string() }
                _ => { fail!("invalid state"); }
            };
            match self.build_value() {
                Ok(value) => { values.insert(key, value); }
                Err(e) => { return Err(e); }
            }
            self.bump();
        }
        return self.parser.error(EOFWhileParsingObject);
    }
}
*/

/// Decodes a json value from an `Iterator<Char>`.
pub fn from_iter<
    Iter: Iterator<char>,
    T: de::Deserializable<ParserError, Parser<Iter>>
>(iter: Iter) -> Result<T, ParserError> {
    let mut parser = Parser::new(iter);
    let value = try!(de::Deserializable::deserialize(&mut parser));

    // Make sure the whole stream has been consumed.
    match parser.next() {
        Some(Ok(token)) => {
            fail!("internal json error, there should have not have been any tokens left");
        }
        Some(Err(err)) => { return Err(err); }
        None => { }
    }

    Ok(value)
}


/// Decodes a json value from a `Json`.
pub fn from_json<
    T: de::Deserializable<ParserError, JsonDeserializer>
>(json: Json) -> Result<T, ParserError> {
    let mut d = JsonDeserializer::new(json);
    de::Deserializable::deserialize(&mut d)
}




    /*
    let contents = match rdr.read_to_end() {
        Ok(c) => c,
        Err(e) => return Err(io_error_to_error(e))
    };
    let s = match str::from_utf8(contents.as_slice()) {
        Some(s) => s.to_string(),
        None => return Err(SyntaxError(NotUtf8, 0, 0))
    };
    let mut builder = Builder::new(s.as_slice().chars());
    builder.build()
    */
//}

/*
/// Decodes a json value from a string
pub fn from_str(s: &str) -> Result<Json, BuilderError> {
    let mut builder = Builder::new(s.chars());
    return builder.build();
}
    */

/// A structure to decode JSON to values in rust.
pub struct Decoder {
    stack: Vec<Json>,
}

impl Decoder {
    /// Creates a new decoder instance for decoding the specified JSON value.
    pub fn new(json: Json) -> Decoder {
        Decoder {
            stack: vec!(json),
        }
    }
}

impl Decoder {
    fn pop(&mut self) -> Json {
        self.stack.pop().unwrap()
    }
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

    /*
impl ::Decoder<DecoderError> for Decoder {

    fn read_nil(&mut self) -> DecodeResult<()> {
        debug!("read_nil");
        try!(expect!(self.pop(), Null));
        Ok(())
    }

    fn read_u64(&mut self)  -> DecodeResult<u64 > { Ok(try!(self.read_f64()) as u64) }
    fn read_u32(&mut self)  -> DecodeResult<u32 > { Ok(try!(self.read_f64()) as u32) }
    fn read_u16(&mut self)  -> DecodeResult<u16 > { Ok(try!(self.read_f64()) as u16) }
    fn read_u8 (&mut self)  -> DecodeResult<u8  > { Ok(try!(self.read_f64()) as u8) }
    fn read_uint(&mut self) -> DecodeResult<uint> { Ok(try!(self.read_f64()) as uint) }

    fn read_i64(&mut self) -> DecodeResult<i64> { Ok(try!(self.read_f64()) as i64) }
    fn read_i32(&mut self) -> DecodeResult<i32> { Ok(try!(self.read_f64()) as i32) }
    fn read_i16(&mut self) -> DecodeResult<i16> { Ok(try!(self.read_f64()) as i16) }
    fn read_i8 (&mut self) -> DecodeResult<i8 > { Ok(try!(self.read_f64()) as i8) }
    fn read_int(&mut self) -> DecodeResult<int> { Ok(try!(self.read_f64()) as int) }

    fn read_bool(&mut self) -> DecodeResult<bool> {
        debug!("read_bool");
        Ok(try!(expect!(self.pop(), Boolean)))
    }

    fn read_f64(&mut self) -> DecodeResult<f64> {
        use std::from_str::FromStr;
        debug!("read_f64");
        match self.pop() {
            Number(f) => Ok(f),
            String(s) => {
                // re: #12967.. a type w/ numeric keys (ie HashMap<uint, V> etc)
                // is going to have a string here, as per JSON spec..
                Ok(FromStr::from_str(s.as_slice()).unwrap())
            },
            value => {
                Err(ExpectedError("Number".to_string(),
                                  format!("{}", value)))
            }
        }
    }

    fn read_f32(&mut self) -> DecodeResult<f32> { Ok(try!(self.read_f64()) as f32) }

    fn read_char(&mut self) -> DecodeResult<char> {
        let s = try!(self.read_str());
        {
            let mut it = s.as_slice().chars();
            match (it.next(), it.next()) {
                // exactly one character
                (Some(c), None) => return Ok(c),
                _ => ()
            }
        }
        Err(ExpectedError("single character string".to_string(),
                          format!("{}", s)))
    }

    fn read_str(&mut self) -> DecodeResult<String> {
        debug!("read_str");
        Ok(try!(expect!(self.pop(), String)))
    }

    fn read_enum<T>(&mut self,
                    name: &str,
                    f: |&mut Decoder| -> DecodeResult<T>) -> DecodeResult<T> {
        debug!("read_enum({})", name);
        f(self)
    }

    fn read_enum_variant<T>(&mut self,
                            names: &[&str],
                            f: |&mut Decoder, uint| -> DecodeResult<T>)
                            -> DecodeResult<T> {
        debug!("read_enum_variant(names={:?})", names);
        let name = match self.pop() {
            String(s) => s,
            Object(mut o) => {
                let n = match o.pop(&"variant".to_string()) {
                    Some(String(s)) => s,
                    Some(val) => {
                        return Err(ExpectedError("String".to_string(),
                                                 format!("{}", val)))
                    }
                    None => {
                        return Err(MissingFieldError("variant".to_string()))
                    }
                };
                match o.pop(&"fields".to_string()) {
                    Some(List(l)) => {
                        for field in l.move_iter().rev() {
                            self.stack.push(field.clone());
                        }
                    },
                    Some(val) => {
                        return Err(ExpectedError("List".to_string(),
                                                 format!("{}", val)))
                    }
                    None => {
                        return Err(MissingFieldError("fields".to_string()))
                    }
                }
                n
            }
            json => {
                return Err(ExpectedError("String or Object".to_string(),
                                         format!("{}", json)))
            }
        };
        let idx = match names.iter()
                             .position(|n| {
                                 str::eq_slice(*n, name.as_slice())
                             }) {
            Some(idx) => idx,
            None => return Err(UnknownVariantError(name))
        };
        f(self, idx)
    }

    fn read_enum_variant_arg<T>(&mut self, idx: uint, f: |&mut Decoder| -> DecodeResult<T>)
                                -> DecodeResult<T> {
        debug!("read_enum_variant_arg(idx={})", idx);
        f(self)
    }

    fn read_enum_struct_variant<T>(&mut self,
                                   names: &[&str],
                                   f: |&mut Decoder, uint| -> DecodeResult<T>)
                                   -> DecodeResult<T> {
        debug!("read_enum_struct_variant(names={:?})", names);
        self.read_enum_variant(names, f)
    }


    fn read_enum_struct_variant_field<T>(&mut self,
                                         name: &str,
                                         idx: uint,
                                         f: |&mut Decoder| -> DecodeResult<T>)
                                         -> DecodeResult<T> {
        debug!("read_enum_struct_variant_field(name={}, idx={})", name, idx);
        self.read_enum_variant_arg(idx, f)
    }

    fn read_struct<T>(&mut self,
                      name: &str,
                      len: uint,
                      f: |&mut Decoder| -> DecodeResult<T>)
                      -> DecodeResult<T> {
        debug!("read_struct(name={}, len={})", name, len);
        let value = try!(f(self));
        self.pop();
        Ok(value)
    }

    fn read_struct_field<T>(&mut self,
                            name: &str,
                            idx: uint,
                            f: |&mut Decoder| -> DecodeResult<T>)
                            -> DecodeResult<T> {
        debug!("read_struct_field(name={}, idx={})", name, idx);
        let mut obj = try!(expect!(self.pop(), Object));

        let value = match obj.pop(&name.to_string()) {
            None => return Err(MissingFieldError(name.to_string())),
            Some(json) => {
                self.stack.push(json);
                try!(f(self))
            }
        };
        self.stack.push(Object(obj));
        Ok(value)
    }

    fn read_tuple<T>(&mut self, f: |&mut Decoder, uint| -> DecodeResult<T>) -> DecodeResult<T> {
        debug!("read_tuple()");
        self.read_seq(f)
    }

    fn read_tuple_arg<T>(&mut self,
                         idx: uint,
                         f: |&mut Decoder| -> DecodeResult<T>) -> DecodeResult<T> {
        debug!("read_tuple_arg(idx={})", idx);
        self.read_seq_elt(idx, f)
    }

    fn read_tuple_struct<T>(&mut self,
                            name: &str,
                            f: |&mut Decoder, uint| -> DecodeResult<T>)
                            -> DecodeResult<T> {
        debug!("read_tuple_struct(name={})", name);
        self.read_tuple(f)
    }

    fn read_tuple_struct_arg<T>(&mut self,
                                idx: uint,
                                f: |&mut Decoder| -> DecodeResult<T>)
                                -> DecodeResult<T> {
        debug!("read_tuple_struct_arg(idx={})", idx);
        self.read_tuple_arg(idx, f)
    }

    fn read_option<T>(&mut self, f: |&mut Decoder, bool| -> DecodeResult<T>) -> DecodeResult<T> {
        match self.pop() {
            Null => f(self, false),
            value => { self.stack.push(value); f(self, true) }
        }
    }

    fn read_seq<T>(&mut self, f: |&mut Decoder, uint| -> DecodeResult<T>) -> DecodeResult<T> {
        debug!("read_seq()");
        let list = try!(expect!(self.pop(), List));
        let len = list.len();
        for v in list.move_iter().rev() {
            self.stack.push(v);
        }
        f(self, len)
    }

    fn read_seq_elt<T>(&mut self,
                       idx: uint,
                       f: |&mut Decoder| -> DecodeResult<T>) -> DecodeResult<T> {
        debug!("read_seq_elt(idx={})", idx);
        f(self)
    }

    fn read_map<T>(&mut self, f: |&mut Decoder, uint| -> DecodeResult<T>) -> DecodeResult<T> {
        debug!("read_map()");
        let obj = try!(expect!(self.pop(), Object));
        let len = obj.len();
        for (key, value) in obj.move_iter() {
            self.stack.push(value);
            self.stack.push(String(key));
        }
        f(self, len)
    }

    fn read_map_elt_key<T>(&mut self, idx: uint, f: |&mut Decoder| -> DecodeResult<T>)
                           -> DecodeResult<T> {
        debug!("read_map_elt_key(idx={})", idx);
        f(self)
    }

    fn read_map_elt_val<T>(&mut self, idx: uint, f: |&mut Decoder| -> DecodeResult<T>)
                           -> DecodeResult<T> {
        debug!("read_map_elt_val(idx={})", idx);
        f(self)
    }
}
    */

/// Test if two json values are less than one another
impl Ord for Json {
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

impl ToJson for () {
    fn to_json(&self) -> Json { Null }
}

impl ToJson for bool {
    fn to_json(&self) -> Json { Boolean(*self) }
}

impl ToJson for String {
    fn to_json(&self) -> Json { String((*self).clone()) }
}

impl<A:ToJson,B:ToJson> ToJson for (A, B) {
    fn to_json(&self) -> Json {
        match *self {
          (ref a, ref b) => {
            List(vec![a.to_json(), b.to_json()])
          }
        }
    }
}

impl<A:ToJson,B:ToJson,C:ToJson> ToJson for (A, B, C) {
    fn to_json(&self) -> Json {
        match *self {
          (ref a, ref b, ref c) => {
            List(vec![a.to_json(), b.to_json(), c.to_json()])
          }
        }
    }
}

impl<A:ToJson> ToJson for ~[A] {
    fn to_json(&self) -> Json { List(self.iter().map(|elt| elt.to_json()).collect()) }
}

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

/*
impl fmt::Show for Json {
    /// Encodes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_writer(f).map_err(|_| fmt::WriteError)
    }
}
*/

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    /*
    //use {Encodable, Decodable};
    use super::{Encoder, Decoder, Error, Boolean, Number, List, String, Null,
                PrettyEncoder, Object, Json, from_str, ParseError, ExpectedError,
                MissingFieldError, UnknownVariantError, DecodeResult, DecoderError,
                JsonEvent, Parser, StackElement,
                ObjectStart, ObjectEnd, ListStart, ListEnd, BooleanValue, NumberValue, StringValue,
                NullValue, SyntaxError, Key, Index, Stack,
                InvalidSyntax, InvalidNumber, EOFWhileParsingObject, EOFWhileParsingList,
                EOFWhileParsingValue, EOFWhileParsingString, KeyMustBeAString, ExpectedColon,
                TrailingCharacters};
    */

    use super::{
        Json,
        Null,
        Boolean,
        Number,
        String,
        List,
        Object,
    };
    use super::{Parser, ParserError, from_iter};
    use super::{JsonDeserializer, from_json, ToJson};
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

    use std::fmt::Show;
    use std::io;
    use std::str;
    use collections::TreeMap;

    macro_rules! treemap {
        ($($k:expr => $v:expr),*) => ({
            let mut _m = ::collections::TreeMap::new();
            $(_m.insert($k, $v);)*
            _m
        })
    }

    #[deriving(Eq, Show)]
    enum Animal {
        Dog,
        Frog(String, int)
    }

    impl<E, D: de::Deserializer<E>> de::Deserializable<E, D> for Animal {
        #[inline]
        fn deserialize_token(d: &mut D, token: de::Token) -> Result<Animal, E> {
            match try!(d.expect_enum_start(token, "Animal", ["Dog", "Frog"])) {
                0 => {
                    try!(d.expect_end());
                    Ok(Dog)
                }
                1 => {
                    let x0 = try!(de::Deserializable::deserialize(d));
                    let x1 = try!(de::Deserializable::deserialize(d));

                    try!(d.expect_end());

                    Ok(Frog(x0, x1))
                }
                _ => Err(d.syntax_error()),
            }
        }
    }

    #[deriving(Eq, Show)]
    struct Inner {
        a: (),
        b: uint,
        c: Vec<String>,
    }

    impl<E, D: de::Deserializer<E>> de::Deserializable<E, D> for Inner {
        #[inline]
        fn deserialize_token(d: &mut D, token: de::Token) -> Result<Inner, E> {
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
                            _ => { return Err(d.syntax_error()); }
                        }
                    }

                    match (a, b, c) {
                        (Some(a), Some(b), Some(c)) => {
                            Ok(Inner { a: a, b: b, c: c })
                        }
                        _ => Err(d.syntax_error()),
                    }
                }
                _ => Err(d.syntax_error()),
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

    #[deriving(Eq, Show)]
    struct Outer {
        inner: Vec<Inner>,
    }

    impl<E, D: de::Deserializer<E>> de::Deserializable<E, D> for Outer {
        #[inline]
        fn deserialize_token(d: &mut D, token: de::Token) -> Result<Outer, E> {
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
                            _ => { return Err(d.syntax_error()); }
                        }
                    }

                    match inner {
                        Some(inner) => {
                            Ok(Outer { inner: inner })
                        }
                        _ => Err(d.syntax_error()),
                    }
                }
                _ => Err(d.syntax_error()),
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

    /*
    fn mk_object(items: &[(String, Json)]) -> Json {
        let mut d = box TreeMap::new();

        for item in items.iter() {
            match *item {
                (ref key, ref value) => { d.insert((*key).clone(), (*value).clone()); },
            }
        };

        Object(d)
    }

    #[test]
    fn test_write_null() {
        assert_eq!(Null.to_str().into_string(), "null".to_string());
        assert_eq!(Null.to_pretty_str().into_string(), "null".to_string());
    }


    #[test]
    fn test_write_number() {
        assert_eq!(Number(3.0).to_str().into_string(), "3".to_string());
        assert_eq!(Number(3.0).to_pretty_str().into_string(), "3".to_string());

        assert_eq!(Number(3.1).to_str().into_string(), "3.1".to_string());
        assert_eq!(Number(3.1).to_pretty_str().into_string(), "3.1".to_string());

        assert_eq!(Number(-1.5).to_str().into_string(), "-1.5".to_string());
        assert_eq!(Number(-1.5).to_pretty_str().into_string(), "-1.5".to_string());

        assert_eq!(Number(0.5).to_str().into_string(), "0.5".to_string());
        assert_eq!(Number(0.5).to_pretty_str().into_string(), "0.5".to_string());
    }

    #[test]
    fn test_write_str() {
        assert_eq!(String("".to_string()).to_str().into_string(), "\"\"".to_string());
        assert_eq!(String("".to_string()).to_pretty_str().into_string(), "\"\"".to_string());

        assert_eq!(String("foo".to_string()).to_str().into_string(), "\"foo\"".to_string());
        assert_eq!(String("foo".to_string()).to_pretty_str().into_string(), "\"foo\"".to_string());
    }

    #[test]
    fn test_write_bool() {
        assert_eq!(Boolean(true).to_str().into_string(), "true".to_string());
        assert_eq!(Boolean(true).to_pretty_str().into_string(), "true".to_string());

        assert_eq!(Boolean(false).to_str().into_string(), "false".to_string());
        assert_eq!(Boolean(false).to_pretty_str().into_string(), "false".to_string());
    }

    #[test]
    fn test_write_list() {
        assert_eq!(List(vec![]).to_str().into_string(), "[]".to_string());
        assert_eq!(List(vec![]).to_pretty_str().into_string(), "[]".to_string());

        assert_eq!(List(vec![Boolean(true)]).to_str().into_string(), "[true]".to_string());
        assert_eq!(
            List(vec![Boolean(true)]).to_pretty_str().into_string(),
            "\
            [\n  \
                true\n\
            ]".to_string()
        );

        let long_test_list = List(vec![
            Boolean(false),
            Null,
            List(vec![String("foo\nbar".to_string()), Number(3.5)])]);

        assert_eq!(long_test_list.to_str().into_string(),
            "[false,null,[\"foo\\nbar\",3.5]]".to_string());
        assert_eq!(
            long_test_list.to_pretty_str().into_string(),
            "\
            [\n  \
                false,\n  \
                null,\n  \
                [\n    \
                    \"foo\\nbar\",\n    \
                    3.5\n  \
                ]\n\
            ]".to_string()
        );
    }

    #[test]
    fn test_write_object() {
        assert_eq!(mk_object([]).to_str().into_string(), "{}".to_string());
        assert_eq!(mk_object([]).to_pretty_str().into_string(), "{}".to_string());

        assert_eq!(
            mk_object([
                ("a".to_string(), Boolean(true))
            ]).to_str().into_string(),
            "{\"a\":true}".to_string()
        );
        assert_eq!(
            mk_object([("a".to_string(), Boolean(true))]).to_pretty_str(),
            "\
            {\n  \
                \"a\": true\n\
            }".to_string()
        );

        let complex_obj = mk_object([
                ("b".to_string(), List(vec![
                    mk_object([("c".to_string(), String("\x0c\r".to_string()))]),
                    mk_object([("d".to_string(), String("".to_string()))])
                ]))
            ]);

        assert_eq!(
            complex_obj.to_str().into_string(),
            "{\
                \"b\":[\
                    {\"c\":\"\\f\\r\"},\
                    {\"d\":\"\"}\
                ]\
            }".to_string()
        );
        assert_eq!(
            complex_obj.to_pretty_str().into_string(),
            "\
            {\n  \
                \"b\": [\n    \
                    {\n      \
                        \"c\": \"\\f\\r\"\n    \
                    },\n    \
                    {\n      \
                        \"d\": \"\"\n    \
                    }\n  \
                ]\n\
            }".to_string()
        );

        let a = mk_object([
            ("a".to_string(), Boolean(true)),
            ("b".to_string(), List(vec![
                mk_object([("c".to_string(), String("\x0c\r".to_string()))]),
                mk_object([("d".to_string(), String("".to_string()))])
            ]))
        ]);

        // We can't compare the strings directly because the object fields be
        // printed in a different order.
        assert_eq!(a.clone(), from_str(a.to_str().as_slice()).unwrap());
        assert_eq!(a.clone(),
                   from_str(a.to_pretty_str().as_slice()).unwrap());
    }

    fn with_str_writer(f: |&mut io::Writer|) -> String {
        use std::io::MemWriter;
        use std::str;

        let mut m = MemWriter::new();
        f(&mut m as &mut io::Writer);
        str::from_utf8(m.unwrap().as_slice()).unwrap().to_string()
    }

    #[test]
    fn test_write_enum() {
        let animal = Dog;
        assert_eq!(
            with_str_writer(|wr| {
                let mut encoder = Encoder::new(wr);
                animal.encode(&mut encoder).unwrap();
            }),
            "\"Dog\"".to_string()
        );
        assert_eq!(
            with_str_writer(|wr| {
                let mut encoder = PrettyEncoder::new(wr);
                animal.encode(&mut encoder).unwrap();
            }),
            "\"Dog\"".to_string()
        );

        let animal = Frog("Henry".to_string(), 349);
        assert_eq!(
            with_str_writer(|wr| {
                let mut encoder = Encoder::new(wr);
                animal.encode(&mut encoder).unwrap();
            }),
            "{\"variant\":\"Frog\",\"fields\":[\"Henry\",349]}".to_string()
        );
        assert_eq!(
            with_str_writer(|wr| {
                let mut encoder = PrettyEncoder::new(wr);
                animal.encode(&mut encoder).unwrap();
            }),
            "\
            [\n  \
                \"Frog\",\n  \
                \"Henry\",\n  \
                349\n\
            ]".to_string()
        );
    }

    #[test]
    fn test_write_some() {
        let value = Some("jodhpurs".to_string());
        let s = with_str_writer(|wr| {
            let mut encoder = Encoder::new(wr);
            value.encode(&mut encoder).unwrap();
        });
        assert_eq!(s, "\"jodhpurs\"".to_string());

        let value = Some("jodhpurs".to_string());
        let s = with_str_writer(|wr| {
            let mut encoder = PrettyEncoder::new(wr);
            value.encode(&mut encoder).unwrap();
        });
        assert_eq!(s, "\"jodhpurs\"".to_string());
    }

    #[test]
    fn test_write_none() {
        let value: Option<String> = None;
        let s = with_str_writer(|wr| {
            let mut encoder = Encoder::new(wr);
            value.encode(&mut encoder).unwrap();
        });
        assert_eq!(s, "null".to_string());

        let s = with_str_writer(|wr| {
            let mut encoder = Encoder::new(wr);
            value.encode(&mut encoder).unwrap();
        });
        assert_eq!(s, "null".to_string());
    }
    #[test]
    fn test_read_identifiers() {
        ("n", SyntaxError(InvalidSyntax, 1, 2)),
        ("nul", SyntaxError(InvalidSyntax, 1, 4)),
        ("t", SyntaxError(InvalidSyntax, 1, 2)),
        ("truz", SyntaxError(InvalidSyntax, 1, 4)),
        ("f", SyntaxError(InvalidSyntax, 1, 2)),
        ("faz", SyntaxError(InvalidSyntax, 1, 3)),

        assert_eq!(from_str("null"), Ok(Null));
        assert_eq!(from_str("true"), Ok(Boolean(true)));
        assert_eq!(from_str("false"), Ok(Boolean(false)));
        assert_eq!(from_str(" null "), Ok(Null));
        assert_eq!(from_str(" true "), Ok(Boolean(true)));
        assert_eq!(from_str(" false "), Ok(Boolean(false)));
    }
    */

    fn test_parse_err<
        'a,
        T: Eq + Show + de::Deserializable<ParserError, Parser<str::Chars<'a>>>
    >(errors: &[(&'a str, ParserError)]) {
        for &(s, ref err) in errors.iter() {
            let v: Result<T, ParserError> = from_iter(s.chars());
            assert_eq!(v.unwrap_err(), *err);
        }
    }

    fn test_parse_ok<
        'a,
        T: Eq + Show + ToJson + de::Deserializable<ParserError, Parser<str::Chars<'a>>>
    >(errors: &[(&'a str, T)]) {
        for &(s, ref value) in errors.iter() {
            let v: T = from_iter(s.chars()).unwrap();
            assert_eq!(v, *value);

            let v: Json = from_iter(s.chars()).unwrap();
            assert_eq!(v, value.to_json());
        }
    }

    fn test_json_deserialize_ok<
        T: Eq + Show + ToJson + de::Deserializable<ParserError, JsonDeserializer>
    >(errors: &[T]) {
        for value in errors.iter() {
            let v: T = from_json(value.to_json()).unwrap();
            assert_eq!(v, *value);
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

    /*
    #[test]
    fn test_read_number() {
        assert_eq!(from_str("+"),   Err(SyntaxError(InvalidSyntax, 1, 1)));
        assert_eq!(from_str("."),   Err(SyntaxError(InvalidSyntax, 1, 1)));
        assert_eq!(from_str("-"),   Err(SyntaxError(InvalidNumber, 1, 2)));
        assert_eq!(from_str("00"),  Err(SyntaxError(InvalidNumber, 1, 2)));
        assert_eq!(from_str("1."),  Err(SyntaxError(InvalidNumber, 1, 3)));
        assert_eq!(from_str("1e"),  Err(SyntaxError(InvalidNumber, 1, 3)));
        assert_eq!(from_str("1e+"), Err(SyntaxError(InvalidNumber, 1, 4)));

        assert_eq!(from_str("3"), Ok(Number(3.0)));
        assert_eq!(from_str("3.1"), Ok(Number(3.1)));
        assert_eq!(from_str("-1.2"), Ok(Number(-1.2)));
        assert_eq!(from_str("0.4"), Ok(Number(0.4)));
        assert_eq!(from_str("0.4e5"), Ok(Number(0.4e5)));
        assert_eq!(from_str("0.4e+15"), Ok(Number(0.4e15)));
        assert_eq!(from_str("0.4e-01"), Ok(Number(0.4e-01)));
        assert_eq!(from_str(" 3 "), Ok(Number(3.0)));
    }
    */

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

    /*
    #[test]
    fn test_read_str() {
        assert_eq!(from_str("\""),    Err(SyntaxError(EOFWhileParsingString, 1, 2)));
        assert_eq!(from_str("\"lol"), Err(SyntaxError(EOFWhileParsingString, 1, 5)));

        assert_eq!(from_str("\"\""), Ok(String("".to_string())));
        assert_eq!(from_str("\"foo\""), Ok(String("foo".to_string())));
        assert_eq!(from_str("\"\\\"\""), Ok(String("\"".to_string())));
        assert_eq!(from_str("\"\\b\""), Ok(String("\x08".to_string())));
        assert_eq!(from_str("\"\\n\""), Ok(String("\n".to_string())));
        assert_eq!(from_str("\"\\r\""), Ok(String("\r".to_string())));
        assert_eq!(from_str("\"\\t\""), Ok(String("\t".to_string())));
        assert_eq!(from_str(" \"foo\" "), Ok(String("foo".to_string())));
        assert_eq!(from_str("\"\\u12ab\""), Ok(String("\u12ab".to_string())));
        assert_eq!(from_str("\"\\uAB12\""), Ok(String("\uAB12".to_string())));
    }
    */

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

    /*
    #[test]
    fn test_read_list() {
        assert_eq!(from_str("["),     Err(SyntaxError(EOFWhileParsingValue, 1, 2)));
        assert_eq!(from_str("[1"),    Err(SyntaxError(EOFWhileParsingList,  1, 3)));
        assert_eq!(from_str("[1,"),   Err(SyntaxError(EOFWhileParsingValue, 1, 4)));
        assert_eq!(from_str("[1,]"),  Err(SyntaxError(InvalidSyntax,        1, 4)));
        assert_eq!(from_str("[6 7]"), Err(SyntaxError(InvalidSyntax,        1, 4)));

        assert_eq!(from_str("[]"), Ok(List(vec![])));
        assert_eq!(from_str("[ ]"), Ok(List(vec![])));
        assert_eq!(from_str("[true]"), Ok(List(vec![Boolean(true)])));
        assert_eq!(from_str("[ false ]"), Ok(List(vec![Boolean(false)])));
        assert_eq!(from_str("[null]"), Ok(List(vec![Null])));
        assert_eq!(from_str("[3, 1]"),
                     Ok(List(vec![Number(3.0), Number(1.0)])));
        assert_eq!(from_str("\n[3, 2]\n"),
                     Ok(List(vec![Number(3.0), Number(2.0)])));
        assert_eq!(from_str("[2, [4, 1]]"),
               Ok(List(vec![Number(2.0), List(vec![Number(4.0), Number(1.0)])])));
    }
    */

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

    /*
    #[test]
    fn test_read_object() {
        assert_eq!(from_str("{"),       Err(SyntaxError(EOFWhileParsingObject, 1, 2)));
        assert_eq!(from_str("{ "),      Err(SyntaxError(EOFWhileParsingObject, 1, 3)));
        assert_eq!(from_str("{1"),      Err(SyntaxError(KeyMustBeAString,      1, 2)));
        assert_eq!(from_str("{ \"a\""), Err(SyntaxError(EOFWhileParsingObject, 1, 6)));
        assert_eq!(from_str("{\"a\""),  Err(SyntaxError(EOFWhileParsingObject, 1, 5)));
        assert_eq!(from_str("{\"a\" "), Err(SyntaxError(EOFWhileParsingObject, 1, 6)));

        assert_eq!(from_str("{\"a\" 1"),   Err(SyntaxError(ExpectedColon,         1, 6)));
        assert_eq!(from_str("{\"a\":"),    Err(SyntaxError(EOFWhileParsingValue,  1, 6)));
        assert_eq!(from_str("{\"a\":1"),   Err(SyntaxError(EOFWhileParsingObject, 1, 7)));
        assert_eq!(from_str("{\"a\":1 1"), Err(SyntaxError(InvalidSyntax,         1, 8)));
        assert_eq!(from_str("{\"a\":1,"),  Err(SyntaxError(EOFWhileParsingObject, 1, 8)));

        assert_eq!(from_str("{}").unwrap(), mk_object([]));
        assert_eq!(from_str("{\"a\": 3}").unwrap(),
                  mk_object([("a".to_string(), Number(3.0))]));

        assert_eq!(from_str(
                      "{ \"a\": null, \"b\" : true }").unwrap(),
                  mk_object([
                      ("a".to_string(), Null),
                      ("b".to_string(), Boolean(true))]));
        assert_eq!(from_str("\n{ \"a\": null, \"b\" : true }\n").unwrap(),
                  mk_object([
                      ("a".to_string(), Null),
                      ("b".to_string(), Boolean(true))]));
        assert_eq!(from_str(
                      "{\"a\" : 1.0 ,\"b\": [ true ]}").unwrap(),
                  mk_object([
                      ("a".to_string(), Number(1.0)),
                      ("b".to_string(), List(vec![Boolean(true)]))
                  ]));
        assert_eq!(from_str(
                      "{\
                          \"a\": 1.0, \
                          \"b\": [\
                              true,\
                              \"foo\\nbar\", \
                              { \"c\": {\"d\": null} } \
                          ]\
                      }").unwrap(),
                  mk_object([
                      ("a".to_string(), Number(1.0)),
                      ("b".to_string(), List(vec![
                          Boolean(true),
                          String("foo\nbar".to_string()),
                          mk_object([
                              ("c".to_string(), mk_object([("d".to_string(), Null)]))
                          ])
                      ]))
                  ]));
    }
    */

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

        /*
    #[test]
    fn test_decode_enum() {
        let value: Result<Animal, ParserError> = from_iter("\"Dog\"".chars());
        assert_eq!(value, Ok(Dog));

        let s = "{\"variant\":\"Frog\",\"fields\":[\"Henry\",349]}";
        let value: Result<Animal, ParserError> = from_iter(s.chars());
        assert_eq!(value, Ok(Frog("Henry".to_string(), 349)));

        let mut decoder = Decoder::new(from_str("\"Dog\"").unwrap());
        let value: Animal = Decodable::decode(&mut decoder).unwrap();
        assert_eq!(value, Dog);

        let s = "{\"variant\":\"Frog\",\"fields\":[\"Henry\",349]}";
        let mut decoder = Decoder::new(from_str(s).unwrap());
        let value: Animal = Decodable::decode(&mut decoder).unwrap();
        assert_eq!(value, Frog("Henry".to_string(), 349));
        assert_eq!(value, Dog);
    }
        */

    /*
    #[test]
    fn test_decode_map() {
        let s = "{\"a\": \"Dog\", \"b\": {\"variant\":\"Frog\",\
                  \"fields\":[\"Henry\", 349]}}";
        let mut decoder = Decoder::new(from_str(s).unwrap());
        let mut map: TreeMap<String, Animal> = Decodable::decode(&mut decoder).unwrap();

        assert_eq!(map.pop(&"a".to_string()), Some(Dog));
        assert_eq!(map.pop(&"b".to_string()), Some(Frog("Henry".to_string(), 349)));
    }

    #[test]
    fn test_multiline_errors() {
        assert_eq!(from_str("{\n  \"foo\":\n \"bar\""),
            Err(SyntaxError(EOFWhileParsingObject, 3u, 8u)));
    }

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

    #[test]
    fn test_find(){
        let json_value = from_str("{\"dog\" : \"cat\"}").unwrap();
        let found_str = json_value.find(&"dog".to_string());
        assert!(found_str.is_some() && found_str.unwrap().as_string().unwrap() == "cat");
    }

    #[test]
    fn test_find_path(){
        let json_value = from_str("{\"dog\":{\"cat\": {\"mouse\" : \"cheese\"}}}").unwrap();
        let found_str = json_value.find_path(&[&"dog".to_string(),
                                             &"cat".to_string(), &"mouse".to_string()]);
        assert!(found_str.is_some() && found_str.unwrap().as_string().unwrap() == "cheese");
    }

    #[test]
    fn test_search(){
        let json_value = from_str("{\"dog\":{\"cat\": {\"mouse\" : \"cheese\"}}}").unwrap();
        let found_str = json_value.search(&"mouse".to_string()).and_then(|j| j.as_string());
        assert!(found_str.is_some());
        assert!(found_str.unwrap() == "cheese");
    }

    #[test]
    fn test_is_object(){
        let json_value = from_str("{}").unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_as_object(){
        let json_value = from_str("{}").unwrap();
        let json_object = json_value.as_object();
        assert!(json_object.is_some());
    }

    #[test]
    fn test_is_list(){
        let json_value = from_str("[1, 2, 3]").unwrap();
        assert!(json_value.is_list());
    }

    #[test]
    fn test_as_list(){
        let json_value = from_str("[1, 2, 3]").unwrap();
        let json_list = json_value.as_list();
        let expected_length = 3;
        assert!(json_list.is_some() && json_list.unwrap().len() == expected_length);
    }

    #[test]
    fn test_is_string(){
        let json_value = from_str("\"dog\"").unwrap();
        assert!(json_value.is_string());
    }

    #[test]
    fn test_as_string(){
        let json_value = from_str("\"dog\"").unwrap();
        let json_str = json_value.as_string();
        let expected_str = "dog";
        assert_eq!(json_str, Some(expected_str));
    }

    #[test]
    fn test_is_number(){
        let json_value = from_str("12").unwrap();
        assert!(json_value.is_number());
    }

    #[test]
    fn test_as_number(){
        let json_value = from_str("12").unwrap();
        let json_num = json_value.as_number();
        let expected_num = 12f64;
        assert!(json_num.is_some() && json_num.unwrap() == expected_num);
    }

    #[test]
    fn test_is_boolean(){
        let json_value = from_str("false").unwrap();
        assert!(json_value.is_boolean());
    }

    #[test]
    fn test_as_boolean(){
        let json_value = from_str("false").unwrap();
        let json_bool = json_value.as_boolean();
        let expected_bool = false;
        assert!(json_bool.is_some() && json_bool.unwrap() == expected_bool);
    }

    #[test]
    fn test_is_null(){
        let json_value = from_str("null").unwrap();
        assert!(json_value.is_null());
    }

    #[test]
    fn test_as_null(){
        let json_value = from_str("null").unwrap();
        let json_null = json_value.as_null();
        let expected_null = ();
        assert!(json_null.is_some() && json_null.unwrap() == expected_null);
    }

    #[test]
    fn test_encode_hashmap_with_numeric_key() {
        use std::str::from_utf8;
        use std::io::Writer;
        use std::io::MemWriter;
        use collections::HashMap;
        let mut hm: HashMap<uint, bool> = HashMap::new();
        hm.insert(1, true);
        let mut mem_buf = MemWriter::new();
        {
            let mut encoder = Encoder::new(&mut mem_buf as &mut io::Writer);
            hm.encode(&mut encoder).unwrap();
        }
        let bytes = mem_buf.unwrap();
        let json_str = from_utf8(bytes.as_slice()).unwrap();
        match from_str(json_str) {
            Err(_) => fail!("Unable to parse json_str: {:?}", json_str),
            _ => {} // it parsed and we are good to go
        }
    }
    #[test]
    fn test_prettyencode_hashmap_with_numeric_key() {
        use std::str::from_utf8;
        use std::io::Writer;
        use std::io::MemWriter;
        use collections::HashMap;
        let mut hm: HashMap<uint, bool> = HashMap::new();
        hm.insert(1, true);
        let mut mem_buf = MemWriter::new();
        {
            let mut encoder = PrettyEncoder::new(&mut mem_buf as &mut io::Writer);
            hm.encode(&mut encoder).unwrap()
        }
        let bytes = mem_buf.unwrap();
        let json_str = from_utf8(bytes.as_slice()).unwrap();
        match from_str(json_str) {
            Err(_) => fail!("Unable to parse json_str: {:?}", json_str),
            _ => {} // it parsed and we are good to go
        }
    }
    #[test]
    fn test_hashmap_with_numeric_key_can_handle_double_quote_delimited_key() {
        use collections::HashMap;
        use Decodable;
        let json_str = "{\"1\":true}";
        let json_obj = match from_str(json_str) {
            Err(_) => fail!("Unable to parse json_str: {:?}", json_str),
            Ok(o) => o
        };
        let mut decoder = Decoder::new(json_obj);
        let _hm: HashMap<uint, bool> = Decodable::decode(&mut decoder).unwrap();
    }

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

    #[bench]
    fn bench_streaming_small(b: &mut Bencher) {
        b.iter( || {
            let mut parser = Parser::new(
                r#"{
                    "a": 1.0,
                    "b": [
                        true,
                        "foo\nbar",
                        { "c": {"d": null} }
                    ]
                }"#.chars()
            );
            loop {
                match parser.next() {
                    None => return,
                    _ => {}
                }
            }
        });
    }
    #[bench]
    fn bench_small(b: &mut Bencher) {
        b.iter( || {
            let _ = from_str(r#"{
                "a": 1.0,
                "b": [
                    true,
                    "foo\nbar",
                    { "c": {"d": null} }
                ]
            }"#);
        });
    }

    fn big_json() -> String {
        let mut src = "[\n".to_string();
        for _ in range(0, 500) {
            src.push_str(r#"{ "a": true, "b": null, "c":3.1415, "d": "Hello world", "e": \
                            [1,2,3]},"#);
        }
        src.push_str("{}]");
        return src;
    }

    #[bench]
    fn bench_streaming_large(b: &mut Bencher) {
        let src = big_json();
        b.iter( || {
            let mut parser = Parser::new(src.as_slice().chars());
            loop {
                match parser.next() {
                    None => return,
                    _ => {}
                }
            }
        });
    }
    #[bench]
    fn bench_large(b: &mut Bencher) {
        let src = big_json();
        b.iter( || { let _ = from_str(src.as_slice()); });
    }
*/
}
