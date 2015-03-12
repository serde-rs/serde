use std::{f32, f64};
use std::io;
use std::num::{Float, FpCategory};
use std::string::FromUtf8Error;

use ser;

/// A structure for implementing serialization to JSON.
pub struct Serializer<W> {
    writer: W,
    format: Format,
    current_indent: usize,
    indent: usize,
}

#[derive(Copy, PartialEq)]
enum Format {
    Compact,
    Pretty,
}

impl<W: io::Write> Serializer<W> {
    /// Creates a new JSON visitr whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn new(writer: W) -> Serializer<W> {
        Serializer {
            writer: writer,
            format: Format::Compact,
            current_indent: 0,
            indent: 0,
        }
    }

    /// Creates a new JSON visitr whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn new_pretty(writer: W) -> Serializer<W> {
        Serializer {
            writer: writer,
            format: Format::Pretty,
            current_indent: 0,
            indent: 2,
        }
    }

    /// Unwrap the `Writer` from the `Serializer`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }

    fn serialize_sep(&mut self, first: bool) -> io::Result<()> {
        match self.format {
            Format::Compact => {
                if first {
                    Ok(())
                } else {
                    self.writer.write_all(b",")
                }
            }
            Format::Pretty => {
                if first {
                    self.current_indent += self.indent;
                    try!(self.writer.write_all(b"\n"));
                } else {
                    try!(self.writer.write_all(b",\n"));
                }

                spaces(&mut self.writer, self.current_indent)
            }
        }
    }

    fn serialize_colon(&mut self) -> io::Result<()> {
        match self.format {
            Format::Compact => self.writer.write_all(b":"),
            Format::Pretty => self.writer.write_all(b": "),
        }
    }

    fn serialize_end(&mut self, current_indent: usize, s: &[u8]) -> io::Result<()> {
        if self.format == Format::Pretty && current_indent != self.current_indent {
            self.current_indent -= self.indent;
            try!(self.writer.write(b"\n"));
            try!(spaces(&mut self.writer, self.current_indent));
        }

        self.writer.write_all(s)
    }
}

impl<W> ser::Serializer for Serializer<W>
    where W: io::Write,
{
    type Error = io::Error;

    #[inline]
    fn visit_bool(&mut self, value: bool) -> io::Result<()> {
        if value {
            self.writer.write_all(b"true")
        } else {
            self.writer.write_all(b"false")
        }
    }

    #[inline]
    fn visit_isize(&mut self, value: isize) -> io::Result<()> {
        write!(&mut self.writer, "{}", value)
    }

    #[inline]
    fn visit_i8(&mut self, value: i8) -> io::Result<()> {
        write!(&mut self.writer, "{}", value)
    }

    #[inline]
    fn visit_i16(&mut self, value: i16) -> io::Result<()> {
        write!(&mut self.writer, "{}", value)
    }

    #[inline]
    fn visit_i32(&mut self, value: i32) -> io::Result<()> {
        write!(&mut self.writer, "{}", value)
    }

    #[inline]
    fn visit_i64(&mut self, value: i64) -> io::Result<()> {
        write!(&mut self.writer, "{}", value)
    }

    #[inline]
    fn visit_usize(&mut self, value: usize) -> io::Result<()> {
        write!(&mut self.writer, "{}", value)
    }

    #[inline]
    fn visit_u8(&mut self, value: u8) -> io::Result<()> {
        write!(&mut self.writer, "{}", value)
    }

    #[inline]
    fn visit_u16(&mut self, value: u16) -> io::Result<()> {
        write!(&mut self.writer, "{}", value)
    }

    #[inline]
    fn visit_u32(&mut self, value: u32) -> io::Result<()> {
        write!(&mut self.writer, "{}", value)
    }

    #[inline]
    fn visit_u64(&mut self, value: u64) -> io::Result<()> {
        write!(&mut self.writer, "{}", value)
    }

    #[inline]
    fn visit_f32(&mut self, value: f32) -> io::Result<()> {
        fmt_f32_or_null(&mut self.writer, value)
    }

    #[inline]
    fn visit_f64(&mut self, value: f64) -> io::Result<()> {
        fmt_f64_or_null(&mut self.writer, value)
    }

    #[inline]
    fn visit_char(&mut self, value: char) -> io::Result<()> {
        escape_char(&mut self.writer, value)
    }

    #[inline]
    fn visit_str(&mut self, value: &str) -> io::Result<()> {
        escape_str(&mut self.writer, value)
    }

    #[inline]
    fn visit_none(&mut self) -> io::Result<()> {
        self.visit_unit()
    }

    #[inline]
    fn visit_some<V>(&mut self, value: V) -> io::Result<()>
        where V: ser::Serialize
    {
        value.serialize(self)
    }

    #[inline]
    fn visit_unit(&mut self) -> io::Result<()> {
        self.writer.write_all(b"null")
    }

    #[inline]
    fn visit_enum_unit(&mut self, _name: &str, variant: &str) -> io::Result<()> {
        let current_indent = self.current_indent;

        try!(self.writer.write_all(b"{"));
        try!(self.serialize_sep(true));
        try!(self.visit_str(variant));
        try!(self.serialize_colon());
        try!(self.writer.write_all(b"[]"));
        self.serialize_end(current_indent, b"}")
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> io::Result<()>
        where V: ser::SeqVisitor,
    {
        let current_indent = self.current_indent;

        try!(self.writer.write_all(b"["));

        while let Some(()) = try!(visitor.visit(self)) { }

        self.serialize_end(current_indent, b"]")
    }

    #[inline]
    fn visit_enum_seq<V>(&mut self, _name: &str, variant: &str, visitor: V) -> io::Result<()>
        where V: ser::SeqVisitor,
    {
        let current_indent = self.current_indent;

        try!(self.writer.write_all(b"{"));
        try!(self.serialize_sep(true));
        try!(self.visit_str(variant));
        try!(self.serialize_colon());
        try!(self.visit_seq(visitor));
        self.serialize_end(current_indent, b"}")
    }

    #[inline]
    fn visit_seq_elt<T>(&mut self, first: bool, value: T) -> io::Result<()>
        where T: ser::Serialize,
    {
        try!(self.serialize_sep(first));

        value.serialize(self)
    }

    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> io::Result<()>
        where V: ser::MapVisitor,
    {
        let current_indent = self.current_indent;

        try!(self.writer.write_all(b"{"));

        while let Some(()) = try!(visitor.visit(self)) { }

        self.serialize_end(current_indent, b"}")
    }

    #[inline]
    fn visit_enum_map<V>(&mut self, _name: &str, variant: &str, visitor: V) -> io::Result<()>
        where V: ser::MapVisitor,
    {
        let current_indent = self.current_indent;

        try!(self.writer.write_all(b"{"));
        try!(self.serialize_sep(true));
        try!(self.visit_str(variant));
        try!(self.serialize_colon());
        try!(self.visit_map(visitor));
        self.serialize_end(current_indent, b"}")
    }

    #[inline]
    fn visit_map_elt<K, V>(&mut self, first: bool, key: K, value: V) -> io::Result<()>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        try!(self.serialize_sep(first));
        try!(key.serialize(self));
        try!(self.serialize_colon());
        value.serialize(self)
    }
}

#[inline]
pub fn escape_bytes<W>(wr: &mut W, bytes: &[u8]) -> io::Result<()>
    where W: io::Write
{
    try!(wr.write_all(b"\""));

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
            try!(wr.write_all(&bytes[start..i]));
        }

        try!(wr.write_all(escaped));

        start = i + 1;
    }

    if start != bytes.len() {
        try!(wr.write_all(&bytes[start..]));
    }

    try!(wr.write_all(b"\""));
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
        FpCategory::Nan | FpCategory::Infinite => wr.write_all(b"null"),
        _ => wr.write_all(f32::to_str_digits(value, 6).as_bytes()),
    }
}

fn fmt_f64_or_null<W>(wr: &mut W, value: f64) -> io::Result<()>
    where W: io::Write
{
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => wr.write_all(b"null"),
        _ => wr.write_all(f64::to_str_digits(value, 6).as_bytes()),
    }
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_writer<W, T>(writer: &mut W, value: &T) -> io::Result<()>
    where W: io::Write,
          T: ser::Serialize,
{
    let mut ser = Serializer::new(writer);
    try!(value.serialize(&mut ser));
    Ok(())
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_writer_pretty<W, T>(writer: &mut W, value: &T) -> io::Result<()>
    where W: io::Write,
          T: ser::Serialize,
{
    let mut ser = Serializer::new_pretty(writer);
    try!(value.serialize(&mut ser));
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

/// Encode the specified struct into a json `[u8]` buffer.
#[inline]
pub fn to_vec_pretty<T>(value: &T) -> Vec<u8>
    where T: ser::Serialize,
{
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let mut writer = Vec::with_capacity(128);
    to_writer_pretty(&mut writer, value).unwrap();
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

/// Encode the specified struct into a json `String` buffer.
#[inline]
pub fn to_string_pretty<T>(value: &T) -> Result<String, FromUtf8Error>
    where T: ser::Serialize
{
    let vec = to_vec_pretty(value);
    String::from_utf8(vec)
}

fn spaces<W>(wr: &mut W, mut n: usize) -> io::Result<()>
    where W: io::Write,
{
    const LEN: usize = 16;
    const BUF: &'static [u8; LEN] = &[b' '; 16];

    while n >= LEN {
        try!(wr.write_all(BUF));
        n -= LEN;
    }

    if n > 0 {
        wr.write_all(&BUF[..n])
    } else {
        Ok(())
    }
}
