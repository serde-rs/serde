use std::char;
use std::i32;
use std::io;
use std::str;

use serde::de;
use serde::iter::LineColIterator;

use super::error::{Error, ErrorCode, Result};

pub struct Deserializer<Iter: Iterator<Item=io::Result<u8>>> {
    rdr: LineColIterator<Iter>,
    ch: Option<u8>,
    str_buf: Vec<u8>,
}

macro_rules! try_or_invalid {
    ($self_:expr, $e:expr) => {
        match $e {
            Some(v) => v,
            None => { return Err($self_.error(ErrorCode::InvalidNumber)); }
        }
    }
}

impl<Iter> Deserializer<Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
{
    /// Creates the JSON parser from an `std::iter::Iterator`.
    #[inline]
    pub fn new(rdr: Iter) -> Deserializer<Iter> {
        Deserializer {
            rdr: LineColIterator::new(rdr),
            ch: None,
            str_buf: Vec::with_capacity(128),
        }
    }

    #[inline]
    pub fn end(&mut self) -> Result<()> {
        try!(self.parse_whitespace());
        if try!(self.eof()) {
            Ok(())
        } else {
            Err(self.error(ErrorCode::TrailingCharacters))
        }
    }

    fn eof(&mut self) -> Result<bool> {
        Ok(try!(self.peek()).is_none())
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        match self.ch {
            Some(ch) => Ok(Some(ch)),
            None => {
                self.ch = try!(self.next_char());
                Ok(self.ch)
            }
        }
    }

    fn peek_or_null(&mut self) -> Result<u8> {
        Ok(try!(self.peek()).unwrap_or(b'\x00'))
    }

    fn eat_char(&mut self) {
        self.ch = None;
    }

    fn next_char(&mut self) -> Result<Option<u8>> {
        match self.ch.take() {
            Some(ch) => Ok(Some(ch)),
            None => {
                match self.rdr.next() {
                    Some(Err(err)) => Err(Error::IoError(err)),
                    Some(Ok(ch)) => Ok(Some(ch)),
                    None => Ok(None),
                }
            }
        }
    }

    fn next_char_or_null(&mut self) -> Result<u8> {
        Ok(try!(self.next_char()).unwrap_or(b'\x00'))
    }

    fn error(&mut self, reason: ErrorCode) -> Error {
        Error::SyntaxError(reason, self.rdr.line(), self.rdr.col())
    }

    fn parse_whitespace(&mut self) -> Result<()> {
        loop {
            match try!(self.peek_or_null()) {
                b' ' | b'\n' | b'\t' | b'\r' => {
                    self.eat_char();
                }
                _ => { return Ok(()); }
            }
        }
    }

    fn parse_value<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        try!(self.parse_whitespace());

        if try!(self.eof()) {
            return Err(self.error(ErrorCode::EOFWhileParsingValue));
        }

        let value = match try!(self.peek_or_null()) {
            b'n' => {
                self.eat_char();
                try!(self.parse_ident(b"ull"));
                visitor.visit_unit()
            }
            b't' => {
                self.eat_char();
                try!(self.parse_ident(b"rue"));
                visitor.visit_bool(true)
            }
            b'f' => {
                self.eat_char();
                try!(self.parse_ident(b"alse"));
                visitor.visit_bool(false)
            }
            b'-' => {
                self.eat_char();
                self.parse_integer(false, visitor)
            }
            b'0' ... b'9' => {
                self.parse_integer(true, visitor)
            }
            b'"' => {
                self.eat_char();
                try!(self.parse_string());
                let s = str::from_utf8(&self.str_buf).unwrap();
                visitor.visit_str(s)
            }
            b'[' => {
                self.eat_char();
                visitor.visit_seq(SeqVisitor::new(self))
            }
            b'{' => {
                self.eat_char();
                visitor.visit_map(MapVisitor::new(self))
            }
            _ => {
                Err(self.error(ErrorCode::ExpectedSomeValue))
            }
        };

        match value {
            Ok(value) => Ok(value),
            Err(Error::SyntaxError(code, _, _)) => Err(self.error(code)),
            Err(err) => Err(err),
        }
    }

    fn parse_ident(&mut self, ident: &[u8]) -> Result<()> {
        for c in ident {
            if Some(*c) != try!(self.next_char()) {
                return Err(self.error(ErrorCode::ExpectedSomeIdent));
            }
        }

        Ok(())
    }

    fn parse_integer<V>(&mut self, pos: bool, visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        match try!(self.next_char_or_null()) {
            b'0' => {
                // There can be only one leading '0'.
                match try!(self.peek_or_null()) {
                    b'0' ... b'9' => {
                        Err(self.error(ErrorCode::InvalidNumber))
                    }
                    _ => {
                        self.parse_number(pos, 0, visitor)
                    }
                }
            },
            c @ b'1' ... b'9' => {
                let mut res: u64 = (c as u64) - ('0' as u64);

                loop {
                    match try!(self.peek_or_null()) {
                        c @ b'0' ... b'9' => {
                            self.eat_char();

                            let digit = (c as u64) - ('0' as u64);

                            // We need to be careful with overflow. If we can, try to keep the
                            // number as a `u64` until we grow too large. At that point, switch to
                            // parsing the value as a `f64`.
                            match res.checked_mul(10).and_then(|val| val.checked_add(digit)) {
                                Some(res_) => { res = res_; }
                                None => {
                                    return self.parse_float(
                                        pos,
                                        (res as f64) * 10.0 + (digit as f64),
                                        visitor);
                                }
                            }
                        }
                        _ => {
                            return self.parse_number(pos, res, visitor);
                        }
                    }
                }
            }
            _ => {
                Err(self.error(ErrorCode::InvalidNumber))
            }
        }
    }

    fn parse_float<V>(&mut self,
                      pos: bool,
                      mut res: f64,
                      mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        loop {
            match try!(self.next_char_or_null()) {
                c @ b'0' ... b'9' => {
                    let digit = (c as u64) - ('0' as u64);

                    res *= 10.0;
                    res += digit as f64;
                }
                _ => {
                    match try!(self.peek_or_null()) {
                        b'.' => {
                            return self.parse_decimal(pos, res, visitor);
                        }
                        b'e' | b'E' => {
                            return self.parse_exponent(pos, res, visitor);
                        }
                        _ => {
                            if !pos {
                                res = -res;
                            }

                            return visitor.visit_f64(res);
                        }
                    }
                }
            }
        }
    }

    fn parse_number<V>(&mut self,
                       pos: bool,
                       res: u64,
                       mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        match try!(self.peek_or_null()) {
            b'.' => {
                self.parse_decimal(pos, res as f64, visitor)
            }
            b'e' | b'E' => {
                self.parse_exponent(pos, res as f64, visitor)
            }
            _ => {
                if pos {
                    visitor.visit_u64(res)
                } else {
                    // FIXME: `wrapping_neg` will be stable in Rust 1.2
                    //let res_i64 = (res as i64).wrapping_neg();
                    let res_i64 = (!res + 1) as i64;

                    // Convert into a float if we underflow.
                    if res_i64 > 0 {
                        visitor.visit_f64(-(res as f64))
                    } else {
                        visitor.visit_i64(res_i64)
                    }
                }
            }
        }
    }

    fn parse_decimal<V>(&mut self,
                        pos: bool,
                        mut res: f64,
                        mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.eat_char();

        let mut dec = 0.1;

        // Make sure a digit follows the decimal place.
        match try!(self.next_char_or_null()) {
            c @ b'0' ... b'9' => {
                res += (((c as u64) - (b'0' as u64)) as f64) * dec;
            }
             _ => { return Err(self.error(ErrorCode::InvalidNumber)); }
        }

        loop {
            match try!(self.peek_or_null()) {
                c @ b'0' ... b'9' => {
                    self.eat_char();

                    dec /= 10.0;
                    res += (((c as u64) - (b'0' as u64)) as f64) * dec;
                }
                _ => { break; }
            }
        }

        match try!(self.peek_or_null()) {
            b'e' | b'E' => {
                self.parse_exponent(pos, res, visitor)
            }
            _ => {
                if pos {
                    visitor.visit_f64(res)
                } else {
                    visitor.visit_f64(-res)
                }
            }
        }

    }

    fn parse_exponent<V>(&mut self,
                         pos: bool,
                         mut res: f64,
                         mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.eat_char();

        let pos_exp = match try!(self.peek_or_null()) {
            b'+' => { self.eat_char(); true }
            b'-' => { self.eat_char(); false }
            _ => { true }
        };

        // Make sure a digit follows the exponent place.
        let mut exp = match try!(self.next_char_or_null()) {
            c @ b'0' ... b'9' => { (c as u64) - (b'0' as u64) }
            _ => { return Err(self.error(ErrorCode::InvalidNumber)); }
        };

        loop {
            match try!(self.peek_or_null()) {
                c @ b'0' ... b'9' => {
                    self.eat_char();

                    exp = try_or_invalid!(self, exp.checked_mul(10));
                    exp = try_or_invalid!(self, exp.checked_add((c as u64) - (b'0' as u64)));
                }
                _ => { break; }
            }
        }

        let exp = if exp <= i32::MAX as u64 {
            10_f64.powi(exp as i32)
        } else {
            return Err(self.error(ErrorCode::InvalidNumber));
        };

        if pos_exp {
            res *= exp;
        } else {
            res /= exp;
        }

        if pos {
            visitor.visit_f64(res)
        } else {
            visitor.visit_f64(-res)
        }
    }

    fn decode_hex_escape(&mut self) -> Result<u16> {
        let mut i = 0;
        let mut n = 0u16;
        while i < 4 && !try!(self.eof()) {
            n = match try!(self.next_char_or_null()) {
                c @ b'0' ... b'9' => n * 16_u16 + ((c as u16) - (b'0' as u16)),
                b'a' | b'A' => n * 16_u16 + 10_u16,
                b'b' | b'B' => n * 16_u16 + 11_u16,
                b'c' | b'C' => n * 16_u16 + 12_u16,
                b'd' | b'D' => n * 16_u16 + 13_u16,
                b'e' | b'E' => n * 16_u16 + 14_u16,
                b'f' | b'F' => n * 16_u16 + 15_u16,
                _ => { return Err(self.error(ErrorCode::InvalidEscape)); }
            };

            i += 1;
        }

        // Error out if we didn't parse 4 digits.
        if i != 4 {
            return Err(self.error(ErrorCode::InvalidEscape));
        }

        Ok(n)
    }

    fn parse_string(&mut self) -> Result<()> {
        self.str_buf.clear();

        loop {
            let ch = match try!(self.next_char()) {
                Some(ch) => ch,
                None => { return Err(self.error(ErrorCode::EOFWhileParsingString)); }
            };

            match ch {
                b'"' => {
                    return Ok(());
                }
                b'\\' => {
                    let ch = match try!(self.next_char()) {
                        Some(ch) => ch,
                        None => { return Err(self.error(ErrorCode::EOFWhileParsingString)); }
                    };

                    match ch {
                        b'"' => self.str_buf.push(b'"'),
                        b'\\' => self.str_buf.push(b'\\'),
                        b'/' => self.str_buf.push(b'/'),
                        b'b' => self.str_buf.push(b'\x08'),
                        b'f' => self.str_buf.push(b'\x0c'),
                        b'n' => self.str_buf.push(b'\n'),
                        b'r' => self.str_buf.push(b'\r'),
                        b't' => self.str_buf.push(b'\t'),
                        b'u' => {
                            let c = match try!(self.decode_hex_escape()) {
                                0xDC00 ... 0xDFFF => {
                                    return Err(self.error(ErrorCode::LoneLeadingSurrogateInHexEscape));
                                }

                                // Non-BMP characters are encoded as a sequence of
                                // two hex escapes, representing UTF-16 surrogates.
                                n1 @ 0xD800 ... 0xDBFF => {
                                    match (try!(self.next_char()), try!(self.next_char())) {
                                        (Some(b'\\'), Some(b'u')) => (),
                                        _ => {
                                            return Err(self.error(ErrorCode::UnexpectedEndOfHexEscape));
                                        }
                                    }

                                    let n2 = try!(self.decode_hex_escape());

                                    if n2 < 0xDC00 || n2 > 0xDFFF {
                                        return Err(self.error(ErrorCode::LoneLeadingSurrogateInHexEscape));
                                    }

                                    let n = (((n1 - 0xD800) as u32) << 10 |
                                              (n2 - 0xDC00) as u32) + 0x1_0000;

                                    match char::from_u32(n as u32) {
                                        Some(c) => c,
                                        None => {
                                            return Err(self.error(ErrorCode::InvalidUnicodeCodePoint));
                                        }
                                    }
                                }

                                n => {
                                    match char::from_u32(n as u32) {
                                        Some(c) => c,
                                        None => {
                                            return Err(self.error(ErrorCode::InvalidUnicodeCodePoint));
                                        }
                                    }
                                }
                            };

                            // FIXME: this allocation is required in order to be compatible with stable
                            // rust, which doesn't support encoding a `char` into a stack buffer.
                            let buf = c.to_string();
                            self.str_buf.extend(buf.bytes());
                        }
                        _ => {
                            return Err(self.error(ErrorCode::InvalidEscape));
                        }
                    }
                }
                ch => {
                    self.str_buf.push(ch);
                }
            }
        }
    }

    fn parse_object_colon(&mut self) -> Result<()> {
        try!(self.parse_whitespace());

        match try!(self.next_char()) {
            Some(b':') => Ok(()),
            Some(_) => Err(self.error(ErrorCode::ExpectedColon)),
            None => Err(self.error(ErrorCode::EOFWhileParsingObject)),
        }
    }
}

impl<Iter> de::Deserializer for Deserializer<Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
{
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.parse_value(visitor)
    }

    /// Parses a `null` as a None, and any other values as a `Some(...)`.
    #[inline]
    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        try!(self.parse_whitespace());

        match try!(self.peek_or_null()) {
            b'n' => {
                self.eat_char();
                try!(self.parse_ident(b"ull"));
                visitor.visit_none()
            }
            _ => {
                visitor.visit_some(self)
            }
        }
    }

    /// Parses a newtype struct as the underlying value.
    #[inline]
    fn visit_newtype_struct<V>(&mut self,
                               _name: &str,
                               mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        visitor.visit_newtype_struct(self)
    }

    /// Parses an enum as an object like `{"$KEY":$VALUE}`, where $VALUE is either a straight
    /// value, a `[..]`, or a `{..}`.
    #[inline]
    fn visit_enum<V>(&mut self,
                     _name: &str,
                     _variants: &'static [&'static str],
                     mut visitor: V) -> Result<V::Value>
        where V: de::EnumVisitor,
    {
        try!(self.parse_whitespace());

        match try!(self.next_char_or_null()) {
            b'{' => {
                try!(self.parse_whitespace());

                let value = {
                    try!(visitor.visit(&mut *self))
                };

                try!(self.parse_whitespace());

                match try!(self.next_char_or_null()) {
                    b'}' => {
                        Ok(value)
                    }
                    _ => {
                        Err(self.error(ErrorCode::ExpectedSomeValue))
                    }
                }
            }
            _ => {
                Err(self.error(ErrorCode::ExpectedSomeValue))
            }
        }
    }

    #[inline]
    fn format() -> &'static str {
        "json"
    }
}

struct SeqVisitor<'a, Iter: 'a + Iterator<Item=io::Result<u8>>> {
    de: &'a mut Deserializer<Iter>,
    first: bool,
}

impl<'a, Iter: Iterator<Item=io::Result<u8>>> SeqVisitor<'a, Iter> {
    fn new(de: &'a mut Deserializer<Iter>) -> Self {
        SeqVisitor {
            de: de,
            first: true,
        }
    }
}

impl<'a, Iter> de::SeqVisitor for SeqVisitor<'a, Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
{
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: de::Deserialize,
    {
        try!(self.de.parse_whitespace());

        match try!(self.de.peek()) {
            Some(b']') => {
                return Ok(None);
            }
            Some(b',') if !self.first => {
                self.de.eat_char();
            }
            Some(_) => {
                if self.first {
                    self.first = false;
                } else {
                    return Err(self.de.error(ErrorCode::ExpectedListCommaOrEnd));
                }
            }
            None => {
                return Err(self.de.error(ErrorCode::EOFWhileParsingList));
            }
        }

        let value = try!(de::Deserialize::deserialize(self.de));
        Ok(Some(value))
    }

    fn end(&mut self) -> Result<()> {
        try!(self.de.parse_whitespace());

        match try!(self.de.next_char()) {
            Some(b']') => { Ok(()) }
            Some(_) => {
                Err(self.de.error(ErrorCode::TrailingCharacters))
            }
            None => {
                Err(self.de.error(ErrorCode::EOFWhileParsingList))
            }
        }
    }
}

struct MapVisitor<'a, Iter: 'a + Iterator<Item=io::Result<u8>>> {
    de: &'a mut Deserializer<Iter>,
    first: bool,
}

impl<'a, Iter: Iterator<Item=io::Result<u8>>> MapVisitor<'a, Iter> {
    fn new(de: &'a mut Deserializer<Iter>) -> Self {
        MapVisitor {
            de: de,
            first: true,
        }
    }
}

impl<'a, Iter> de::MapVisitor for MapVisitor<'a, Iter>
    where Iter: Iterator<Item=io::Result<u8>>
{
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: de::Deserialize,
    {
        try!(self.de.parse_whitespace());

        match try!(self.de.peek()) {
            Some(b'}') => {
                return Ok(None);
            }
            Some(b',') if !self.first => {
                self.de.eat_char();
                try!(self.de.parse_whitespace());
            }
            Some(_) => {
                if self.first {
                    self.first = false;
                } else {
                    return Err(self.de.error(ErrorCode::ExpectedObjectCommaOrEnd));
                }
            }
            None => {
                return Err(self.de.error(ErrorCode::EOFWhileParsingObject));
            }
        }

        match try!(self.de.peek()) {
            Some(b'"') => {
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            Some(_) => {
                Err(self.de.error(ErrorCode::KeyMustBeAString))
            }
            None => {
                Err(self.de.error(ErrorCode::EOFWhileParsingValue))
            }
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: de::Deserialize,
    {
        try!(self.de.parse_object_colon());

        Ok(try!(de::Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<()> {
        try!(self.de.parse_whitespace());

        match try!(self.de.next_char()) {
            Some(b'}') => { Ok(()) }
            Some(_) => {
                Err(self.de.error(ErrorCode::TrailingCharacters))
            }
            None => {
                Err(self.de.error(ErrorCode::EOFWhileParsingObject))
            }
        }
    }

    fn missing_field<V>(&mut self, _field: &'static str) -> Result<V>
        where V: de::Deserialize,
    {
        let mut de = de::value::ValueDeserializer::into_deserializer(());
        Ok(try!(de::Deserialize::deserialize(&mut de)))
    }
}

impl<Iter> de::VariantVisitor for Deserializer<Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
{
    type Error = Error;

    fn visit_variant<V>(&mut self) -> Result<V>
        where V: de::Deserialize
    {
        let val = try!(de::Deserialize::deserialize(self));
        try!(self.parse_object_colon());
        Ok(val)
    }

    fn visit_unit(&mut self) -> Result<()> {
        de::Deserialize::deserialize(self)
    }

    fn visit_newtype<T>(&mut self) -> Result<T>
        where T: de::Deserialize,
    {
        de::Deserialize::deserialize(self)
    }

    fn visit_tuple<V>(&mut self,
                      _len: usize,
                      visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::visit(self, visitor)
    }

    fn visit_struct<V>(&mut self,
                       _fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::visit(self, visitor)
    }
}

/// Decodes a json value from a `std::io::Read`.
pub fn from_iter<I, T>(iter: I) -> Result<T>
    where I: Iterator<Item=io::Result<u8>>,
          T: de::Deserialize,
{
    let mut de = Deserializer::new(iter);
    let value = try!(de::Deserialize::deserialize(&mut de));

    // Make sure the whole stream has been consumed.
    try!(de.end());
    Ok(value)
}

/// Decodes a json value from a `std::io::Read`.
pub fn from_reader<R, T>(rdr: R) -> Result<T>
    where R: io::Read,
          T: de::Deserialize,
{
    from_iter(rdr.bytes())
}

/// Decodes a json value from a `&str`.
pub fn from_slice<T>(v: &[u8]) -> Result<T>
    where T: de::Deserialize
{
    from_iter(v.iter().map(|byte| Ok(*byte)))
}

/// Decodes a json value from a `&str`.
pub fn from_str<T>(s: &str) -> Result<T>
    where T: de::Deserialize
{
    from_slice(s.as_bytes())
}
