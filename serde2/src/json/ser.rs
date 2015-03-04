use std::{f32, f64};
use std::io;
use std::num::{Float, FpCategory};
use std::string::FromUtf8Error;

use ser;

/// A structure for implementing serialization to JSON.
pub struct Serializer<W> {
    writer: W,
}

impl<W: io::Write> Serializer<W> {
    /// Creates a new JSON visitr whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn new(writer: W) -> Serializer<W> {
        Serializer {
            writer: writer,
        }
    }

    /// Unwrap the `Writer` from the `Serializer`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W: io::Write> ser::Serializer for Serializer<W> {
    type Value = ();
    type Error = io::Error;

    #[inline]
    fn visit<T>(&mut self, value: &T) -> io::Result<()>
        where T: ser::Serialize,
    {
        value.visit(&mut Visitor { writer: &mut self.writer })
    }
}

struct Visitor<'a, W: 'a> {
    writer: &'a mut W,
}

impl<'a, W: io::Write> ser::Visitor for Visitor<'a, W> {
    type Value = ();
    type Error = io::Error;

    #[inline]
    fn visit_bool(&mut self, value: bool) -> io::Result<()> {
        if value {
            try!(self.writer.write(b"true"));
        } else {
            try!(self.writer.write(b"false"));
        }
        Ok(())
    }

    #[inline]
    fn visit_isize(&mut self, value: isize) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i8(&mut self, value: i8) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i16(&mut self, value: i16) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i32(&mut self, value: i32) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i64(&mut self, value: i64) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_usize(&mut self, value: usize) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u8(&mut self, value: u8) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u16(&mut self, value: u16) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u32(&mut self, value: u32) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u64(&mut self, value: u64) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_f32(&mut self, value: f32) -> io::Result<()> {
        fmt_f32_or_null(self.writer, value)
    }

    #[inline]
    fn visit_f64(&mut self, value: f64) -> io::Result<()> {
        fmt_f64_or_null(self.writer, value)
    }

    #[inline]
    fn visit_char(&mut self, value: char) -> io::Result<()> {
        escape_char(self.writer, value)
    }

    #[inline]
    fn visit_str(&mut self, value: &str) -> io::Result<()> {
        escape_str(self.writer, value)
    }

    #[inline]
    fn visit_none(&mut self) -> io::Result<()> {
        self.visit_unit()
    }

    #[inline]
    fn visit_some<V>(&mut self, value: V) -> io::Result<()>
        where V: ser::Serialize
    {
        value.visit(self)
    }

    #[inline]
    fn visit_unit(&mut self) -> io::Result<()> {
        try!(self.writer.write(b"null"));
        Ok(())
    }

    #[inline]
    fn visit_enum_unit(&mut self, _name: &str, variant: &str) -> io::Result<()> {
        try!(self.writer.write(b"{"));
        try!(self.visit_str(variant));
        try!(self.writer.write(b":[]}"));
        Ok(())
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> io::Result<()>
        where V: ser::SeqVisitor,
    {
        try!(self.writer.write(b"["));

        while let Some(()) = try!(visitor.visit(self)) { }

        try!(self.writer.write(b"]"));
        Ok(())
    }

    #[inline]
    fn visit_enum_seq<V>(&mut self, _name: &str, variant: &str, visitor: V) -> io::Result<()>
        where V: ser::SeqVisitor,
    {
        try!(self.writer.write(b"{"));
        try!(self.visit_str(variant));
        try!(self.writer.write(b":"));
        try!(self.visit_seq(visitor));
        try!(self.writer.write(b"}"));

        Ok(())
    }

    #[inline]
    fn visit_seq_elt<T>(&mut self, first: bool, value: T) -> io::Result<()>
        where T: ser::Serialize,
    {
        if !first {
            try!(self.writer.write(b","));
        }

        value.visit(self)
    }

    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> io::Result<()>
        where V: ser::MapVisitor,
    {
        try!(self.writer.write(b"{"));

        while let Some(()) = try!(visitor.visit(self)) { }

        try!(self.writer.write(b"}"));

        Ok(())
    }

    #[inline]
    fn visit_enum_map<V>(&mut self, _name: &str, variant: &str, visitor: V) -> io::Result<()>
        where V: ser::MapVisitor,
    {
        try!(self.writer.write(b"{"));
        try!(self.visit_str(variant));
        try!(self.writer.write(b":"));
        try!(self.visit_map(visitor));
        try!(self.writer.write(b"}"));

        Ok(())
    }

    #[inline]
    fn visit_map_elt<K, V>(&mut self, first: bool, key: K, value: V) -> io::Result<()>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        if !first {
            try!(self.writer.write(b","));
        }

        try!(key.visit(self));
        try!(self.writer.write(b":"));
        try!(value.visit(self));
        Ok(())
    }
}

#[inline]
pub fn escape_bytes<W>(wr: &mut W, bytes: &[u8]) -> io::Result<()>
    where W: io::Write
{
    try!(wr.write(b"\""));

    let mut start = 0;

    for (i, byte) in bytes.iter().enumerate() {
        let escaped = match *byte {
            b'"' => b"\\\"",
            b'\\' => b"\\\\",
            b'\x08' => b"\\b",
            b'\x0c' => b"\\f",
            b'\n' => b"\\n",
            b'\r' => b"\\r",
            b'\t' => b"\\t",
            _ => { continue; }
        };

        if start < i {
            try!(wr.write(&bytes[start..i]));
        }

        try!(wr.write(escaped));

        start = i + 1;
    }

    if start != bytes.len() {
        try!(wr.write(&bytes[start..]));
    }

    try!(wr.write(b"\""));
    Ok(())
}

#[inline]
pub fn escape_str<W>(wr: &mut W, value: &str) -> io::Result<()>
    where W: io::Write
{
    escape_bytes(wr, value.as_bytes())
}

#[inline]
fn escape_char<W>(wr: &mut W, value: char) -> io::Result<()>
    where W: io::Write
{
    let buf = &mut [0; 4];
    value.encode_utf8(buf);
    escape_bytes(wr, buf)
}

fn fmt_f32_or_null<W>(wr: &mut W, value: f32) -> io::Result<()>
    where W: io::Write
{
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => try!(wr.write(b"null")),
        _ => try!(wr.write(f32::to_str_digits(value, 6).as_bytes())),
    };

    Ok(())
}

fn fmt_f64_or_null<W>(wr: &mut W, value: f64) -> io::Result<()>
    where W: io::Write
{
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => try!(wr.write(b"null")),
        _ => try!(wr.write(f64::to_str_digits(value, 6).as_bytes())),
    };

    Ok(())
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_writer<W, T>(writer: &mut W, value: &T) -> io::Result<()>
    where W: io::Write,
          T: ser::Serialize,
{
    let mut ser = Serializer::new(writer);
    try!(ser::Serializer::visit(&mut ser, value));
    Ok(())
}

/// Encode the specified struct into a json `[u8]` buffer.
#[inline]
pub fn to_vec<T>(value: &T) -> Vec<u8>
    where T: ser::Serialize,
{
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let mut writer = Vec::with_capacity(128);
    to_writer(&mut writer, value).unwrap();
    writer
}

/// Encode the specified struct into a json `String` buffer.
#[inline]
pub fn to_string<T>(value: &T) -> Result<String, FromUtf8Error>
    where T: ser::Serialize
{
    let vec = to_vec(value);
    String::from_utf8(vec)
}
