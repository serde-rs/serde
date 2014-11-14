use std::collections::{HashMap, TreeMap, tree_map};
use std::fmt;
use std::io::{AsRefWriter, IoResult, MemWriter};
use std::io;
use std::str;
use std::string;
use std::vec;

use de;
use ser::Serialize;
use ser;

use super::PrettySerializer;
use super::Serializer;
use super::SerializeResult;
use super::ParserError;
use super::{
    MissingFieldError,
    SyntaxError,
    DeserializerError,
    ExpectName,
    ExpectConversion,
    ExpectTokens,
    EOFWhileParsingValue,
    ExpectedError,
    UnknownVariantError,
};

/// Represents a JSON value
#[deriving(Clone, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Floating(f64),
    String(string::String),
    List(Vec<Value>),
    Object(TreeMap<string::String, Value>),
}

impl Value {
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
    pub fn find<'a>(&'a self, key: &string::String) -> Option<&'a Value>{
        match self {
            &Object(ref map) => map.get(key),
            _ => None
        }
    }

    /// Attempts to get a nested Json Object for each key in `keys`.
    /// If any key is found not to exist, find_path will return None.
    /// Otherwise, it will return the Json value associated with the final key.
    pub fn find_path<'a>(&'a self, keys: &[&string::String]) -> Option<&'a Value>{
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
    pub fn search<'a>(&'a self, key: &string::String) -> Option<&'a Value> {
        match self {
            &Object(ref map) => {
                match map.get(key) {
                    Some(json_value) => Some(json_value),
                    None => {
                        let mut value : Option<&'a Value> = None;
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
    pub fn as_object<'a>(&'a self) -> Option<&'a TreeMap<string::String, Value>> {
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
    pub fn as_list<'a>(&'a self) -> Option<&'a Vec<Value>> {
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

impl fmt::Show for Value {
    /// Serializes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_writer(WriterFormatter(f)).map_err(|_| fmt::WriteError)
    }
}

impl<S: ser::Serializer<E>, E> ser::Serialize<S, E> for Value {
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

impl<D: de::Deserializer<E>, E> de::Deserialize<D, E> for Value {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<Value, E> {
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
                let fields: Vec<Value> = try!(de::Deserialize::deserialize_token(d, token));
                let mut object = TreeMap::new();
                object.insert(name.to_string(), List(fields));
                Ok(Object(object))
            }
            de::End => Err(d.syntax_error(de::End, [de::EndKind])),
        }
    }
}

enum JsonDeserializerState {
    JsonDeserializerValueState(Value),
    JsonDeserializerListState(vec::MoveItems<Value>),
    JsonDeserializerObjectState(tree_map::MoveEntries<string::String, Value>),
    JsonDeserializerEndState,
}

pub struct JsonDeserializer {
    stack: Vec<JsonDeserializerState>,
}

impl JsonDeserializer {
    /// Creates a new deserializer instance for deserializing the specified JSON value.
    pub fn new(json: Value) -> JsonDeserializer {
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

/// Decodes a json value from a `Value`.
pub fn from_json<
    T: de::Deserialize<JsonDeserializer, ParserError>
>(json: Value) -> Result<T, ParserError> {
    let mut d = JsonDeserializer::new(json);
    de::Deserialize::deserialize(&mut d)
}

/// A trait for converting values to JSON
pub trait ToJson {
    /// Converts the value of `self` to an instance of JSON
    fn to_json(&self) -> Value;
}

impl ToJson for Value {
    fn to_json(&self) -> Value { (*self).clone() }
}

impl ToJson for int {
    fn to_json(&self) -> Value { Integer(*self as i64) }
}

impl ToJson for i8 {
    fn to_json(&self) -> Value { Integer(*self as i64) }
}

impl ToJson for i16 {
    fn to_json(&self) -> Value { Integer(*self as i64) }
}

impl ToJson for i32 {
    fn to_json(&self) -> Value { Integer(*self as i64) }
}

impl ToJson for i64 {
    fn to_json(&self) -> Value { Integer(*self as i64) }
}

impl ToJson for uint {
    fn to_json(&self) -> Value { Integer(*self as i64) }
}

impl ToJson for u8 {
    fn to_json(&self) -> Value { Integer(*self as i64) }
}

impl ToJson for u16 {
    fn to_json(&self) -> Value { Integer(*self as i64) }
}

impl ToJson for u32 {
    fn to_json(&self) -> Value { Integer(*self as i64) }
}

impl ToJson for u64 {
    fn to_json(&self) -> Value { Integer(*self as i64) }
}

impl ToJson for f32 {
    fn to_json(&self) -> Value { Floating(*self as f64) }
}

impl ToJson for f64 {
    fn to_json(&self) -> Value { Floating(*self) }
}

impl ToJson for bool {
    fn to_json(&self) -> Value { Boolean(*self) }
}

impl<'a> ToJson for &'a str {
    fn to_json(&self) -> Value { String(self.to_string()) }
}

impl ToJson for string::String {
    fn to_json(&self) -> Value { String((*self).clone()) }
}

macro_rules! peel_to_json_tuple {
    ($name:ident, $($other:ident,)*) => (impl_to_json_tuple!($($other,)*))
}

macro_rules! impl_to_json_tuple {
    () => {
        impl<> ToJson for () {
            #[inline]
            fn to_json(&self) -> Value {
                Null
            }
        }
    };
    ( $($name:ident,)+ ) => {
        impl<$($name: ToJson),*> ToJson for ($($name,)*) {
            #[inline]
            #[allow(non_snake_case)]
            fn to_json(&self) -> Value {
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
    fn to_json(&self) -> Value { List(self.iter().map(|elt| elt.to_json()).collect()) }
}

impl<A:ToJson> ToJson for TreeMap<string::String, A> {
    fn to_json(&self) -> Value {
        let mut d = TreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_json());
        }
        Object(d)
    }
}

impl<A:ToJson> ToJson for HashMap<string::String, A> {
    fn to_json(&self) -> Value {
        let mut d = TreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_json());
        }
        Object(d)
    }
}

impl<A:ToJson> ToJson for Option<A> {
    fn to_json(&self) -> Value {
        match *self {
          None => Null,
          Some(ref value) => value.to_json()
        }
    }
}
