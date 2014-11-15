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

JSON API provide an enum `json::Value` and a trait `ToJson` to serialize
object.  The trait `ToJson` serialize object into a container `json::Value` and
the API provide writer to serialize them into a stream or a string ...

When using `ToJson` the `Serialize` trait implementation is not mandatory.

A basic `ToJson` example using a TreeMap of attribute name / attribute value:


```rust
#![feature(phase)]
#[phase(plugin)]
extern crate serde_macros;
extern crate serde;

use std::collections::TreeMap;
use serde::json::{ToJson, Value};

pub struct MyStruct  {
    attr1: u8,
    attr2: String,
}

impl ToJson for MyStruct {
    fn to_json( &self ) -> Value {
        let mut d = TreeMap::new();
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
#![feature(phase)]
#[phase(plugin)]
extern crate serde_macros;
extern crate serde;

use serde::json::{ObjectBuilder, ToJson, Value};

pub struct MyStruct  {
    attr1: u8,
    attr2: String,
}

impl ToJson for MyStruct {
    fn to_json( &self ) -> Value {
        ObjectBuilder::new()
            .insert("attr1", &self.attr1)
            .insert("attr2", &self.attr2)
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
    fn to_json( &self ) -> json::Value {
        json::builder::ObjectBuilder::new()
            .insert("data_int", &self.data_int)
            .insert("data_str", &self.data_str)
            .insert("data_vector", &self.data_vector)
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
    let json_str: String = json.to_string().into_string();

    // Deserialize like before.

    let mut parser = json::Parser::new(json_str.as_slice().bytes());
    let deserialized: TestStruct1 = Deserialize::deserialize(&mut parser).unwrap();
}
```

*/

use std::error;
use std::fmt;
use std::io;
use std::string;

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
pub use self::de::{
    Parser,
    from_str,
};
pub use self::value::{Value, ToJson};
pub use self::builder::{ListBuilder, ObjectBuilder};

pub mod builder;
pub mod de;
pub mod ser;
pub mod value;

/// The errors that can arise while parsing a JSON stream.
#[deriving(Clone, PartialEq)]
pub enum ErrorCode {
    ConversionError(super::de::Token),
    EOFWhileParsingList,
    EOFWhileParsingObject,
    EOFWhileParsingString,
    EOFWhileParsingValue,
    ExpectedColon,
    ExpectedConversion,
    ExpectedEnumEnd,
    ExpectedEnumEndToken,
    ExpectedEnumMapStart,
    ExpectedEnumToken,
    ExpectedEnumVariantString,
    ExpectedListCommaOrEnd,
    ExpectedName,
    ExpectedObjectCommaOrEnd,
    ExpectedSomeIdent,
    ExpectedSomeValue,
    ExpectedTokens(super::de::Token, &'static [super::de::TokenKind]),
    InvalidEscape,
    InvalidNumber,
    InvalidUnicodeCodePoint,
    KeyMustBeAString,
    LoneLeadingSurrogateInHexEscape,
    MissingField(&'static str),
    NotFourDigit,
    NotUtf8,
    TrailingCharacters,
    UnexpectedEndOfHexEscape,
    UnexpectedName(super::de::Token),
    UnknownVariant,
    UnrecognizedHex,
}

impl fmt::Show for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConversionError(ref token) => write!(f, "failed to convert {}", token),
            EOFWhileParsingList => "EOF While parsing list".fmt(f),
            EOFWhileParsingObject => "EOF While parsing object".fmt(f),
            EOFWhileParsingString => "EOF While parsing string".fmt(f),
            EOFWhileParsingValue => "EOF While parsing value".fmt(f),
            ExpectedColon => "expected `:`".fmt(f),
            ExpectedConversion => "expected conversion".fmt(f),
            ExpectedEnumEnd => "expected enum end".fmt(f),
            ExpectedEnumEndToken => "expected enum map end".fmt(f),
            ExpectedEnumMapStart => "expected enum map start".fmt(f),
            ExpectedEnumToken => "expected enum token".fmt(f),
            ExpectedEnumVariantString => "expected variant".fmt(f),
            ExpectedListCommaOrEnd => "expected `,` or `]`".fmt(f),
            ExpectedName => "expected name".fmt(f),
            ExpectedObjectCommaOrEnd => "expected `,` or `}`".fmt(f),
            ExpectedSomeIdent => "expected ident".fmt(f),
            ExpectedSomeValue => "expected value".fmt(f),
            ExpectedTokens(ref token, tokens) => write!(f, "expected {}, found {}", tokens, token),
            InvalidEscape => "invalid escape".fmt(f),
            InvalidNumber => "invalid number".fmt(f),
            InvalidUnicodeCodePoint => "invalid unicode code point".fmt(f),
            KeyMustBeAString => "key must be a string".fmt(f),
            LoneLeadingSurrogateInHexEscape => "lone leading surrogate in hex escape".fmt(f),
            MissingField(ref field) => write!(f, "missing field \"{}\"", field),
            NotFourDigit => "invalid \\u escape (not four digits)".fmt(f),
            NotUtf8 => "contents not utf-8".fmt(f),
            TrailingCharacters => "trailing characters".fmt(f),
            UnexpectedEndOfHexEscape => "unexpected end of hex escape".fmt(f),
            UnexpectedName(ref name) => write!(f, "unexpected name {}", name),
            UnknownVariant => "unknown variant".fmt(f),
            UnrecognizedHex => "invalid \\u escape (unrecognized hex)".fmt(f),
        }
    }
}

#[deriving(Clone, PartialEq, Show)]
pub enum Error {
    /// msg, line, col
    SyntaxError(ErrorCode, uint, uint),
    IoError(io::IoError),
    ExpectedError(string::String, string::String),
    MissingFieldError(string::String),
    UnknownVariantError(string::String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            SyntaxError(..) => "syntax error",
            IoError(ref error) => error.description(),
            ExpectedError(ref expected, _) => expected.as_slice(),
            MissingFieldError(_) => "missing field",
            UnknownVariantError(_) => "unknown variant",
        }
    }

    fn detail(&self) -> Option<String> {
        match *self {
            SyntaxError(ref code, line, col) => {
                Some(format!("{} at line {} column {}", code, line, col))
            }
            IoError(ref error) => error.detail(),
            ExpectedError(ref expected, ref found) => {
                Some(format!("expected {}, found {}", expected, found))
            }
            MissingFieldError(ref field) => {
                Some(format!("missing field {}", field))
            }
            UnknownVariantError(ref variant) => {
                Some(format!("unknown variant {}", variant))
            }
        }
    }
}

impl error::FromError<io::IoError> for Error {
    fn from_error(error: io::IoError) -> Error {
        IoError(error)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Show;
    use std::io;
    use std::str;
    use std::string;
    use std::collections::TreeMap;

    use super::value::{
        Value,
        Null,
        Boolean,
        Floating,
        String,
        List,
        Object,
    };
    use super::{Parser, Error, from_str};
    use super::value;
    use super::value::{ToJson, from_json};
    use super::{
        EOFWhileParsingList,
        EOFWhileParsingObject,
        EOFWhileParsingString,
        EOFWhileParsingValue,
        ExpectedColon,
        ExpectedListCommaOrEnd,
        ExpectedObjectCommaOrEnd,
        ExpectedSomeIdent,
        ExpectedSomeValue,
        InvalidNumber,
        KeyMustBeAString,
        SyntaxError,
        TrailingCharacters,
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
        fn to_json(&self) -> Value {
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
        fn to_json(&self) -> Value {
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
        fn to_json(&self) -> Value {
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
        T: Show + de::Deserialize<Parser<str::Bytes<'a>>, Error>
    >(errors: &[(&'a str, Error)]) {
        for &(s, ref err) in errors.iter() {
            let v: Result<T, Error> = from_str(s);
            assert_eq!(v.unwrap_err(), *err);
        }
    }

    fn test_parse_ok<
        'a,
        T: PartialEq + Show + ToJson + de::Deserialize<Parser<str::Bytes<'a>>, Error>
    >(errors: &[(&'a str, T)]) {
        for &(s, ref value) in errors.iter() {
            let v: T = from_str(s).unwrap();
            assert_eq!(v, *value);

            let v: Value = from_str(s).unwrap();
            assert_eq!(v, value.to_json());
        }
    }

    fn test_json_deserialize_ok<
        T: PartialEq + Show + ToJson + de::Deserialize<value::Deserializer, Error>
    >(errors: &[T]) {
        for value in errors.iter() {
            let v: T = from_json(value.to_json()).unwrap();
            assert_eq!(v, *value);

            // Make sure we can round trip back to `Json`.
            let v: Value = from_json(value.to_json()).unwrap();
            assert_eq!(v, value.to_json());
        }
    }

    #[test]
    fn test_parse_null() {
        test_parse_err::<()>([
            ("n", SyntaxError(ExpectedSomeIdent, 1, 2)),
            ("nul", SyntaxError(ExpectedSomeIdent, 1, 4)),
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
            ("t", SyntaxError(ExpectedSomeIdent, 1, 2)),
            ("truz", SyntaxError(ExpectedSomeIdent, 1, 4)),
            ("f", SyntaxError(ExpectedSomeIdent, 1, 2)),
            ("faz", SyntaxError(ExpectedSomeIdent, 1, 3)),
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
            ("+", SyntaxError(ExpectedSomeValue, 1, 1)),
            (".", SyntaxError(ExpectedSomeValue, 1, 1)),
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
            ("[1,]", SyntaxError(ExpectedSomeValue, 1, 4)),
            ("[1 2]", SyntaxError(ExpectedListCommaOrEnd, 1, 4)),
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
            ("{\"a\":1 1", SyntaxError(ExpectedObjectCommaOrEnd, 1, 8)),
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
        let json_value: Value = from_str("{\"dog\" : \"cat\"}").unwrap();
        let found_str = json_value.find(&"dog".to_string());
        assert!(found_str.is_some() && found_str.unwrap().as_string().unwrap() == "cat");
    }

    #[test]
    fn test_find_path(){
        let json_value: Value = from_str("{\"dog\":{\"cat\": {\"mouse\" : \"cheese\"}}}").unwrap();
        let found_str = json_value.find_path(&[&"dog".to_string(),
                                             &"cat".to_string(), &"mouse".to_string()]);
        assert!(found_str.is_some() && found_str.unwrap().as_string().unwrap() == "cheese");
    }

    #[test]
    fn test_search(){
        let json_value: Value = from_str("{\"dog\":{\"cat\": {\"mouse\" : \"cheese\"}}}").unwrap();
        let found_str = json_value.search(&"mouse".to_string()).and_then(|j| j.as_string());
        assert!(found_str.is_some());
        assert!(found_str.unwrap() == "cheese");
    }

    #[test]
    fn test_is_object() {
        let json_value: Value = from_str("{}").unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_as_object() {
        let json_value: Value = from_str("{}").unwrap();
        let json_object = json_value.as_object();
        let map = TreeMap::<string::String, Value>::new();
        assert_eq!(json_object, Some(&map));
    }

    #[test]
    fn test_is_list() {
        let json_value: Value = from_str("[1, 2, 3]").unwrap();
        assert!(json_value.is_list());
    }

    #[test]
    fn test_as_list() {
        let json_value: Value = from_str("[1, 2, 3]").unwrap();
        let json_list = json_value.as_list();
        let expected_length = 3;
        assert!(json_list.is_some() && json_list.unwrap().len() == expected_length);
    }

    #[test]
    fn test_is_string() {
        let json_value: Value = from_str("\"dog\"").unwrap();
        assert!(json_value.is_string());
    }

    #[test]
    fn test_as_string() {
        let json_value: Value = from_str("\"dog\"").unwrap();
        let json_str = json_value.as_string();
        let expected_str = "dog";
        assert_eq!(json_str, Some(expected_str));
    }

    #[test]
    fn test_is_number() {
        let json_value: Value = from_str("12").unwrap();
        assert!(json_value.is_number());

        let json_value: Value = from_str("12.0").unwrap();
        assert!(json_value.is_number());
    }

    #[test]
    fn test_is_i64() {
        let json_value: Value = from_str("12").unwrap();
        assert!(json_value.is_i64());

        let json_value: Value = from_str("12.0").unwrap();
        assert!(!json_value.is_i64());
    }

    #[test]
    fn test_is_f64() {
        let json_value: Value = from_str("12").unwrap();
        assert!(!json_value.is_f64());

        let json_value: Value = from_str("12.0").unwrap();
        assert!(json_value.is_f64());
    }

    #[test]
    fn test_as_i64() {
        let json_value: Value = from_str("12").unwrap();
        assert_eq!(json_value.as_i64(), Some(12));
    }

    #[test]
    fn test_as_f64() {
        let json_value: Value = from_str("12").unwrap();
        assert_eq!(json_value.as_f64(), Some(12.0));
    }

    #[test]
    fn test_is_boolean() {
        let json_value: Value = from_str("false").unwrap();
        assert!(json_value.is_boolean());
    }

    #[test]
    fn test_as_boolean() {
        let json_value: Value = from_str("false").unwrap();
        let json_bool = json_value.as_boolean();
        let expected_bool = false;
        assert!(json_bool.is_some() && json_bool.unwrap() == expected_bool);
    }

    #[test]
    fn test_is_null() {
        let json_value: Value = from_str("null").unwrap();
        assert!(json_value.is_null());
    }

    #[test]
    fn test_as_null() {
        let json_value: Value = from_str("null").unwrap();
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
        let _json_value: Value = from_str(json_str).unwrap();
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
        let _json_value: Value = from_str(json_str).unwrap();
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

    use json::value::{
        Value,
        Null,
        Boolean,
        Integer,
        Floating,
        String,
        List,
        Object,
    };
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

    fn serializer_json(count: uint) -> Value {
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
