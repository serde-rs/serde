use std::char;
use std::num::Float;
use unicode::str::Utf16Item;
use std::str;

use de;
use super::error::{Error, ErrorCode};

pub struct Deserializer<Iter> {
    rdr: Iter,
    ch: Option<u8>,
    line: usize,
    col: usize,
    buf: Vec<u8>,
}

impl<Iter> Deserializer<Iter>
    where Iter: Iterator<Item=u8>,
{
    /// Creates the JSON parser.
    #[inline]
    pub fn new(rdr: Iter) -> Deserializer<Iter> {
        let mut p = Deserializer {
            rdr: rdr,
            ch: Some(b'\x00'),
            line: 1,
            col: 0,
            buf: Vec::with_capacity(128),
        };
        p.bump();
        return p;
    }

    #[inline]
    pub fn end(&mut self) -> Result<(), Error> {
        self.parse_whitespace();
        if self.eof() {
            Ok(())
        } else {
            Err(self.error(ErrorCode::TrailingCharacters))
        }
    }

    fn eof(&self) -> bool { self.ch.is_none() }

    fn ch_or_null(&self) -> u8 { self.ch.unwrap_or(b'\x00') }

    fn bump(&mut self) {
        self.ch = self.rdr.next();

        if self.ch_is(b'\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }

    fn next_char(&mut self) -> Option<u8> {
        self.bump();
        self.ch
    }

    fn ch_is(&self, c: u8) -> bool {
        self.ch == Some(c)
    }

    fn error(&mut self, reason: ErrorCode) -> Error {
        Error::SyntaxError(reason, self.line, self.col)
    }

    fn parse_whitespace(&mut self) {
        while self.ch_is(b' ') ||
              self.ch_is(b'\n') ||
              self.ch_is(b'\t') ||
              self.ch_is(b'\r') { self.bump(); }
    }

    fn parse_value<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        self.parse_whitespace();

        if self.eof() {
            return Err(self.error(ErrorCode::EOFWhileParsingValue));
        }

        match self.ch_or_null() {
            b'n' => {
                try!(self.parse_ident(b"ull"));
                visitor.visit_unit()
            }
            b't' => {
                try!(self.parse_ident(b"rue"));
                visitor.visit_bool(true)
            }
            b'f' => {
                try!(self.parse_ident(b"alse"));
                visitor.visit_bool(false)
            }
            b'0' ... b'9' | b'-' => self.parse_number(visitor),
            b'"' => {
                try!(self.parse_string());
                let s = str::from_utf8(&self.buf).unwrap();
                visitor.visit_str(s)
            }
            b'[' => {
                self.bump();
                visitor.visit_seq(SeqVisitor::new(self))
            }
            b'{' => {
                self.bump();
                visitor.visit_map(MapVisitor::new(self))
            }
            _ => {
                Err(self.error(ErrorCode::ExpectedSomeValue))
            }
        }
    }

    fn parse_ident(&mut self, ident: &[u8]) -> Result<(), Error> {
        if ident.iter().all(|c| Some(*c) == self.next_char()) {
            self.bump();
            Ok(())
        } else {
            Err(self.error(ErrorCode::ExpectedSomeIdent))
        }
    }

    fn parse_number<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        let mut neg = 1;

        if self.ch_is(b'-') {
            self.bump();
            neg = -1;
        }

        let res = try!(self.parse_integer());

        if self.ch_is(b'.') || self.ch_is(b'e') || self.ch_is(b'E') {
            let neg = neg as f64;
            let mut res = res as f64;

            if self.ch_is(b'.') {
                res = try!(self.parse_decimal(res));
            }

            if self.ch_is(b'e') || self.ch_is(b'E') {
                res = try!(self.parse_exponent(res));
            }

            visitor.visit_f64(neg * res)
        } else {
            visitor.visit_i64(neg * res)
        }
    }

    fn parse_integer(&mut self) -> Result<i64, Error> {
        let mut res = 0;

        match self.ch_or_null() {
            b'0' => {
                self.bump();

                // There can be only one leading '0'.
                match self.ch_or_null() {
                    b'0' ... b'9' => {
                        return Err(self.error(ErrorCode::InvalidNumber));
                    }
                    _ => ()
                }
            },
            b'1' ... b'9' => {
                while !self.eof() {
                    match self.ch_or_null() {
                        c @ b'0' ... b'9' => {
                            res *= 10;
                            res += (c as i64) - (b'0' as i64);
                            self.bump();
                        }
                        _ => break,
                    }
                }
            }
            _ => { return Err(self.error(ErrorCode::InvalidNumber)); }
        }

        Ok(res)
    }

    fn parse_decimal(&mut self, res: f64) -> Result<f64, Error> {
        self.bump();

        // Make sure a digit follows the decimal place.
        match self.ch_or_null() {
            b'0' ... b'9' => (),
             _ => { return Err(self.error(ErrorCode::InvalidNumber)); }
        }

        let mut res = res;
        let mut dec = 1.0;
        while !self.eof() {
            match self.ch_or_null() {
                c @ b'0' ... b'9' => {
                    dec /= 10.0;
                    res += (((c as u64) - (b'0' as u64)) as f64) * dec;
                    self.bump();
                }
                _ => break,
            }
        }

        Ok(res)
    }

    fn parse_exponent(&mut self, mut res: f64) -> Result<f64, Error> {
        self.bump();

        let mut exp = 0;
        let mut neg_exp = false;

        if self.ch_is(b'+') {
            self.bump();
        } else if self.ch_is(b'-') {
            self.bump();
            neg_exp = true;
        }

        // Make sure a digit follows the exponent place.
        match self.ch_or_null() {
            b'0' ... b'9' => (),
            _ => { return Err(self.error(ErrorCode::InvalidNumber)); }
        }
        while !self.eof() {
            match self.ch_or_null() {
                c @ b'0' ... b'9' => {
                    exp *= 10;
                    exp += (c as i32) - (b'0' as i32);

                    self.bump();
                }
                _ => break
            }
        }

        let exp: f64 = 10_f64.powi(exp);
        if neg_exp {
            res /= exp;
        } else {
            res *= exp;
        }

        Ok(res)
    }

    fn decode_hex_escape(&mut self) -> Result<u16, Error> {
        let mut i = 0;
        let mut n = 0u16;
        while i < 4 && !self.eof() {
            self.bump();
            n = match self.ch_or_null() {
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

    fn parse_string(&mut self) -> Result<(), Error> {
        self.buf.clear();

        let mut escape = false;

        loop {
            let ch = match self.next_char() {
                Some(ch) => ch,
                None => { return Err(self.error(ErrorCode::EOFWhileParsingString)); }
            };

            if escape {
                match ch {
                    b'"' => self.buf.push(b'"'),
                    b'\\' => self.buf.push(b'\\'),
                    b'/' => self.buf.push(b'/'),
                    b'b' => self.buf.push(b'\x08'),
                    b'f' => self.buf.push(b'\x0c'),
                    b'n' => self.buf.push(b'\n'),
                    b'r' => self.buf.push(b'\r'),
                    b't' => self.buf.push(b'\t'),
                    b'u' => {
                        let c = match try!(self.decode_hex_escape()) {
                            0xDC00 ... 0xDFFF => {
                                return Err(self.error(ErrorCode::LoneLeadingSurrogateInHexEscape));
                            }

                            // Non-BMP characters are encoded as a sequence of
                            // two hex escapes, representing UTF-16 surrogates.
                            n1 @ 0xD800 ... 0xDBFF => {
                                let c1 = self.next_char();
                                let c2 = self.next_char();
                                match (c1, c2) {
                                    (Some(b'\\'), Some(b'u')) => (),
                                    _ => {
                                        return Err(self.error(ErrorCode::UnexpectedEndOfHexEscape));
                                    }
                                }

                                let buf = &[n1, try!(self.decode_hex_escape())];
                                match ::unicode::str::utf16_items(buf).next() {
                                    Some(Utf16Item::ScalarValue(c)) => c,
                                    _ => {
                                        return Err(self.error(ErrorCode::LoneLeadingSurrogateInHexEscape));
                                    }
                                }
                            }

                            n => match char::from_u32(n as u32) {
                                Some(c) => c,
                                None => {
                                    return Err(self.error(ErrorCode::InvalidUnicodeCodePoint));
                                }
                            }
                        };

                        let buf = &mut [0; 4];
                        let len = c.encode_utf8(buf).unwrap_or(0);
                        self.buf.extend(buf[..len].iter().map(|b| *b));
                    }
                    _ => {
                        return Err(self.error(ErrorCode::InvalidEscape));
                    }
                }
                escape = false;
            } else {
                match ch {
                    b'"' => {
                        self.bump();
                        return Ok(());
                    }
                    b'\\' => {
                        escape = true;
                    }
                    ch => {
                        self.buf.push(ch);
                    }
                }
            }
        }
    }

    fn parse_object_colon(&mut self) -> Result<(), Error> {
        self.parse_whitespace();

        if self.ch_is(b':') {
            self.bump();
            Ok(())
        } else if self.eof() {
            Err(self.error(ErrorCode::EOFWhileParsingObject))
        } else {
            Err(self.error(ErrorCode::ExpectedColon))
        }
    }
}

impl<Iter> de::Deserializer for Deserializer<Iter>
    where Iter: Iterator<Item=u8>,
{
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        self.parse_value(visitor)
    }

    #[inline]
    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        self.parse_whitespace();

        if self.eof() {
            return Err(self.error(ErrorCode::EOFWhileParsingValue));
        }

        if self.ch_is(b'n') {
            try!(self.parse_ident(b"ull"));
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    #[inline]
    fn visit_enum<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        self.parse_whitespace();

        if self.ch_is(b'{') {
            self.bump();
            self.parse_whitespace();

            try!(self.parse_string());
            try!(self.parse_object_colon());

            let variant = str::from_utf8(&self.buf).unwrap().to_string();

            let value = try!(visitor.visit_variant(&variant, EnumVisitor {
                de: self,
            }));

            self.parse_whitespace();

            if self.ch_is(b'}') {
                self.bump();
                Ok(value)
            } else {
                return Err(self.error(ErrorCode::ExpectedSomeValue));
            }
        } else {
            Err(self.error(ErrorCode::ExpectedSomeValue))
        }
    }
}

struct SeqVisitor<'a, Iter: 'a> {
    de: &'a mut Deserializer<Iter>,
    first: bool,
}

impl<'a, Iter> SeqVisitor<'a, Iter> {
    fn new(de: &'a mut Deserializer<Iter>) -> Self {
        SeqVisitor {
            de: de,
            first: true,
        }
    }
}

impl<'a, Iter> de::SeqVisitor for SeqVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>
{
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        self.de.parse_whitespace();

        if self.de.ch_is(b']') {
            return Ok(None);
        }

        if self.first {
            self.first = false;
        } else {
            if self.de.ch_is(b',') {
                self.de.bump();
            } else if self.de.eof() {
                return Err(self.de.error(ErrorCode::EOFWhileParsingList));
            } else {
                return Err(self.de.error(ErrorCode::ExpectedListCommaOrEnd));
            }
        }

        let value = try!(de::Deserialize::deserialize(self.de));
        Ok(Some(value))
    }

    fn end(&mut self) -> Result<(), Error> {
        self.de.parse_whitespace();

        if self.de.ch_is(b']') {
            self.de.bump();
            Ok(())
        } else if self.de.eof() {
            Err(self.de.error(ErrorCode::EOFWhileParsingList))
        } else {
            Err(self.de.error(ErrorCode::TrailingCharacters))
        }
    }
}

struct MapVisitor<'a, Iter: 'a> {
    de: &'a mut Deserializer<Iter>,
    first: bool,
}

impl<'a, Iter> MapVisitor<'a, Iter> {
    fn new(de: &'a mut Deserializer<Iter>) -> Self {
        MapVisitor {
            de: de,
            first: true,
        }
    }
}

impl<'a, Iter> de::MapVisitor for MapVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>
{
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: de::Deserialize,
    {
        self.de.parse_whitespace();

        if self.de.ch_is(b'}') {
            return Ok(None);
        }

        if self.first {
            self.first = false;
        } else {
            if self.de.ch_is(b',') {
                self.de.bump();
                self.de.parse_whitespace();
            } else if self.de.eof() {
                return Err(self.de.error(ErrorCode::EOFWhileParsingObject));
            } else {
                return Err(self.de.error(ErrorCode::ExpectedObjectCommaOrEnd));
            }
        }

        if self.de.eof() {
            println!("here3");
            return Err(self.de.error(ErrorCode::EOFWhileParsingValue));
        }

        if !self.de.ch_is(b'"') {
            return Err(self.de.error(ErrorCode::KeyMustBeAString));
        }

        Ok(Some(try!(de::Deserialize::deserialize(self.de))))
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        try!(self.de.parse_object_colon());

        Ok(try!(de::Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<(), Error> {
        self.de.parse_whitespace();

        if self.de.ch_is(b'}') {
            self.de.bump();
            Ok(())
        } else if self.de.eof() {
            Err(self.de.error(ErrorCode::EOFWhileParsingObject))
        } else {
            Err(self.de.error(ErrorCode::TrailingCharacters))
        }
    }

    fn missing_field<V>(&mut self, _field: &'static str) -> Result<V, Error>
        where V: de::Deserialize,
    {
        // See if the type can deserialize from a unit.
        struct UnitDeserializer;

        impl de::Deserializer for UnitDeserializer {
            type Error = Error;

            fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
                where V: de::Visitor,
            {
                visitor.visit_unit()
            }

            fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
                where V: de::Visitor,
            {
                visitor.visit_none()
            }
        }

        Ok(try!(de::Deserialize::deserialize(&mut UnitDeserializer)))
    }
}

struct EnumVisitor<'a, Iter: 'a> {
    de: &'a mut Deserializer<Iter>,
}

impl<'a, Iter> de::EnumVisitor for EnumVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>,
{
    type Error = Error;

    fn visit_unit(&mut self) -> Result<(), Error> {
        de::Deserialize::deserialize(self.de)
    }

    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumSeqVisitor,
    {
        self.de.parse_whitespace();

        if self.de.ch_is(b'[') {
            self.de.bump();
            visitor.visit(SeqVisitor::new(self.de))
        } else {
            Err(self.de.error(ErrorCode::ExpectedSomeValue))
        }
    }

    fn visit_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumMapVisitor,
    {
        self.de.parse_whitespace();

        if self.de.ch_is(b'{') {
            self.de.bump();
            visitor.visit(MapVisitor::new(self.de))
        } else {
            Err(self.de.error(ErrorCode::ExpectedSomeValue))
        }
    }
}

/// Decodes a json value from an `Iterator<u8>`.
pub fn from_iter<I, T>(iter: I) -> Result<T, Error>
    where I: Iterator<Item=u8>,
          T: de::Deserialize
{
    let mut de = Deserializer::new(iter);
    let value = try!(de::Deserialize::deserialize(&mut de));

    // Make sure the whole stream has been consumed.
    try!(de.end());
    Ok(value)
}

/// Decodes a json value from a string
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
    where T: de::Deserialize
{
    from_iter(s.bytes())
}
