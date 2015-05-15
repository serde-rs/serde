use std::io;
use std::num::FpCategory;
use std::string::FromUtf8Error;

use ser;

/// A structure for implementing serialization to JSON.
pub struct Serializer<W, F=CompactFormatter> {
    writer: W,
    formatter: F,

    /// `first` is used to signify if we should print a comma when we are walking through a
    /// sequence.
    first: bool,
}

impl<W> Serializer<W>
    where W: io::Write,
{
    /// Creates a new JSON serializer.
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, CompactFormatter)
    }
}

impl<'a, W> Serializer<W, PrettyFormatter<'a>>
    where W: io::Write,
{
    /// Creates a new JSON pretty print serializer.
    #[inline]
    pub fn pretty(writer: W) -> Self {
        Serializer::with_formatter(writer, PrettyFormatter::new())
    }
}

impl<W, F> Serializer<W, F>
    where W: io::Write,
          F: Formatter,
{
    /// Creates a new JSON visitor whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn with_formatter(writer: W, formatter: F) -> Self {
        Serializer {
            writer: writer,
            formatter: formatter,
            first: false,
        }
    }

    /// Unwrap the `Writer` from the `Serializer`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W, F> ser::Serializer for Serializer<W, F>
    where W: io::Write,
          F: Formatter,
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
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(self.visit_str(variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(self.writer.write_all(b"[]"));
        self.formatter.close(&mut self.writer, b'}')
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> io::Result<()>
        where V: ser::SeqVisitor,
    {
        match visitor.len() {
            Some(len) if len == 0 => {
                self.writer.write_all(b"[]")
            }
            _ => {
                try!(self.formatter.open(&mut self.writer, b'['));

                self.first = true;

                while let Some(()) = try!(visitor.visit(self)) { }

                self.formatter.close(&mut self.writer, b']')
            }
        }

    }

    #[inline]
    fn visit_enum_seq<V>(&mut self, _name: &str, variant: &str, visitor: V) -> io::Result<()>
        where V: ser::SeqVisitor,
    {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(self.visit_str(variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(self.visit_seq(visitor));
        self.formatter.close(&mut self.writer, b'}')
    }

    #[inline]
    fn visit_seq_elt<T>(&mut self, value: T) -> io::Result<()>
        where T: ser::Serialize,
    {
        try!(self.formatter.comma(&mut self.writer, self.first));
        self.first = false;

        value.serialize(self)
    }

    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> io::Result<()>
        where V: ser::MapVisitor,
    {
        match visitor.len() {
            Some(len) if len == 0 => {
                self.writer.write_all(b"{}")
            }
            _ => {
                try!(self.formatter.open(&mut self.writer, b'{'));

                self.first = true;

                while let Some(()) = try!(visitor.visit(self)) { }

                self.formatter.close(&mut self.writer, b'}')
            }
        }
    }

    #[inline]
    fn visit_enum_map<V>(&mut self, _name: &str, variant: &str, visitor: V) -> io::Result<()>
        where V: ser::MapVisitor,
    {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(self.visit_str(variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(self.visit_map(visitor));

        self.formatter.close(&mut self.writer, b'}')
    }

    #[inline]
    fn visit_map_elt<K, V>(&mut self, key: K, value: V) -> io::Result<()>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        try!(self.formatter.comma(&mut self.writer, self.first));
        self.first = false;

        try!(key.serialize(self));
        try!(self.formatter.colon(&mut self.writer));
        value.serialize(self)
    }

    #[inline]
    fn format() -> &'static str {
        "json"
    }
}

pub trait Formatter {
    fn open<W>(&mut self, writer: &mut W, ch: u8) -> io::Result<()>
        where W: io::Write;

    fn comma<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where W: io::Write;

    fn colon<W>(&mut self, writer: &mut W) -> io::Result<()>
        where W: io::Write;

    fn close<W>(&mut self, writer: &mut W, ch: u8) -> io::Result<()>
        where W: io::Write;
}

pub struct CompactFormatter;

impl Formatter for CompactFormatter {
    fn open<W>(&mut self, writer: &mut W, ch: u8) -> io::Result<()>
        where W: io::Write,
    {
        writer.write_all(&[ch])
    }

    fn comma<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where W: io::Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b",")
        }
    }

    fn colon<W>(&mut self, writer: &mut W) -> io::Result<()>
        where W: io::Write,
    {
        writer.write_all(b":")
    }

    fn close<W>(&mut self, writer: &mut W, ch: u8) -> io::Result<()>
        where W: io::Write,
    {
        writer.write_all(&[ch])
    }
}

pub struct PrettyFormatter<'a> {
    current_indent: usize,
    indent: &'a [u8],
}

impl<'a> PrettyFormatter<'a> {
    fn new() -> Self {
        PrettyFormatter::with_indent(b"  ")
    }

    fn with_indent(indent: &'a [u8]) -> Self {
        PrettyFormatter {
            current_indent: 0,
            indent: indent,
        }
    }
}

impl<'a> Formatter for PrettyFormatter<'a> {
    fn open<W>(&mut self, writer: &mut W, ch: u8) -> io::Result<()>
        where W: io::Write,
    {
        self.current_indent += 1;
        writer.write_all(&[ch])
    }

    fn comma<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where W: io::Write,
    {
        if first {
            try!(writer.write_all(b"\n"));
        } else {
            try!(writer.write_all(b",\n"));
        }

        indent(writer, self.current_indent, self.indent)
    }

    fn colon<W>(&mut self, writer: &mut W) -> io::Result<()>
        where W: io::Write,
    {
        writer.write_all(b": ")
    }

    fn close<W>(&mut self, writer: &mut W, ch: u8) -> io::Result<()>
        where W: io::Write,
    {
        self.current_indent -= 1;
        try!(writer.write(b"\n"));
        try!(indent(writer, self.current_indent, self.indent));

        writer.write_all(&[ch])
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
    // FIXME: this allocation is required in order to be compatible with stable
    // rust, which doesn't support encoding a `char` into a stack buffer.
    escape_bytes(wr, value.to_string().as_bytes())
}

fn fmt_f32_or_null<W>(wr: &mut W, value: f32) -> io::Result<()>
    where W: io::Write
{
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => wr.write_all(b"null"),
        _ => {
            let s = format!("{:?}", value);
            try!(wr.write_all(s.as_bytes()));
            if !s.contains('.') {
                try!(wr.write_all(b".0"))
            }
            Ok(())
        }
    }
}

fn fmt_f64_or_null<W>(wr: &mut W, value: f64) -> io::Result<()>
    where W: io::Write
{
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => wr.write_all(b"null"),
        _ => {
            let s = format!("{:?}", value);
            try!(wr.write_all(s.as_bytes()));
            if !s.contains('.') {
                try!(wr.write_all(b".0"))
            }
            Ok(())
        }
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
    let mut ser = Serializer::pretty(writer);
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

fn indent<W>(wr: &mut W, n: usize, s: &[u8]) -> io::Result<()>
    where W: io::Write,
{
    for _ in 0 .. n {
        try!(wr.write_all(s));
    }

    Ok(())
}
