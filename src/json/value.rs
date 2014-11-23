use std::collections::{HashMap, TreeMap, tree_map};
use std::fmt;
use std::io::{ByRefWriter, IoResult, MemWriter};
use std::io;
use std::str;
use std::string;
use std::vec;

use de::{mod, Token, TokenKind};
use ser::Serialize;
use ser;

use super::ser::{Serializer, PrettySerializer};
use super::error::{Error, ErrorCode};

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
    pub fn to_writer<W: Writer>(&self, wr: W) -> IoResult<()> {
        let mut serializer = Serializer::new(wr);
        self.serialize(&mut serializer)
    }

    /// Serializes a json value into an io::writer.
    /// Pretty-prints in a more readable format.
    pub fn to_pretty_writer<W: Writer>(&self, wr: W) -> IoResult<()> {
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
            &Value::Object(ref map) => map.get(key),
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
            &Value::Object(ref map) => {
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
            Value::Object(ref map) => Some(map),
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
            Value::List(ref list) => Some(list),
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
            Value::String(ref s) => Some(s.as_slice()),
            _ => None
        }
    }

    /// Returns true if the Json value is a i64 or f64. Returns false otherwise.
    pub fn is_number(&self) -> bool {
        match *self {
            Value::Integer(_) | Value::Floating(_) => true,
            _ => false,
        }
    }

    /// Returns true if the Json value is a i64. Returns false otherwise.
    pub fn is_i64(&self) -> bool {
        match *self {
            Value::Integer(_) => true,
            _ => false,
        }
    }

    /// Returns true if the Json value is a f64. Returns false otherwise.
    pub fn is_f64(&self) -> bool {
        match *self {
            Value::Floating(_) => true,
            _ => false,
        }
    }

    /// If the Json value is a i64, returns the associated i64.
    /// Returns None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Integer(n) => Some(n),
            Value::Floating(n) => Some(n as i64),
            _ => None
        }
    }

    /// If the Json value is a f64, returns the associated f64.
    /// Returns None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Integer(n) => Some(n as f64),
            Value::Floating(n) => Some(n),
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
            Value::Boolean(b) => Some(b),
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
            Value::Null => Some(()),
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
        self.to_writer(WriterFormatter(f)).map_err(|_| fmt::Error)
    }
}

impl<S: ser::Serializer<E>, E> ser::Serialize<S, E> for Value {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        match *self {
            Value::Null => {
                ().serialize(s)
            }
            Value::Boolean(v) => {
                v.serialize(s)
            }
            Value::Integer(v) => {
                v.serialize(s)
            }
            Value::Floating(v) => {
                v.serialize(s)
            }
            Value::String(ref v) => {
                v.serialize(s)
            }
            Value::List(ref v) => {
                v.serialize(s)
            }
            Value::Object(ref v) => {
                v.serialize(s)
            }
        }
    }
}

impl<D: de::Deserializer<E>, E> de::Deserialize<D, E> for Value {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Value, E> {
        match token {
            Token::Null => Ok(Value::Null),
            Token::Bool(x) => Ok(Value::Boolean(x)),
            Token::Int(x) => Ok(Value::Integer(x as i64)),
            Token::I8(x) => Ok(Value::Integer(x as i64)),
            Token::I16(x) => Ok(Value::Integer(x as i64)),
            Token::I32(x) => Ok(Value::Integer(x as i64)),
            Token::I64(x) => Ok(Value::Integer(x)),
            Token::Uint(x) => Ok(Value::Integer(x as i64)),
            Token::U8(x) => Ok(Value::Integer(x as i64)),
            Token::U16(x) => Ok(Value::Integer(x as i64)),
            Token::U32(x) => Ok(Value::Integer(x as i64)),
            Token::U64(x) => Ok(Value::Integer(x as i64)),
            Token::F32(x) => Ok(Value::Floating(x as f64)),
            Token::F64(x) => Ok(Value::Floating(x)),
            Token::Char(x) => Ok(Value::String(x.to_string())),
            Token::Str(x) => Ok(Value::String(x.to_string())),
            Token::String(x) => Ok(Value::String(x)),
            Token::Option(false) => Ok(Value::Null),
            Token::Option(true) => de::Deserialize::deserialize(d),
            Token::TupleStart(_) | Token::SeqStart(_) => {
                let list = try!(de::Deserialize::deserialize_token(d, token));
                Ok(Value::List(list))
            }
            Token::StructStart(_, _) | Token::MapStart(_) => {
                let object = try!(de::Deserialize::deserialize_token(d, token));
                Ok(Value::Object(object))
            }
            Token::EnumStart(_, name, len) => {
                let token = Token::SeqStart(len);
                let fields: Vec<Value> = try!(de::Deserialize::deserialize_token(d, token));
                let mut object = TreeMap::new();
                object.insert(name.to_string(), Value::List(fields));
                Ok(Value::Object(object))
            }
            Token::End => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::EndKind,
                ];
                Err(d.syntax_error(Token::End, EXPECTED_TOKENS))
            }
        }
    }
}

enum State {
    Value(Value),
    List(vec::MoveItems<Value>),
    Object(tree_map::MoveEntries<string::String, Value>),
    End,
}

pub struct Deserializer {
    stack: Vec<State>,
}

impl Deserializer {
    /// Creates a new deserializer instance for deserializing the specified JSON value.
    pub fn new(json: Value) -> Deserializer {
        Deserializer {
            stack: vec!(State::Value(json)),
        }
    }
}

impl Iterator<Result<Token, Error>> for Deserializer {
    #[inline]
    fn next(&mut self) -> Option<Result<Token, Error>> {
        loop {
            match self.stack.pop() {
                Some(State::Value(value)) => {
                    let token = match value {
                        Value::Null => Token::Null,
                        Value::Boolean(x) => Token::Bool(x),
                        Value::Integer(x) => Token::I64(x),
                        Value::Floating(x) => Token::F64(x),
                        Value::String(x) => Token::String(x),
                        Value::List(x) => {
                            let len = x.len();
                            self.stack.push(State::List(x.into_iter()));
                            Token::SeqStart(len)
                        }
                        Value::Object(x) => {
                            let len = x.len();
                            self.stack.push(State::Object(x.into_iter()));
                            Token::MapStart(len)
                        }
                    };

                    return Some(Ok(token));
                }
                Some(State::List(mut iter)) => {
                    match iter.next() {
                        Some(value) => {
                            self.stack.push(State::List(iter));
                            self.stack.push(State::Value(value));
                            // loop around.
                        }
                        None => {
                            return Some(Ok(Token::End));
                        }
                    }
                }
                Some(State::Object(mut iter)) => {
                    match iter.next() {
                        Some((key, value)) => {
                            self.stack.push(State::Object(iter));
                            self.stack.push(State::Value(value));
                            return Some(Ok(Token::String(key)));
                        }
                        None => {
                            return Some(Ok(Token::End));
                        }
                    }
                }
                Some(State::End) => {
                    return Some(Ok(Token::End));
                }
                None => { return None; }
            }
        }
    }
}

impl de::Deserializer<Error> for Deserializer {
    fn end_of_stream_error(&mut self) -> Error {
        Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 0, 0)
    }

    fn syntax_error(&mut self,
                    token: Token,
                    expected: &'static [TokenKind]) -> Error {
        Error::SyntaxError(ErrorCode::ExpectedTokens(token, expected), 0, 0)
    }

    fn unexpected_name_error(&mut self, token: Token) -> Error {
        Error::SyntaxError(ErrorCode::UnexpectedName(token), 0, 0)
    }

    fn conversion_error(&mut self, token: Token) -> Error {
        Error::SyntaxError(ErrorCode::ConversionError(token), 0, 0)
    }

    #[inline]
    fn missing_field<
        T: de::Deserialize<Deserializer, Error>
    >(&mut self, _field: &'static str) -> Result<T, Error> {
        // JSON can represent `null` values as a missing value, so this isn't
        // necessarily an error.
        de::Deserialize::deserialize_token(self, Token::Null)
    }

    // Special case treating options as a nullable value.
    #[inline]
    fn expect_option<
        U: de::Deserialize<Deserializer, Error>
    >(&mut self, token: Token) -> Result<Option<U>, Error> {
        match token {
            Token::Null => Ok(None),
            token => {
                let value: U = try!(de::Deserialize::deserialize_token(self, token));
                Ok(Some(value))
            }
        }
    }

    // Special case treating enums as a String or a `{"variant": "...", "fields": [...]}`.
    #[inline]
    fn expect_enum_start(&mut self,
                         token: Token,
                         _name: &str,
                         variants: &[&str]) -> Result<uint, Error> {
        let variant = match token {
            Token::MapStart(_) => {
                let state = match self.stack.pop() {
                    Some(state) => state,
                    None => { panic!("state machine error, state stack empty"); }
                };

                let mut iter = match state {
                    State::Object(iter) => iter,
                    _ => { panic!("state machine error, expected an object"); }
                };

                let (variant, fields) = match iter.next() {
                    Some((variant, Value::List(fields))) => (variant, fields),
                    Some((key, value)) => {
                        return Err(
                            Error::ExpectedError(
                                "List".to_string(),
                                format!("{} => {}", key, value)
                            )
                        );
                    }
                    None => {
                        return Err(Error::MissingFieldError("<variant-name>".to_string()));
                    }
                };

                // Error out if there are other fields in the enum.
                match iter.next() {
                    Some((key, value)) => {
                        return Err(
                            Error::ExpectedError(
                                "None".to_string(),
                                format!("{} => {}", key, value)
                            )
                        );
                    }
                    None => { }
                }

                self.stack.push(State::End);

                for field in fields.into_iter().rev() {
                    self.stack.push(State::Value(field));
                }

                variant
            }
            token => {
                return Err(
                    Error::ExpectedError(
                        "String or Object".to_string(),
                        format!("{}", token)
                    )
                );
            }
        };

        match variants.iter().position(|v| *v == variant.as_slice()) {
            Some(idx) => Ok(idx),
            None => Err(Error::UnknownVariantError(variant)),
        }
    }

    #[inline]
    fn expect_struct_start(&mut self, token: Token, _name: &str) -> Result<(), Error> {
        match token {
            Token::MapStart(_) => Ok(()),
            _ => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::MapStartKind
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }
}

/// Decodes a json value from a `Value`.
pub fn from_json<
    T: de::Deserialize<Deserializer, Error>
>(json: Value) -> Result<T, Error> {
    let mut d = Deserializer::new(json);
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
    fn to_json(&self) -> Value { Value::Integer(*self as i64) }
}

impl ToJson for i8 {
    fn to_json(&self) -> Value { Value::Integer(*self as i64) }
}

impl ToJson for i16 {
    fn to_json(&self) -> Value { Value::Integer(*self as i64) }
}

impl ToJson for i32 {
    fn to_json(&self) -> Value { Value::Integer(*self as i64) }
}

impl ToJson for i64 {
    fn to_json(&self) -> Value { Value::Integer(*self as i64) }
}

impl ToJson for uint {
    fn to_json(&self) -> Value { Value::Integer(*self as i64) }
}

impl ToJson for u8 {
    fn to_json(&self) -> Value { Value::Integer(*self as i64) }
}

impl ToJson for u16 {
    fn to_json(&self) -> Value { Value::Integer(*self as i64) }
}

impl ToJson for u32 {
    fn to_json(&self) -> Value { Value::Integer(*self as i64) }
}

impl ToJson for u64 {
    fn to_json(&self) -> Value { Value::Integer(*self as i64) }
}

impl ToJson for f32 {
    fn to_json(&self) -> Value { Value::Floating(*self as f64) }
}

impl ToJson for f64 {
    fn to_json(&self) -> Value { Value::Floating(*self) }
}

impl ToJson for bool {
    fn to_json(&self) -> Value { Value::Boolean(*self) }
}

impl<'a> ToJson for &'a str {
    fn to_json(&self) -> Value { Value::String(self.to_string()) }
}

impl ToJson for string::String {
    fn to_json(&self) -> Value { Value::String((*self).clone()) }
}

macro_rules! peel_to_json_tuple {
    ($name:ident, $($other:ident,)*) => (impl_to_json_tuple!($($other,)*))
}

macro_rules! impl_to_json_tuple {
    () => {
        impl<> ToJson for () {
            #[inline]
            fn to_json(&self) -> Value {
                Value::Null
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

                Value::List(list)
            }
        }
        peel_to_json_tuple!($($name,)*)
    }
}

impl_to_json_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

impl<A:ToJson> ToJson for Vec<A> {
    fn to_json(&self) -> Value {
        Value::List(self.iter().map(|elt| elt.to_json()).collect())
    }
}

impl<A:ToJson> ToJson for TreeMap<string::String, A> {
    fn to_json(&self) -> Value {
        let mut d = TreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_json());
        }
        Value::Object(d)
    }
}

impl<A:ToJson> ToJson for HashMap<string::String, A> {
    fn to_json(&self) -> Value {
        let mut d = TreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_json());
        }
        Value::Object(d)
    }
}

impl<A:ToJson> ToJson for Option<A> {
    fn to_json(&self) -> Value {
        match *self {
          None => Value::Null,
          Some(ref value) => value.to_json()
        }
    }
}

impl<'a, T: ToJson> ToJson for &'a T {
    fn to_json(&self) -> Value {
        (*self).to_json()
    }
}
