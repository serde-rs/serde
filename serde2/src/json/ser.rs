use std::io;
use std::num::{FPNaN, FPInfinite};
use std::f64;

use ser;

/// A structure for implementing serialization to JSON.
pub struct Serializer<W> {
    writer: W,
}

impl<W: Writer> Serializer<W> {
    /// Creates a new JSON serializer whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn new(writer: W) -> Serializer<W> {
        Serializer {
            writer: writer,
        }
    }

    /// Unwrap the Writer from the Serializer.
    #[inline]
    pub fn unwrap(self) -> W {
        self.writer
    }
}

impl<W: Writer> ser::VisitorState<io::IoResult<()>> for Serializer<W> {
    #[inline]
    fn visit_null(&mut self) -> io::IoResult<()> {
        self.writer.write_str("null")
    }

    #[inline]
    fn visit_bool(&mut self, value: bool) -> io::IoResult<()> {
        if value {
            self.writer.write_str("true")
        } else {
            self.writer.write_str("false")
        }
    }

    #[inline]
    fn visit_int(&mut self, value: int) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i8(&mut self, value: i8) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i16(&mut self, value: i16) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i32(&mut self, value: i32) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i64(&mut self, value: i64) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_uint(&mut self, value: uint) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u8(&mut self, value: u8) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u16(&mut self, value: u16) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u32(&mut self, value: u32) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u64(&mut self, value: u64) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_f64(&mut self, value: f64) -> io::IoResult<()> {
        fmt_f64_or_null(&mut self.writer, value)
    }

    #[inline]
    fn visit_char(&mut self, v: char) -> io::IoResult<()> {
        escape_char(&mut self.writer, v)
    }

    #[inline]
    fn visit_str(&mut self, value: &str) -> io::IoResult<()> {
        escape_str(&mut self.writer, value)
    }

    #[inline]
    fn visit_seq<
        V: ser::Visitor<Serializer<W>, io::IoResult<()>>
    >(&mut self, mut visitor: V) -> io::IoResult<()> {
        try!(write!(self.writer, "["));

        loop {
            match visitor.visit(self) {
                Some(Ok(())) => { }
                Some(Err(err)) => { return Err(err); }
                None => { break; }
            }
        }

        write!(self.writer, "]")
    }

    #[inline]
    fn visit_seq_elt<
        T: ser::Serialize<Serializer<W>, io::IoResult<()>>
    >(&mut self, first: bool, value: T) -> io::IoResult<()> {
        if !first {
            try!(write!(self.writer, ","));
        }

        value.serialize(self)
    }

    #[inline]
    fn visit_map<
        V: ser::Visitor<Serializer<W>, io::IoResult<()>>
    >(&mut self, mut visitor: V) -> io::IoResult<()> {
        try!(write!(self.writer, "{{"));

        loop {
            match visitor.visit(self) {
                Some(Ok(())) => { }
                Some(Err(err)) => { return Err(err); }
                None => { break; }
            }
        }
        write!(self.writer, "}}")
    }

    #[inline]
    fn visit_map_elt<
        K: ser::Serialize<Serializer<W>, io::IoResult<()>>,
        V: ser::Serialize<Serializer<W>, io::IoResult<()>>
    >(&mut self, first: bool, key: K, value: V) -> io::IoResult<()> {
        if !first {
            try!(write!(self.writer, ","));
        }

        try!(key.serialize(self));
        try!(write!(self.writer, ":"));
        value.serialize(self)
    }
}

fn escape_bytes<W: Writer>(wr: &mut W, bytes: &[u8]) -> io::IoResult<()> {
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

fn escape_str<W: Writer>(wr: &mut W, value: &str) -> io::IoResult<()> {
    escape_bytes(wr, value.as_bytes())
}

pub fn escape_char<W: Writer>(wr: &mut W, value: char) -> io::IoResult<()> {
    let mut buf = [0, .. 4];
    value.encode_utf8(buf);
    escape_bytes(wr, buf)
}

fn fmt_f64_or_null<W: Writer>(wr: &mut W, value: f64) -> io::IoResult<()> {
    match value.classify() {
        FPNaN | FPInfinite => wr.write_str("null"),
        _ => wr.write_str(f64::to_str_digits(value, 6).as_slice()),
    }
}

#[inline]
pub fn to_vec<
    T: ser::Serialize<Serializer<io::MemWriter>, io::IoResult<()>>
>(value: &T) -> io::IoResult<Vec<u8>> {
    let writer = io::MemWriter::new();
    let mut state = Serializer::new(writer);
    try!(value.serialize(&mut state));
    Ok(state.unwrap().unwrap())
}

#[inline]
pub fn to_string<
    T: ser::Serialize<Serializer<io::MemWriter>, io::IoResult<()>>
>(value: &T) -> io::IoResult<Result<String, Vec<u8>>> {
    let vec = try!(to_vec(value));
    Ok(String::from_utf8(vec))
}
