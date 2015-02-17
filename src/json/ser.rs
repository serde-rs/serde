use std::f32;
use std::f64;
use std::num::{Float, FpCategory};
use std::io;
use std::string::FromUtf8Error;

use ser;

/// A structure for implementing serialization to JSON.
pub struct Serializer<W> {
    wr: W,
    first: bool,
}

impl<W: io::Write> Serializer<W> {
    /// Creates a new JSON serializer whose output will be written to the writer
    /// specified.
    pub fn new(wr: W) -> Serializer<W> {
        Serializer {
            wr: wr,
            first: true,
        }
    }

    /// Unwrap the io::Write from the Serializer.
    pub fn unwrap(self) -> W {
        self.wr
    }
}

impl<W: io::Write> ser::Serializer<io::Error> for Serializer<W> {
    #[inline]
    fn serialize_null(&mut self) -> io::Result<()> {
        self.wr.write_all(b"null")
    }

    #[inline]
    fn serialize_bool(&mut self, value: bool) -> io::Result<()> {
        if value {
            self.wr.write_all(b"true")
        } else {
            self.wr.write_all(b"false")
        }
    }

    #[inline]
    fn serialize_isize(&mut self, value: isize) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_i8(&mut self, value: i8) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_i16(&mut self, value: i16) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_i32(&mut self, value: i32) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_i64(&mut self, value: i64) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_usize(&mut self, value: usize) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_u8(&mut self, value: u8) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_u16(&mut self, value: u16) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_u32(&mut self, value: u32) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_u64(&mut self, value: u64) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_f32(&mut self, value: f32) -> io::Result<()> {
        fmt_f32_or_null(&mut self.wr, value)
    }

    #[inline]
    fn serialize_f64(&mut self, value: f64) -> io::Result<()> {
        fmt_f64_or_null(&mut self.wr, value)
    }

    #[inline]
    fn serialize_char(&mut self, value: char) -> io::Result<()> {
        escape_char(&mut self.wr, value)
    }

    #[inline]
    fn serialize_str(&mut self, value: &str) -> io::Result<()> {
        escape_str(&mut self.wr, value)
    }

    #[inline]
    fn serialize_tuple_start(&mut self, _len: usize) -> io::Result<()> {
        self.first = true;
        write!(&mut self.wr, "[")
    }

    #[inline]
    fn serialize_tuple_elt<
        T: ser::Serialize<Serializer<W>, io::Error>
    >(&mut self, value: &T) -> io::Result<()> {
        if self.first {
            self.first = false;
        } else {
            try!(write!(&mut self.wr, ","));
        }
        value.serialize(self)
    }

    #[inline]
    fn serialize_tuple_end(&mut self) -> io::Result<()> {
        write!(&mut self.wr, "]")
    }

    #[inline]
    fn serialize_struct_start(&mut self, _name: &str, _len: usize) -> io::Result<()> {
        self.first = true;
        write!(&mut self.wr, "{{")
    }

    #[inline]
    fn serialize_struct_elt<
        T: ser::Serialize<Serializer<W>, io::Error>
    >(&mut self, name: &str, value: &T) -> io::Result<()> {
        use ser::Serialize;

        if self.first {
            self.first = false;
        } else {
            try!(write!(&mut self.wr, ","));
        }
        try!(name.serialize(self));
        try!(write!(&mut self.wr, ":"));
        value.serialize(self)
    }

    #[inline]
    fn serialize_struct_end(&mut self) -> io::Result<()> {
        write!(&mut self.wr, "}}")
    }

    #[inline]
    fn serialize_enum_start(&mut self, _name: &str, variant: &str, _len: usize) -> io::Result<()> {
        self.first = true;
        try!(write!(&mut self.wr, "{{"));
        try!(self.serialize_str(variant));
        write!(&mut self.wr, ":[")
    }

    #[inline]
    fn serialize_enum_elt<
        T: ser::Serialize<Serializer<W>, io::Error>
    >(&mut self, value: &T) -> io::Result<()> {
        if self.first {
            self.first = false;
        } else {
            try!(write!(&mut self.wr, ","));
        }
        value.serialize(self)
    }

    #[inline]
    fn serialize_enum_end(&mut self) -> io::Result<()> {
        write!(&mut self.wr, "]}}")
    }

    #[inline]
    fn serialize_option<
        T: ser::Serialize<Serializer<W>, io::Error>
    >(&mut self, v: &Option<T>) -> io::Result<()> {
        match *v {
            Some(ref v) => {
                v.serialize(self)
            }
            None => {
                self.serialize_null()
            }
        }
    }

    #[inline]
    fn serialize_seq<
        T: ser::Serialize<Serializer<W>, io::Error>,
        Iter: Iterator<Item=T>
    >(&mut self, iter: Iter) -> io::Result<()> {
        try!(write!(&mut self.wr, "["));
        let mut first = true;
        for elt in iter {
            if first {
                first = false;
            } else {
                try!(write!(&mut self.wr, ","));
            }
            try!(elt.serialize(self));

        }
        write!(&mut self.wr, "]")
    }

    #[inline]
    fn serialize_map<
        K: ser::Serialize<Serializer<W>, io::Error>,
        V: ser::Serialize<Serializer<W>, io::Error>,
        Iter: Iterator<Item=(K, V)>
    >(&mut self, iter: Iter) -> io::Result<()> {
        try!(write!(&mut self.wr, "{{"));
        let mut first = true;
        for (key, value) in iter {
            if first {
                first = false;
            } else {
                try!(write!(&mut self.wr, ","));
            }
            try!(key.serialize(self));
            try!(write!(&mut self.wr, ":"));
            try!(value.serialize(self));
        }
        write!(&mut self.wr, "}}")
    }
}

/// Another serializer for JSON, but prints out human-readable JSON instead of
/// compact data
pub struct PrettySerializer<W> {
    wr: W,
    indent: usize,
    first: bool,
}

impl<W: io::Write> PrettySerializer<W> {
    /// Creates a new serializer whose output will be written to the specified writer
    pub fn new(wr: W) -> PrettySerializer<W> {
        PrettySerializer {
            wr: wr,
            indent: 0,
            first: true,
        }
    }

    /// Unwrap the io::Write from the Serializer.
    pub fn unwrap(self) -> W {
        self.wr
    }

    #[inline]
    fn serialize_sep(&mut self) -> io::Result<()> {
        if self.first {
            self.first = false;
            self.indent += 2;
            try!(write!(&mut self.wr, "\n"));
        } else {
            try!(write!(&mut self.wr, ",\n"));
        }

        spaces(&mut self.wr, self.indent)
    }

    #[inline]
    fn serialize_end(&mut self, s: &str) -> io::Result<()> {
        if !self.first {
            try!(write!(&mut self.wr, "\n"));
            self.indent -= 2;
            try!(spaces(&mut self.wr, self.indent));
        }

        self.first = false;

        write!(&mut self.wr, "{}", s)
    }
}

impl<W: io::Write> ser::Serializer<io::Error> for PrettySerializer<W> {
    #[inline]
    fn serialize_null(&mut self) -> io::Result<()> {
        write!(&mut self.wr, "null")
    }

    #[inline]
    fn serialize_bool(&mut self, v: bool) -> io::Result<()> {
        if v {
            self.wr.write_all(b"true")
        } else {
            self.wr.write_all(b"false")
        }
    }

    #[inline]
    fn serialize_isize(&mut self, value: isize) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_i8(&mut self, value: i8) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_i16(&mut self, value: i16) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_i32(&mut self, value: i32) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_i64(&mut self, value: i64) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_usize(&mut self, value: usize) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_u8(&mut self, value: u8) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_u16(&mut self, value: u16) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_u32(&mut self, value: u32) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_u64(&mut self, value: u64) -> io::Result<()> {
        write!(&mut self.wr, "{}", value)
    }

    #[inline]
    fn serialize_f32(&mut self, value: f32) -> io::Result<()> {
        fmt_f32_or_null(&mut self.wr, value)
    }

    #[inline]
    fn serialize_f64(&mut self, value: f64) -> io::Result<()> {
        fmt_f64_or_null(&mut self.wr, value)
    }

    #[inline]
    fn serialize_char(&mut self, value: char) -> io::Result<()> {
        escape_char(&mut self.wr, value)
    }

    #[inline]
    fn serialize_str(&mut self, value: &str) -> io::Result<()> {
        escape_str(&mut self.wr, value)
    }

    #[inline]
    fn serialize_tuple_start(&mut self, _len: usize) -> io::Result<()> {
        self.first = true;
        self.wr.write_all(b"[")
    }

    #[inline]
    fn serialize_tuple_elt<
        T: ser::Serialize<PrettySerializer<W>, io::Error>
    >(&mut self, value: &T) -> io::Result<()> {
        try!(self.serialize_sep());
        value.serialize(self)
    }

    #[inline]
    fn serialize_tuple_end(&mut self) -> io::Result<()> {
        self.serialize_end("]")
    }

    #[inline]
    fn serialize_struct_start(&mut self, _name: &str, _len: usize) -> io::Result<()> {
        self.first = true;
        self.wr.write_all(b"{")
    }

    #[inline]
    fn serialize_struct_elt<
        T: ser::Serialize<PrettySerializer<W>, io::Error>
    >(&mut self, name: &str, value: &T) -> io::Result<()> {
        try!(self.serialize_sep());
        try!(self.serialize_str(name));
        try!(self.wr.write_all(b": "));
        value.serialize(self)
    }

    #[inline]
    fn serialize_struct_end(&mut self) -> io::Result<()> {
        self.serialize_end("}")
    }

    #[inline]
    fn serialize_enum_start(&mut self, _name: &str, variant: &str, _len: usize) -> io::Result<()> {
        self.first = true;
        try!(self.wr.write_all(b"{"));
        try!(self.serialize_sep());
        try!(self.serialize_str(variant));
        self.first = true;
        self.wr.write_all(b": [")
    }

    #[inline]
    fn serialize_enum_elt<
        T: ser::Serialize<PrettySerializer<W>, io::Error>
    >(&mut self, value: &T) -> io::Result<()> {
        try!(self.serialize_sep());
        value.serialize(self)
    }

    #[inline]
    fn serialize_enum_end(&mut self) -> io::Result<()> {
        try!(self.serialize_tuple_end());
        self.serialize_struct_end()
    }

    #[inline]
    fn serialize_option<
        T: ser::Serialize<PrettySerializer<W>, io::Error>
    >(&mut self, v: &Option<T>) -> io::Result<()> {
        match *v {
            Some(ref v) => {
                v.serialize(self)
            }
            None => {
                self.serialize_null()
            }
        }
    }

    #[inline]
    fn serialize_seq<
        T: ser::Serialize<PrettySerializer<W>, io::Error>,
        Iter: Iterator<Item=T>
    >(&mut self, iter: Iter) -> io::Result<()> {
        try!(self.wr.write_all(b"["));

        self.first = true;
        for elt in iter {
            try!(self.serialize_sep());
            try!(elt.serialize(self));
        }

        self.serialize_end("]")
    }

    #[inline]
    fn serialize_map<
        K: ser::Serialize<PrettySerializer<W>, io::Error>,
        V: ser::Serialize<PrettySerializer<W>, io::Error>,
        Iter: Iterator<Item=(K, V)>
    >(&mut self, iter: Iter) -> io::Result<()> {
        try!(self.wr.write_all(b"{"));

        self.first = true;
        for (key, value) in iter {
            try!(self.serialize_sep());
            try!(key.serialize(self));
            try!(self.wr.write_all(b": "));
            try!(value.serialize(self));
        }

        self.serialize_end("}")
    }
}

fn escape_bytes<W>(wr: &mut W, bytes: &[u8]) -> io::Result<()>
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

    wr.write_all(b"\"")
}

pub fn escape_str<W>(wr: &mut W, value: &str) -> io::Result<()>
    where W: io::Write
{
    escape_bytes(wr, value.as_bytes())
}

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

fn spaces<W: io::Write>(wr: &mut W, mut n: usize) -> io::Result<()> {
    const LEN: usize = 16;
    const BUF: &'static [u8; LEN] = &[b' '; LEN];

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

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> io::Result<W>
    where W: io::Write,
          T: ser::Serialize<Serializer<W>, io::Error>
{
    let mut serializer = Serializer::new(writer);
    try!(value.serialize(&mut serializer));
    Ok(serializer.unwrap())
}

/// Encode the specified struct into a json `[u8]` buffer.
#[inline]
pub fn to_vec<T>(value: &T) -> Vec<u8>
    where T: ser::Serialize<Serializer<Vec<u8>>, io::Error>
{
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let writer = Vec::with_capacity(128);
    to_writer(writer, value).unwrap()
}

/// Encode the specified struct into a json `String` buffer.
#[inline]
pub fn to_string<T>(value: &T) -> Result<String, FromUtf8Error>
    where T: ser::Serialize<Serializer<Vec<u8>>, io::Error>
{
    let vec = to_vec(value);
    String::from_utf8(vec)
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_pretty_writer<
    W: io::Write,
    T: ser::Serialize<PrettySerializer<W>, io::Error>
>(writer: W, value: &T) -> io::Result<W> {
    let mut serializer = PrettySerializer::new(writer);
    try!(value.serialize(&mut serializer));
    Ok(serializer.unwrap())
}

/// Encode the specified struct into a json `[u8]` buffer.
pub fn to_pretty_vec<
    T: ser::Serialize<PrettySerializer<Vec<u8>>, io::Error>
>(value: &T) -> Vec<u8> {
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let writer = Vec::with_capacity(128);
    to_pretty_writer(writer, value).unwrap()
}

/// Encode the specified struct into a json `String` buffer.
pub fn to_pretty_string<
    T: ser::Serialize<PrettySerializer<Vec<u8>>, io::Error>
>(value: &T) -> Result<String, FromUtf8Error> {
    let buf = to_pretty_vec(value);
    String::from_utf8(buf)
}
