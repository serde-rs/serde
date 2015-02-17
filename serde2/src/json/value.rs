use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::str;

use ser::{self, Serializer};

#[derive(PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl ser::Serialize for Value {
    #[inline]
    fn visit<
        V: ser::Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        match *self {
            Value::Null => {
                visitor.visit_unit()
            }
            Value::Bool(v) => {
                visitor.visit_bool(v)
            }
            Value::I64(v) => {
                visitor.visit_i64(v)
            }
            Value::F64(v) => {
                visitor.visit_f64(v)
            }
            Value::String(ref v) => {
                visitor.visit_str(&v)
            }
            Value::Array(ref v) => {
                v.visit(visitor)
            }
            Value::Object(ref v) => {
                v.visit(visitor)
            }
        }
    }
}

struct WriterFormatter<'a, 'b: 'a> {
    inner: &'a mut fmt::Formatter<'b>,
}

impl<'a, 'b> io::Write for WriterFormatter<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.write_str(str::from_utf8(buf).unwrap()) {
            Ok(_) => Ok(buf.len()),
            Err(_) => Err(io::Error::last_os_error()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl fmt::Debug for Value {
    /// Serializes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut wr = WriterFormatter { inner: f };
        super::ser::to_writer(&mut wr, self).map_err(|_| fmt::Error)
    }
}

pub fn to_value<T>(value: &T) -> Value where T: ser::Serialize {
    let mut writer = Writer::new();
    writer.visit(value).ok().unwrap();
    writer.unwrap()
}

enum State {
    Value(Value),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
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

impl ser::Serializer for Writer {
    type Value = ();
    type Error = ();

    #[inline]
    fn visit<
        T: ser::Serialize,
    >(&mut self, value: &T) -> Result<(), ()> {
        try!(value.visit(self));
        Ok(())
    }
}

impl ser::Visitor for Writer {
    type Value = ();
    type Error = ();

    #[inline]
    fn visit_unit(&mut self) -> Result<(), ()> {
        self.state.push(State::Value(Value::Null));
        Ok(())
    }

    #[inline]
    fn visit_bool(&mut self, value: bool) -> Result<(), ()> {
        self.state.push(State::Value(Value::Bool(value)));
        Ok(())
    }

    #[inline]
    fn visit_i64(&mut self, value: i64) -> Result<(), ()> {
        self.state.push(State::Value(Value::I64(value)));
        Ok(())
    }

    #[inline]
    fn visit_u64(&mut self, value: u64) -> Result<(), ()> {
        self.state.push(State::Value(Value::I64(value as i64)));
        Ok(())
    }

    #[inline]
    fn visit_f64(&mut self, value: f64) -> Result<(), ()> {
        self.state.push(State::Value(Value::F64(value as f64)));
        Ok(())
    }

    #[inline]
    fn visit_char(&mut self, value: char) -> Result<(), ()> {
        self.state.push(State::Value(Value::String(value.to_string())));
        Ok(())
    }

    #[inline]
    fn visit_str(&mut self, value: &str) -> Result<(), ()> {
        self.state.push(State::Value(Value::String(value.to_string())));
        Ok(())
    }

    #[inline]
    fn visit_none(&mut self) -> Result<(), ()> {
        self.visit_unit()
    }

    #[inline]
    fn visit_some<
        V: ser::Serialize,
    >(&mut self, value: V) -> Result<(), ()> {
        value.visit(self)
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<(), ()>
        where V: ser::SeqVisitor,
    {
        let len = match visitor.size_hint() {
            (_, Some(len)) => len,
            (len, None) => len,
        };

        let values = Vec::with_capacity(len);

        self.state.push(State::Array(values));

        while let Some(()) = try!(visitor.visit(self)) { }

        match self.state.pop().unwrap() {
            State::Array(values) => {
                self.state.push(State::Value(Value::Array(values)));
            }
            _ => panic!(),
        }

        Ok(())
    }

    #[inline]
    fn visit_seq_elt<T>(&mut self, _first: bool, value: T) -> Result<(), ()>
        where T: ser::Serialize,
    {
        try!(value.visit(self));

        let value = match self.state.pop().unwrap() {
            State::Value(value) => value,
            _ => panic!(),
        };

        match *self.state.last_mut().unwrap() {
            State::Array(ref mut values) => { values.push(value); }
            _ => panic!(),
        }

        Ok(())
    }

    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> Result<(), ()>
        where V: ser::MapVisitor,
    {
        let values = BTreeMap::new();

        self.state.push(State::Object(values));

        while let Some(()) = try!(visitor.visit(self)) { }

        match self.state.pop().unwrap() {
            State::Object(values) => {
                self.state.push(State::Value(Value::Object(values)));
            }
            _ => panic!(),
        }

        Ok(())
    }

    #[inline]
    fn visit_map_elt<K, V>(&mut self, _first: bool, key: K, value: V) -> Result<(), ()>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        try!(key.visit(self));
        try!(value.visit(self));

        let key = match self.state.pop().unwrap() {
            State::Value(Value::String(value)) => value,
            _ => panic!(),
        };

        let value = match self.state.pop().unwrap() {
            State::Value(value) => value,
            _ => panic!(),
        };

        match *self.state.last_mut().unwrap() {
            State::Object(ref mut values) => { values.insert(key, value); }
            _ => panic!(),
        }

        Ok(())
    }
}
