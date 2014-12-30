use std::f64;
use std::io::{mod, ByRefWriter, IoError};
use std::num::{Float, FpCategory};
use std::str::Utf8Error;

use ser;
use ser::Serializer;

/// A structure for implementing serialization to JSON.
pub struct Writer<W> {
    writer: W,
}

impl<W: io::Writer> Writer<W> {
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

impl<W: io::Writer> ser::Serializer<W, (), IoError> for Writer<W> {
    #[inline]
    fn visit<
        T: ser::Serialize,
    >(&mut self, value: &T) -> Result<(), IoError> {
        value.visit(&mut self.writer, Visitor)
    }
}

struct Visitor;

impl<W: io::Writer> ser::Visitor<W, (), IoError> for Visitor {
    #[inline]
    fn visit_null(&self, writer: &mut W) -> Result<(), IoError> {
        writer.write_str("null")
    }

    #[inline]
    fn visit_bool(&self, writer: &mut W, value: bool) -> Result<(), IoError> {
        if value {
            writer.write_str("true")
        } else {
            writer.write_str("false")
        }
    }

    #[inline]
    fn visit_int(&self, writer: &mut W, value: int) -> Result<(), IoError> {
        write!(writer, "{}", value)
    }

    #[inline]
    fn visit_i8(&self, writer: &mut W, value: i8) -> Result<(), IoError> {
        write!(writer, "{}", value)
    }

    #[inline]
    fn visit_i16(&self, writer: &mut W, value: i16) -> Result<(), IoError> {
        write!(writer, "{}", value)
    }

    #[inline]
    fn visit_i32(&self, writer: &mut W, value: i32) -> Result<(), IoError> {
        write!(writer, "{}", value)
    }

    #[inline]
    fn visit_i64(&self, writer: &mut W, value: i64) -> Result<(), IoError> {
        write!(writer, "{}", value)
    }

    #[inline]
    fn visit_uint(&self, writer: &mut W, value: uint) -> Result<(), IoError> {
        write!(writer, "{}", value)
    }

    #[inline]
    fn visit_u8(&self, writer: &mut W, value: u8) -> Result<(), IoError> {
        write!(writer, "{}", value)
    }

    #[inline]
    fn visit_u16(&self, writer: &mut W, value: u16) -> Result<(), IoError> {
        write!(writer, "{}", value)
    }

    #[inline]
    fn visit_u32(&self, writer: &mut W, value: u32) -> Result<(), IoError> {
        write!(writer, "{}", value)
    }

    #[inline]
    fn visit_u64(&self, writer: &mut W, value: u64) -> Result<(), IoError> {
        write!(writer, "{}", value)
    }

    #[inline]
    fn visit_f64(&self, writer: &mut W, value: f64) -> Result<(), IoError> {
        fmt_f64_or_null(writer, value)
    }

    #[inline]
    fn visit_char(&self, writer: &mut W, v: char) -> Result<(), IoError> {
        escape_char(writer, v)
    }

    #[inline]
    fn visit_str(&self, writer: &mut W, value: &str) -> Result<(), IoError> {
        escape_str(writer, value)
    }

    #[inline]
    fn visit_seq<
        V: ser::SeqVisitor<W, (), IoError>
    >(&self, writer: &mut W, mut visitor: V) -> Result<(), IoError> {
        try!(writer.write_str("["));

        loop {
            match try!(visitor.visit(writer, Visitor)) {
                Some(()) => { }
                None => { break; }
            }
        }

        writer.write_str("]")
    }

    #[inline]
    fn visit_seq_elt<
        T: ser::Serialize,
    >(&self, writer: &mut W, first: bool, value: T) -> Result<(), IoError> {
        if !first {
            try!(writer.write_str(","));
        }

        value.visit(writer, Visitor)
    }

    #[inline]
    fn visit_map<
        V: ser::MapVisitor<W, (), IoError>
    >(&self, writer: &mut W, mut visitor: V) -> Result<(), IoError> {
        try!(writer.write_str("{{"));

        loop {
            match try!(visitor.visit(writer, Visitor)) {
                Some(()) => { }
                None => { break; }
            }
        }

        writer.write_str("}}")
    }

    #[inline]
    fn visit_map_elt<
        K: ser::Serialize,
        V: ser::Serialize,
    >(&self, writer: &mut W, first: bool, key: K, value: V) -> Result<(), IoError> {
        if !first {
            try!(writer.write_str(","));
        }

        try!(key.visit(writer, Visitor));
        try!(writer.write_str(":"));
        value.visit(writer, Visitor)
    }
}

#[inline]
pub fn escape_bytes<W: io::Writer>(wr: &mut W, bytes: &[u8]) -> Result<(), IoError> {
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
            try!(wr.write(bytes.slice(start, i)));
        }

        try!(wr.write_str(escaped));

        start = i + 1;
    }

    if start != bytes.len() {
        try!(wr.write(bytes.slice_from(start)));
    }

    wr.write_str("\"")
}

#[inline]
pub fn escape_str<W: io::Writer>(wr: &mut W, value: &str) -> Result<(), IoError> {
    escape_bytes(wr, value.as_bytes())
}

#[inline]
pub fn escape_char<W: io::Writer>(wr: &mut W, value: char) -> Result<(), IoError> {
    let mut buf = &mut [0, .. 4];
    value.encode_utf8(buf);
    escape_bytes(wr, buf)
}

fn fmt_f64_or_null<W: io::Writer>(wr: &mut W, value: f64) -> Result<(), IoError> {
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => wr.write_str("null"),
        _ => wr.write_str(f64::to_str_digits(value, 6).as_slice()),
    }
}

#[inline]
pub fn to_writer<
    W: io::Writer,
    T: ser::Serialize,
>(wr: &mut W, value: &T) -> Result<(), IoError> {
    let mut wr = Writer::new(wr.by_ref());
    try!(wr.visit(value));
    Ok(())
}

#[inline]
pub fn to_vec<
    T: ser::Serialize,
>(value: &T) -> Result<Vec<u8>, IoError> {
    let mut wr = Vec::with_capacity(128);
    to_writer(&mut wr, value).unwrap();
    Ok(wr)
}

#[inline]
pub fn to_string<
    T: ser::Serialize,
>(value: &T) -> Result<Result<String, (Vec<u8>, Utf8Error)>, IoError> {
    let vec = try!(to_vec(value));
    Ok(String::from_utf8(vec))
}
