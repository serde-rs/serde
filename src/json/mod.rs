//! JSON and serialization
//!
//! # What is JSON?
//!
//! JSON (JavaScript Object Notation) is a way to write data in JavaScript.  Like XML, it allows to
//! encode structured data in a text format that can be easily read by humans.  Its simple syntax
//! and native compatibility with JavaScript have made it a widely used format.
//!
//! Data types that can be encoded are JavaScript types (see the `serde::json:Value` enum for more
//! details):
//!
//! * `Boolean`: equivalent to rust's `bool`
//! * `I64`: equivalent to rust's `i64`
//! * `U64`: equivalent to rust's `u64`
//! * `F64`: equivalent to rust's `i64`
//! * `String`: equivalent to rust's `String`
//! * `Array`: equivalent to rust's `Vec<T>`, but also allowing objects of different types in the
//!    same array
//! * `Object`: equivalent to rust's `BTreeMap<String, serde::json::Value>`
//! * `Null`
//!
//! An object is a series of string keys mapping to values, in `"key": value` format.  Arrays are
//! enclosed in square brackets ([ ... ]) and objects in curly brackets ({ ... }).  A simple JSON
//! document encoding a person, his/her age, address and phone numbers could look like
//!
//! ```ignore
//! {
//!     "FirstName": "John",
//!     "LastName": "Doe",
//!     "Age": 43,
//!     "Address": {
//!         "Street": "Downing Street 10",
//!         "City": "London",
//!         "Country": "Great Britain"
//!     },
//!     "PhoneNumbers": [
//!         "+44 1234567",
//!         "+44 2345678"
//!     ]
//! }
//! ```
//!
//! # Type-based Serialization and Deserialization
//!
//! Serde provides a mechanism for low boilerplate serialization & deserialization of values to and
//! from JSON via the serialization API.  To be able to serialize a piece of data, it must implement
//! the `serde::Serialize` trait.  To be able to deserialize a piece of data, it must implement the
//! `serde::Deserialize` trait.  Serde provides provides an annotation to automatically generate
//! the code for these traits: `#[derive_serialize]` and `#[derive_deserialize]`.
//!
//! The JSON API also provides an enum `serde::json::Value` and a method `to_value` to serialize
//! objects.  A `serde::json::Value` value can be serialized as a string or buffer using the
//! functions described above.  You can also use the `json::Encoder` object, which implements the
//! `Encoder` trait.
//!
//! # Examples of use
//!
//! ## Using Autoserialization
//!
//! Create a struct called `TestStruct` and serialize and deserialize it to and from JSON using the
//! serialization API, using the derived serialization code.
//!
//! ```rust
//! // Required to use the annotations.
//! #![feature(custom_derive, plugin)]
//! #![plugin(serde_macros)]
//!
//! extern crate serde;
//!
//! use serde::json;
//!
//! // Automatically generate `Serialize` and `Deserialize` trait implementations
//! #[derive_serialize]
//! #[derive_deserialize]
//! pub struct TestStruct  {
//!     data_int: u8,
//!     data_str: String,
//!     data_vector: Vec<u8>,
//! }
//!
//! fn main() {
//!     let object = TestStruct {
//!         data_int: 1,
//!         data_str: "homura".to_string(),
//!         data_vector: vec![2,3,4,5],
//!     };
//!
//!     // Serialize using `json::to_string`
//!     let serialized = json::to_string(&object).unwrap();
//!
//!     // Deserialize using `json::from_str`
//!     let deserialized: TestStruct = json::from_str(&serialized).unwrap();
//! }
//! ```
//!
//! ## Parsing a `str` to `Value` and reading the result
//!
//! ```rust
//! #![feature(custom_derive, plugin)]
//! #![plugin(serde_macros)]
//!
//! extern crate serde;
//!
//! use serde::json::{self, Value};
//!
//! fn main() {
//!     let data: Value = json::from_str("{\"foo\": 13, \"bar\": \"baz\"}").unwrap();
//!     println!("data: {:?}", data);
//!     // data: {"bar":"baz","foo":13}
//!     println!("object? {}", data.is_object());
//!     // object? true
//!
//!     let obj = data.as_object().unwrap();
//!     let foo = obj.get("foo").unwrap();
//!
//!     println!("array? {:?}", foo.as_array());
//!     // array? None
//!     println!("u64? {:?}", foo.as_u64());
//!     // u64? Some(13u64)
//!
//!     for (key, value) in obj.iter() {
//!         println!("{}: {}", key, match *value {
//!             Value::U64(v) => format!("{} (u64)", v),
//!             Value::String(ref v) => format!("{} (string)", v),
//!             _ => format!("other")
//!         });
//!     }
//!     // bar: baz (string)
//!     // foo: 13 (u64)
//! }
//! ```

pub use self::de::{Deserializer, from_str};
pub use self::error::{Error, ErrorCode};
pub use self::ser::{
    Serializer,
    to_writer,
    to_writer_pretty,
    to_vec,
    to_vec_pretty,
    to_string,
    to_string_pretty,
    escape_str,
};
pub use self::value::{Value, to_value, from_value};

pub mod builder;
pub mod de;
pub mod error;
pub mod ser;
pub mod value;
