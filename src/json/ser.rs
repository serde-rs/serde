use std::f32;
use std::f64;
use std::num::{Float, FpCategory};
use std::io::{IoError, IoResult};
use std::string::FromUtf8Error;

use ser::Serialize;
use ser;

fn escape_bytes<W: Writer>(wr: &mut W, bytes: &[u8]) -> IoResult<()> {
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

pub fn escape_str<W: Writer>(wr: &mut W, v: &str) -> IoResult<()> {
    escape_bytes(wr, v.as_bytes())
}

fn escape_char<W: Writer>(wr: &mut W, v: char) -> IoResult<()> {
    let buf = &mut [0; 4];
    v.encode_utf8(buf);
    escape_bytes(wr, buf)
}

fn fmt_f32_or_null<W: Writer>(wr: &mut W, v: f32) -> IoResult<()> {
    match v.classify() {
        FpCategory::Nan | FpCategory::Infinite => wr.write_str("null"),
        _ => wr.write_str(&f32::to_str_digits(v, 6)),
    }
}

fn fmt_f64_or_null<W: Writer>(wr: &mut W, v: f64) -> IoResult<()> {
    match v.classify() {
        FpCategory::Nan | FpCategory::Infinite => wr.write_str("null"),
        _ => wr.write_str(&f64::to_str_digits(v, 6)),
    }
}

fn spaces<W: Writer>(wr: &mut W, mut n: usize) -> IoResult<()> {
    const LEN: usize = 16;
    const BUF: &'static [u8; LEN] = &[b' '; LEN];

    while n >= LEN {
        try!(wr.write(BUF));
        n -= LEN;
    }

    if n > 0 {
        wr.write(BUF.slice_to(n))
    } else {
        Ok(())
    }
}

/*
#[derive(Show)]
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

impl<W: Writer> Serializer<W> {
    /// Creates a new JSON serializer whose output will be written to the writer
    /// specified.
    pub fn new(wr: W) -> Serializer<W> {
        Serializer {
            wr: wr,
            first: true,
        }
    }

    /// Unwrap the Writer from the Serializer.
    pub fn unwrap(self) -> W {
        self.wr
    }
}

impl<W: Writer> ser::Serializer<IoError> for Serializer<W> {
    #[inline]
    fn serialize_null(&mut self) -> IoResult<()> {
        self.wr.write_str("null")
    }

    #[inline]
    fn serialize_bool(&mut self, v: bool) -> IoResult<()> {
        if v {
            self.wr.write_str("true")
        } else {
            self.wr.write_str("false")
        }
    }

    #[inline]
    fn serialize_isize(&mut self, v: isize) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i8(&mut self, v: i8) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i16(&mut self, v: i16) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i32(&mut self, v: i32) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i64(&mut self, v: i64) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_usize(&mut self, v: usize) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u8(&mut self, v: u8) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u16(&mut self, v: u16) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u32(&mut self, v: u32) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u64(&mut self, v: u64) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_f32(&mut self, v: f32) -> IoResult<()> {
        fmt_f32_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_f64(&mut self, v: f64) -> IoResult<()> {
        fmt_f64_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_char(&mut self, v: char) -> IoResult<()> {
        escape_char(&mut self.wr, v)
    }

    #[inline]
    fn serialize_str(&mut self, v: &str) -> IoResult<()> {
        escape_str(&mut self.wr, v)
    }

    #[inline]
    fn serialize_tuple_start(&mut self, _len: usize) -> IoResult<()> {
        self.first = true;
        self.wr.write_str("[")
    }

    #[inline]
    fn serialize_tuple_elt<
        T: Serialize<Serializer<W>, IoError>
    >(&mut self, value: &T) -> IoResult<()> {
        if self.first {
            self.first = false;
        } else {
            try!(self.wr.write_str(","));
        }
        value.serialize(self)
    }

    #[inline]
    fn serialize_tuple_end(&mut self) -> IoResult<()> {
        self.wr.write_str("]")
    }

    #[inline]
    fn serialize_struct_start(&mut self, _name: &str, _len: usize) -> IoResult<()> {
        self.first = true;
        self.wr.write_str("{")
    }

    #[inline]
    fn serialize_struct_elt<
        T: Serialize<Serializer<W>, IoError>
    >(&mut self, name: &str, value: &T) -> IoResult<()> {
        if self.first {
            self.first = false;
        } else {
            try!(self.wr.write_str(","));
        }
        try!(name.serialize(self));
        try!(self.wr.write_str(":"));
        value.serialize(self)
    }

    #[inline]
    fn serialize_struct_end(&mut self) -> IoResult<()> {
        self.wr.write_str("}")
    }

    #[inline]
    fn serialize_enum_start(&mut self, _name: &str, variant: &str, _len: usize) -> IoResult<()> {
        self.first = true;
        try!(self.wr.write_str("{"));
        try!(self.serialize_str(variant));
        self.wr.write_str(":[")
    }

    #[inline]
    fn serialize_enum_elt<
        T: Serialize<Serializer<W>, IoError>
    >(&mut self, value: &T) -> IoResult<()> {
        if self.first {
            self.first = false;
        } else {
            try!(self.wr.write_str(","));
        }
        value.serialize(self)
    }

    #[inline]
    fn serialize_enum_end(&mut self) -> IoResult<()> {
        self.wr.write_str("]}")
    }

    #[inline]
    fn serialize_option<
        T: Serialize<Serializer<W>, IoError>
    >(&mut self, v: &Option<T>) -> IoResult<()> {
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
        T: Serialize<Serializer<W>, IoError>,
        Iter: Iterator<Item=T>
    >(&mut self, mut iter: Iter) -> IoResult<()> {
        try!(self.wr.write_str("["));
        let mut first = true;
        for elt in iter {
            if first {
                first = false;
            } else {
                try!(self.wr.write_str(","));
            }
            try!(elt.serialize(self));

        }
        self.wr.write_str("]")
    }

    #[inline]
    fn serialize_map<
        K: Serialize<Serializer<W>, IoError>,
        V: Serialize<Serializer<W>, IoError>,
        Iter: Iterator<Item=(K, V)>
    >(&mut self, mut iter: Iter) -> IoResult<()> {
        try!(self.wr.write_str("{"));
        let mut first = true;
        for (key, value) in iter {
            if first {
                first = false;
            } else {
                try!(self.wr.write_str(","));
            }
            try!(key.serialize(self));
            try!(self.wr.write_str(":"));
            try!(value.serialize(self));
        }
        self.wr.write_str("}")
    }
}

/// Another serializer for JSON, but prints out human-readable JSON instead of
/// compact data
pub struct PrettySerializer<W> {
    wr: W,
    indent: usize,
    first: bool,
}

impl<W: Writer> PrettySerializer<W> {
    /// Creates a new serializer whose output will be written to the specified writer
    pub fn new(wr: W) -> PrettySerializer<W> {
        PrettySerializer {
            wr: wr,
            indent: 0,
            first: true,
        }
    }

    /// Unwrap the Writer from the Serializer.
    pub fn unwrap(self) -> W {
        self.wr
    }

    #[inline]
    fn serialize_sep(&mut self) -> IoResult<()> {
        if self.first {
            self.first = false;
            self.indent += 2;
            try!(self.wr.write_str("\n"));
        } else {
            try!(self.wr.write_str(",\n"));
        }

        spaces(&mut self.wr, self.indent)
    }

    #[inline]
    fn serialize_end(&mut self, s: &str) -> IoResult<()> {
        if !self.first {
            try!(self.wr.write_str("\n"));
            self.indent -= 2;
            try!(spaces(&mut self.wr, self.indent));
        }

        self.first = false;

        self.wr.write_str(s)
    }
}

impl<W: Writer> ser::Serializer<IoError> for PrettySerializer<W> {
    #[inline]
    fn serialize_null(&mut self) -> IoResult<()> {
        self.wr.write_str("null")
    }

    #[inline]
    fn serialize_bool(&mut self, v: bool) -> IoResult<()> {
        if v {
            self.wr.write_str("true")
        } else {
            self.wr.write_str("false")
        }
    }

    #[inline]
    fn serialize_isize(&mut self, v: isize) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i8(&mut self, v: i8) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i16(&mut self, v: i16) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i32(&mut self, v: i32) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_i64(&mut self, v: i64) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_usize(&mut self, v: usize) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u8(&mut self, v: u8) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u16(&mut self, v: u16) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u32(&mut self, v: u32) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_u64(&mut self, v: u64) -> IoResult<()> {
        write!(&mut self.wr, "{}", v)
    }

    #[inline]
    fn serialize_f32(&mut self, v: f32) -> IoResult<()> {
        fmt_f32_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_f64(&mut self, v: f64) -> IoResult<()> {
        fmt_f64_or_null(&mut self.wr, v)
    }

    #[inline]
    fn serialize_char(&mut self, v: char) -> IoResult<()> {
        escape_char(&mut self.wr, v)
    }

    #[inline]
    fn serialize_str(&mut self, v: &str) -> IoResult<()> {
        escape_str(&mut self.wr, v)
    }

    #[inline]
    fn serialize_tuple_start(&mut self, _len: usize) -> IoResult<()> {
        self.first = true;
        self.wr.write_str("[")
    }

    #[inline]
    fn serialize_tuple_elt<
        T: Serialize<PrettySerializer<W>, IoError>
    >(&mut self, value: &T) -> IoResult<()> {
        try!(self.serialize_sep());
        value.serialize(self)
    }

    #[inline]
    fn serialize_tuple_end(&mut self) -> IoResult<()> {
        self.serialize_end("]")
    }

    #[inline]
    fn serialize_struct_start(&mut self, _name: &str, _len: usize) -> IoResult<()> {
        self.first = true;
        self.wr.write_str("{")
    }

    #[inline]
    fn serialize_struct_elt<
        T: Serialize<PrettySerializer<W>, IoError>
    >(&mut self, name: &str, value: &T) -> IoResult<()> {
        try!(self.serialize_sep());
        try!(self.serialize_str(name));
        try!(self.wr.write_str(": "));
        value.serialize(self)
    }

    #[inline]
    fn serialize_struct_end(&mut self) -> IoResult<()> {
        self.serialize_end("}")
    }

    #[inline]
    fn serialize_enum_start(&mut self, _name: &str, variant: &str, _len: usize) -> IoResult<()> {
        self.first = true;
        try!(self.wr.write_str("{"));
        try!(self.serialize_sep());
        try!(self.serialize_str(variant));
        self.first = true;
        self.wr.write_str(": [")
    }

    #[inline]
    fn serialize_enum_elt<
        T: Serialize<PrettySerializer<W>, IoError>
    >(&mut self, value: &T) -> IoResult<()> {
        try!(self.serialize_sep());
        value.serialize(self)
    }

    #[inline]
    fn serialize_enum_end(&mut self) -> IoResult<()> {
        try!(self.serialize_tuple_end());
        self.serialize_struct_end()
    }

    #[inline]
    fn serialize_option<
        T: Serialize<PrettySerializer<W>, IoError>
    >(&mut self, v: &Option<T>) -> IoResult<()> {
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
        T: Serialize<PrettySerializer<W>, IoError>,
        Iter: Iterator<Item=T>
    >(&mut self, mut iter: Iter) -> IoResult<()> {
        try!(self.wr.write_str("["));

        self.first = true;
        for elt in iter {
            try!(self.serialize_sep());
            try!(elt.serialize(self));
        }

        self.serialize_end("]")
    }

    #[inline]
    fn serialize_map<
        K: Serialize<PrettySerializer<W>, IoError>,
        V: Serialize<PrettySerializer<W>, IoError>,
        Iter: Iterator<Item=(K, V)>
    >(&mut self, mut iter: Iter) -> IoResult<()> {
        try!(self.wr.write_str("{"));

        self.first = true;
        for (key, value) in iter {
            try!(self.serialize_sep());
            try!(key.serialize(self));
            try!(self.wr.write_str(": "));
            try!(value.serialize(self));
        }

        self.serialize_end("}")
    }
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_writer<
    W: Writer,
    T: Serialize<Serializer<W>, IoError>
>(writer: W, value: &T) -> IoResult<W> {
    let mut serializer = Serializer::new(writer);
    try!(value.serialize(&mut serializer));
    Ok(serializer.unwrap())
}

/// Encode the specified struct into a json `[u8]` buffer.
#[inline]
pub fn to_vec<
    T: Serialize<Serializer<Vec<u8>>, IoError>
>(value: &T) -> Vec<u8> {
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let writer = Vec::with_capacity(128);
    to_writer(writer, value).unwrap()
}

/// Encode the specified struct into a json `String` buffer.
#[inline]
pub fn to_string<
    T: Serialize<Serializer<Vec<u8>>, IoError>
>(value: &T) -> Result<String, FromUtf8Error> {
    let buf = to_vec(value);
    String::from_utf8(buf)
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_pretty_writer<
    W: Writer,
    T: Serialize<PrettySerializer<W>, IoError>
>(writer: W, value: &T) -> IoResult<W> {
    let mut serializer = PrettySerializer::new(writer);
    try!(value.serialize(&mut serializer));
    Ok(serializer.unwrap())
}

/// Encode the specified struct into a json `[u8]` buffer.
pub fn to_pretty_vec<
    T: Serialize<PrettySerializer<Vec<u8>>, IoError>
>(value: &T) -> Vec<u8> {
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let writer = Vec::with_capacity(128);
    to_pretty_writer(writer, value).unwrap()
}

/// Encode the specified struct into a json `String` buffer.
pub fn to_pretty_string<
    T: Serialize<PrettySerializer<Vec<u8>>, IoError>
>(value: &T) -> Result<String, FromUtf8Error> {
    let buf = to_pretty_vec(value);
    String::from_utf8(buf)
}
