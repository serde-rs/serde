use std::f32;
use std::f64;
use std::num::{Float, FpCategory};
use std::io;
use std::string::FromUtf8Error;

use ser::Serialize;
use ser;

fn escape_bytes<W: io::Write>(wr: &mut W, bytes: &[u8]) -> io::Result<()> {
    try!(write!(wr, "\""));

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

        try!(write!(wr, "{}", escaped));

        start = i + 1;
    }

    if start != bytes.len() {
        try!(wr.write_all(&bytes[start..]));
    }

    write!(wr, "\"")
}

pub fn escape_str<W: io::Write>(wr: &mut W, v: &str) -> io::Result<()> {
    escape_bytes(wr, v.as_bytes())
}

fn escape_char<W: io::Write>(wr: &mut W, v: char) -> io::Result<()> {
    let buf = &mut [0; 4];
    v.encode_utf8(buf);
    escape_bytes(wr, buf)
}

fn fmt_f32_or_null<W: io::Write>(wr: &mut W, v: f32) -> io::Result<()> {
    match v.classify() {
        FpCategory::Nan | FpCategory::Infinite => write!(wr, "null"),
        _ => write!(wr, "{}", f32::to_str_digits(v, 6)),
    }
}

fn fmt_f64_or_null<W: io::Write>(wr: &mut W, v: f64) -> io::Result<()> {
    match v.classify() {
        FpCategory::Nan | FpCategory::Infinite => write!(wr, "null"),
        _ => write!(wr, "{}", f64::to_str_digits(v, 6)),
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

/*
#[derive(Debug)]
enum SerializerState {
    ValueState,
    TupleState,
    StructState,
    EnumState,
}
*/

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
        write!(&mut self.wr, "null")
    }

    #[inline]
    fn serialize_bool(&mut self, v: bool) -> io::Result<()> {
        if v {
            write!(&mut self.wr, "true")
        } else {
            write!(&mut self.wr, "false")
        }
    }

    #[inline]
    fn serialize_isize(&mut self, v: isize) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i8(&mut self, v: i8) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i16(&mut self, v: i16) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i32(&mut self, v: i32) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i64(&mut self, v: i64) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_usize(&mut self, v: usize) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u8(&mut self, v: u8) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u16(&mut self, v: u16) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u32(&mut self, v: u32) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u64(&mut self, v: u64) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_f32(&mut self, v: f32) -> io::Result<()> {
        fmt_f32_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_f64(&mut self, v: f64) -> io::Result<()> {
        fmt_f64_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_char(&mut self, v: char) -> io::Result<()> {
        escape_char(&mut self.wr, v)
    }

    #[inline]
    fn serialize_str(&mut self, v: &str) -> io::Result<()> {
        escape_str(&mut self.wr, v)
    }

    #[inline]
    fn serialize_tuple_start(&mut self, _len: usize) -> io::Result<()> {
        self.first = true;
        write!(&mut self.wr, "[")
    }

    #[inline]
    fn serialize_tuple_elt<
        T: Serialize<Serializer<W>, io::Error>
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
        T: Serialize<Serializer<W>, io::Error>
    >(&mut self, name: &str, value: &T) -> io::Result<()> {
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
        T: Serialize<Serializer<W>, io::Error>
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
        T: Serialize<Serializer<W>, io::Error>
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
        T: Serialize<Serializer<W>, io::Error>,
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
        K: Serialize<Serializer<W>, io::Error>,
        V: Serialize<Serializer<W>, io::Error>,
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
            write!(&mut self.wr, "true")
        } else {
            write!(&mut self.wr, "false")
        }
    }

    #[inline]
    fn serialize_isize(&mut self, v: isize) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i8(&mut self, v: i8) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i16(&mut self, v: i16) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i32(&mut self, v: i32) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i64(&mut self, v: i64) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_usize(&mut self, v: usize) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u8(&mut self, v: u8) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u16(&mut self, v: u16) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u32(&mut self, v: u32) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u64(&mut self, v: u64) -> io::Result<()> {
        write!(&mut &mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_f32(&mut self, v: f32) -> io::Result<()> {
        fmt_f32_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_f64(&mut self, v: f64) -> io::Result<()> {
        fmt_f64_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_char(&mut self, v: char) -> io::Result<()> {
        escape_char(&mut self.wr, v)
    }

    #[inline]
    fn serialize_str(&mut self, v: &str) -> io::Result<()> {
        escape_str(&mut self.wr, v)
    }

    #[inline]
    fn serialize_tuple_start(&mut self, _len: usize) -> io::Result<()> {
        self.first = true;
        write!(&mut self.wr, "[")
    }

    #[inline]
    fn serialize_tuple_elt<
        T: Serialize<PrettySerializer<W>, io::Error>
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
        write!(&mut self.wr, "{{")
    }

    #[inline]
    fn serialize_struct_elt<
        T: Serialize<PrettySerializer<W>, io::Error>
    >(&mut self, name: &str, value: &T) -> io::Result<()> {
        try!(self.serialize_sep());
        try!(self.serialize_str(name));
        try!(write!(&mut self.wr, ": "));
        value.serialize(self)
    }

    #[inline]
    fn serialize_struct_end(&mut self) -> io::Result<()> {
        self.serialize_end("}")
    }

    #[inline]
    fn serialize_enum_start(&mut self, _name: &str, variant: &str, _len: usize) -> io::Result<()> {
        self.first = true;
        try!(write!(&mut self.wr, "{{"));
        try!(self.serialize_sep());
        try!(self.serialize_str(variant));
        self.first = true;
        write!(&mut self.wr, ": [")
    }

    #[inline]
    fn serialize_enum_elt<
        T: Serialize<PrettySerializer<W>, io::Error>
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
        T: Serialize<PrettySerializer<W>, io::Error>
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
        T: Serialize<PrettySerializer<W>, io::Error>,
        Iter: Iterator<Item=T>
    >(&mut self, iter: Iter) -> io::Result<()> {
        try!(write!(&mut self.wr, "["));

        self.first = true;
        for elt in iter {
            try!(self.serialize_sep());
            try!(elt.serialize(self));
        }

        self.serialize_end("]")
    }

    #[inline]
    fn serialize_map<
        K: Serialize<PrettySerializer<W>, io::Error>,
        V: Serialize<PrettySerializer<W>, io::Error>,
        Iter: Iterator<Item=(K, V)>
    >(&mut self, iter: Iter) -> io::Result<()> {
        try!(write!(&mut self.wr, "{{"));

        self.first = true;
        for (key, value) in iter {
            try!(self.serialize_sep());
            try!(key.serialize(self));
            try!(write!(&mut self.wr, ": "));
            try!(value.serialize(self));
        }

        self.serialize_end("}")
    }
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_writer<
    W: io::Write,
    T: Serialize<Serializer<W>, io::Error>
>(writer: W, value: &T) -> io::Result<W> {
    let mut serializer = Serializer::new(writer);
    try!(value.serialize(&mut serializer));
    Ok(serializer.unwrap())
}

/// Encode the specified struct into a json `[u8]` buffer.
#[inline]
pub fn to_vec<
    T: Serialize<Serializer<Vec<u8>>, io::Error>
>(value: &T) -> Vec<u8> {
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let writer = Vec::with_capacity(128);
    to_writer(writer, value).unwrap()
}

/// Encode the specified struct into a json `String` buffer.
#[inline]
pub fn to_string<
    T: Serialize<Serializer<Vec<u8>>, io::Error>
>(value: &T) -> Result<String, FromUtf8Error> {
    let buf = to_vec(value);
    String::from_utf8(buf)
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_pretty_writer<
    W: io::Write,
    T: Serialize<PrettySerializer<W>, io::Error>
>(writer: W, value: &T) -> io::Result<W> {
    let mut serializer = PrettySerializer::new(writer);
    try!(value.serialize(&mut serializer));
    Ok(serializer.unwrap())
}

/// Encode the specified struct into a json `[u8]` buffer.
pub fn to_pretty_vec<
    T: Serialize<PrettySerializer<Vec<u8>>, io::Error>
>(value: &T) -> Vec<u8> {
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let writer = Vec::with_capacity(128);
    to_pretty_writer(writer, value).unwrap()
}

/// Encode the specified struct into a json `String` buffer.
pub fn to_pretty_string<
    T: Serialize<PrettySerializer<Vec<u8>>, io::Error>
>(value: &T) -> Result<String, FromUtf8Error> {
    let buf = to_pretty_vec(value);
    String::from_utf8(buf)
}
