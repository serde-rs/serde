use std::collections::TreeMap;
use std::fmt;
use std::io;

use ser::{mod, Serializer};

#[deriving(PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    Array(Vec<Value>),
    Object(TreeMap<String, Value>),
}

impl ser::Serialize for Value {
    #[inline]
    fn visit<
        S,
        R,
        E,
        V: ser::Visitor<S, R, E>,
    >(&self, s: &mut S, visitor: V) -> Result<R, E> {
        match *self {
            Value::Null => {
                visitor.visit_null(s)
            }
            Value::Bool(v) => {
                visitor.visit_bool(s, v)
            }
            Value::I64(v) => {
                visitor.visit_i64(s, v)
            }
            Value::F64(v) => {
                visitor.visit_f64(s, v)
            }
            Value::String(ref v) => {
                visitor.visit_str(s, v.as_slice())
            }
            Value::Array(ref v) => {
                v.visit(s, visitor)
            }
            Value::Object(ref v) => {
                v.visit(s, visitor)
            }
        }
    }
}

struct WriterFormatter<'a, 'b: 'a>(&'a mut fmt::Formatter<'b>);

impl<'a, 'b> io::Writer for WriterFormatter<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> io::IoResult<()> {
        let WriterFormatter(ref mut f) = *self;
        f.write(buf).map_err(|_| io::IoError::last_error())
    }
}

impl fmt::Show for Value {
    /// Serializes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut wr = WriterFormatter(f);
        super::ser::to_writer(&mut wr, self).map_err(|_| fmt::Error)
    }
}

pub fn to_value<
    T: ser::Serialize,
>(value: &T) -> Value {
    let mut writer = Writer::new();
    writer.visit(value).unwrap();
    writer.unwrap()
}

enum State {
    Value(Value),
    Array(Vec<Value>),
    Object(TreeMap<String, Value>),
}

pub struct Writer {
    state: Vec<State>,
}

impl Writer {
    pub fn new() -> Writer {
        Writer {
            state: Vec::with_capacity(4),
        }
    }

    pub fn unwrap(mut self) -> Value {
        match self.state.pop().unwrap() {
            State::Value(value) => value,
            _ => panic!(),
        }
    }
}

impl ser::Serializer<Writer, (), ()> for Writer {
    #[inline]
    fn visit<
        T: ser::Serialize,
    >(&mut self, value: &T) -> Result<(), ()> {
        value.visit(self, Visitor)
    }
}

struct Visitor;

impl ser::Visitor<Writer, (), ()> for Visitor {
    #[inline]
    fn visit_null(&self, state: &mut Writer) -> Result<(), ()> {
        state.state.push(State::Value(Value::Null));
        Ok(())
    }

    #[inline]
    fn visit_bool(&self, state: &mut Writer, value: bool) -> Result<(), ()> {
        state.state.push(State::Value(Value::Bool(value)));
        Ok(())
    }

    #[inline]
    fn visit_i64(&self, state: &mut Writer, value: i64) -> Result<(), ()> {
        state.state.push(State::Value(Value::I64(value)));
        Ok(())
    }

    #[inline]
    fn visit_u64(&self, state: &mut Writer, value: u64) -> Result<(), ()> {
        state.state.push(State::Value(Value::I64(value as i64)));
        Ok(())
    }

    #[inline]
    fn visit_f64(&self, state: &mut Writer, value: f64) -> Result<(), ()> {
        state.state.push(State::Value(Value::F64(value as f64)));
        Ok(())
    }

    #[inline]
    fn visit_char(&self, state: &mut Writer, value: char) -> Result<(), ()> {
        state.state.push(State::Value(Value::String(value.to_string())));
        Ok(())
    }

    #[inline]
    fn visit_str(&self, state: &mut Writer, value: &str) -> Result<(), ()> {
        state.state.push(State::Value(Value::String(value.to_string())));
        Ok(())
    }

    #[inline]
    fn visit_seq<
        V: ser::SeqVisitor<Writer, (), ()>
    >(&self, state: &mut Writer, mut visitor: V) -> Result<(), ()> {
        let len = match visitor.size_hint() {
            (_, Some(len)) => len,
            (len, None) => len,
        };

        let values = Vec::with_capacity(len);

        state.state.push(State::Array(values));

        loop {
            match try!(visitor.visit(state, Visitor)) {
                Some(()) => { }
                None => { break; }
            }
        }

        match state.state.pop().unwrap() {
            State::Array(values) => { state.state.push(State::Value(Value::Array(values))); }
            _ => panic!(),
        }

        Ok(())
    }

    #[inline]
    fn visit_seq_elt<
        T: ser::Serialize,
    >(&self, state: &mut Writer, _first: bool, value: T) -> Result<(), ()> {
        try!(value.visit(state, Visitor));
        let value = match state.state.pop().unwrap() {
            State::Value(value) => value,
            _ => panic!(),
        };

        match *state.state.last_mut().unwrap() {
            State::Array(ref mut values) => { values.push(value); }
            _ => panic!(),
        }

        Ok(())
    }

    #[inline]
    fn visit_map<
        V: ser::MapVisitor<Writer, (), ()>
    >(&self, state: &mut Writer, mut visitor: V) -> Result<(), ()> {
        let values = TreeMap::new();

        state.state.push(State::Object(values));

        loop {
            match try!(visitor.visit(state, Visitor)) {
                Some(()) => { }
                None => { break; }
            }
        }

        match state.state.pop().unwrap() {
            State::Object(values) => { state.state.push(State::Value(Value::Object(values))); }
            _ => panic!(),
        }

        Ok(())
    }

    #[inline]
    fn visit_map_elt<
        K: ser::Serialize,
        V: ser::Serialize,
    >(&self, state: &mut Writer, _first: bool, key: K, value: V) -> Result<(), ()> {
        try!(key.visit(state, Visitor));
        try!(value.visit(state, Visitor));

        let key = match state.state.pop().unwrap() {
            State::Value(Value::String(value)) => value,
            _ => panic!(),
        };

        let value = match state.state.pop().unwrap() {
            State::Value(value) => value,
            _ => panic!(),
        };

        match *state.state.last_mut().unwrap() {
            State::Object(ref mut values) => { values.insert(key, value); }
            _ => panic!(),
        }

        Ok(())
    }
}
