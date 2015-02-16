use std::f64;
use std::old_io::{self, ByRefWriter, IoError};
use std::num::{Float, FpCategory};
use std::string::FromUtf8Error;

use ser;
use ser::Serializer;

/// A structure for implementing serialization to JSON.
pub struct Writer<W> {
    writer: W,
}

impl<W: old_io::Writer> Writer<W> {
    /// Creates a new JSON visitr whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn new(writer: W) -> Writer<W> {
        Writer {
            writer: writer,
        }
    }

    /// Unwrap the Writer from the Serializer.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W: old_io::Writer> ser::Serializer for Writer<W> {
    type Value = ();
    type Error = IoError;

    #[inline]
    fn visit<
        T: ser::Serialize,
    >(&mut self, value: &T) -> Result<(), IoError> {
        value.visit(&mut Visitor { writer: &mut self.writer })
    }
}

struct Visitor<'a, W: 'a> {
    writer: &'a mut W,
}

impl<'a, W: old_io::Writer> ser::Visitor for Visitor<'a, W> {
    type Value = ();
    type Error = IoError;

    #[inline]
    fn visit_unit(&mut self) -> Result<(), IoError> {
        self.writer.write_str("null")
    }

    #[inline]
    fn visit_bool(&mut self, value: bool) -> Result<(), IoError> {
        if value {
            self.writer.write_str("true")
        } else {
            self.writer.write_str("false")
        }
    }

    #[inline]
    fn visit_isize(&mut self, value: isize) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i8(&mut self, value: i8) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i16(&mut self, value: i16) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i32(&mut self, value: i32) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i64(&mut self, value: i64) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_usize(&mut self, value: usize) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u8(&mut self, value: u8) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u16(&mut self, value: u16) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u32(&mut self, value: u32) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u64(&mut self, value: u64) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_f64(&mut self, value: f64) -> Result<(), IoError> {
        fmt_f64_or_null(self.writer, value)
    }

    #[inline]
    fn visit_char(&mut self, v: char) -> Result<(), IoError> {
        escape_char(self.writer, v)
    }

    #[inline]
    fn visit_str(&mut self, value: &str) -> Result<(), IoError> {
        escape_str(self.writer, value)
    }

    #[inline]
    fn visit_none(&mut self) -> Result<(), IoError> {
        self.visit_unit()
    }

    #[inline]
    fn visit_some<V>(&mut self, value: V) -> Result<(), IoError>
        where V: ser::Serialize
    {
        value.visit(self)
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<(), IoError>
        where V: ser::SeqVisitor,
    {
        try!(self.writer.write_str("["));

        while let Some(()) = try!(visitor.visit(self)) { }

        self.writer.write_str("]")
    }

    #[inline]
    fn visit_seq_elt<T>(&mut self, first: bool, value: T) -> Result<(), IoError>
        where T: ser::Serialize,
    {
        if !first {
            try!(self.writer.write_str(","));
        }

        value.visit(self)
    }

    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> Result<(), IoError>
        where V: ser::MapVisitor,
    {
        try!(self.writer.write_str("{"));

        while let Some(()) = try!(visitor.visit(self)) { }

        self.writer.write_str("}")
    }

    #[inline]
    fn visit_map_elt<K, V>(&mut self, first: bool, key: K, value: V) -> Result<(), IoError>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        if !first {
            try!(self.writer.write_str(","));
        }

        try!(key.visit(self));
        try!(self.writer.write_str(":"));
        value.visit(self)
    }
}

#[inline]
pub fn escape_bytes<W: old_io::Writer>(wr: &mut W, bytes: &[u8]) -> Result<(), IoError> {
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
            try!(wr.write_all(&bytes[start..i]));
        }

        try!(wr.write_str(escaped));

        start = i + 1;
    }

    if start != bytes.len() {
        try!(wr.write_all(&bytes[start..]));
    }

    wr.write_str("\"")
}

#[inline]
pub fn escape_str<W: old_io::Writer>(wr: &mut W, value: &str) -> Result<(), IoError> {
    escape_bytes(wr, value.as_bytes())
}

#[inline]
pub fn escape_char<W: old_io::Writer>(wr: &mut W, value: char) -> Result<(), IoError> {
    let mut buf = &mut [0; 4];
    value.encode_utf8(buf);
    escape_bytes(wr, buf)
}

fn fmt_f64_or_null<W: old_io::Writer>(wr: &mut W, value: f64) -> Result<(), IoError> {
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => wr.write_str("null"),
        _ => wr.write_str(&f64::to_str_digits(value, 6)),
    }
}

#[inline]
pub fn to_writer<W, T>(wr: &mut W, value: &T) -> Result<(), IoError>
    where W: old_io::Writer,
          T: ser::Serialize,
{
    let mut wr = Writer::new(wr.by_ref());
    try!(wr.visit(value));
    Ok(())
}

#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, IoError>
    where T: ser::Serialize,
{
    let mut wr = Vec::with_capacity(128);
    to_writer(&mut wr, value).unwrap();
    Ok(wr)
}

#[inline]
pub fn to_string<T>(value: &T) -> Result<Result<String, FromUtf8Error>, IoError>
    where T: ser::Serialize,
{
    let vec = try!(to_vec(value));
    Ok(String::from_utf8(vec))
}
