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

impl<Iter: Iterator<Item=u8>> Deserializer<Iter> {
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

    #[inline]
    fn ch_or_null(&self) -> u8 { self.ch.unwrap_or(b'\x00') }

    #[inline]
    fn bump(&mut self) {
        self.ch = self.rdr.next();

        if self.ch_is(b'\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }

    #[inline]
    fn next_char(&mut self) -> Option<u8> {
        self.bump();
        self.ch
    }

    #[inline]
    fn ch_is(&self, c: u8) -> bool {
        self.ch == Some(c)
    }

    #[inline]
    fn error(&mut self, reason: ErrorCode) -> Error {
        Error::SyntaxError(reason, self.line, self.col)
    }

    #[inline]
    fn parse_whitespace(&mut self) {
        while self.ch_is(b' ') ||
              self.ch_is(b'\n') ||
              self.ch_is(b'\t') ||
              self.ch_is(b'\r') { self.bump(); }
    }

    #[inline]
    fn parse_value<
        V: de::Visitor,
    >(&mut self, mut visitor: V) -> Result<V::Value, Error> {
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
                visitor.visit_seq(SeqVisitor {
                    de: self,
                    first: true,
                })
            }
            b'{' => {
                self.bump();
                visitor.visit_map(MapVisitor {
                    de: self,
                    first: true,
                })
            }
            _ => {
                Err(self.error(ErrorCode::ExpectedSomeValue))
            }
        }
    }

    #[inline]
    fn parse_ident(&mut self, ident: &[u8]) -> Result<(), Error> {
        if ident.iter().all(|c| Some(*c) == self.next_char()) {
            self.bump();
            Ok(())
        } else {
            Err(self.error(ErrorCode::ExpectedSomeIdent))
        }
    }

    #[inline]
    fn parse_number<
        V: de::Visitor,
    >(&mut self, mut visitor: V) -> Result<V::Value, Error> {
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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
}

impl<Iter: Iterator<Item=u8>> de::Deserializer for Deserializer<Iter> {
    type Error = Error;

    #[inline]
    fn visit<
        V: de::Visitor,
    >(&mut self, visitor: V) -> Result<V::Value, Error> {
        self.parse_value(visitor)
    }
}

struct SeqVisitor<'a, Iter: 'a> {
    de: &'a mut Deserializer<Iter>,
    first: bool,
}

impl<'a, Iter: Iterator<Item=u8>> de::SeqVisitor for SeqVisitor<'a, Iter> {
    type Error = Error;

    fn visit<
        T: de::Deserialize,
    >(&mut self) -> Result<Option<T>, Error> {
        self.de.parse_whitespace();

        if self.de.ch_is(b']') {
            self.de.bump();
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

impl<'a, Iter: Iterator<Item=u8>> de::MapVisitor for MapVisitor<'a, Iter> {
    type Error = Error;

    fn visit_key<
        K: de::Deserialize,
    >(&mut self) -> Result<Option<K>, Error> {
        self.de.parse_whitespace();

        if self.de.ch_is(b'}') {
            self.de.bump();
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
            return Err(self.de.error(ErrorCode::EOFWhileParsingValue));
        }

        if !self.de.ch_is(b'"') {
            return Err(self.de.error(ErrorCode::KeyMustBeAString));
        }

        Ok(Some(try!(de::Deserialize::deserialize(self.de))))
    }

    fn visit_value<
        V: de::Deserialize,
    >(&mut self) -> Result<V, Error> {
        self.de.parse_whitespace();

        if self.de.ch_is(b':') {
            self.de.bump();
        } else if self.de.eof() {
            return Err(self.de.error(ErrorCode::EOFWhileParsingObject));
        } else {
            return Err(self.de.error(ErrorCode::ExpectedColon));
        }

        self.de.parse_whitespace();

        Ok(try!(de::Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<(), Error> {
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

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::collections::BTreeMap;

    use de::Deserialize;
    use super::from_str;
    use super::super::error::{Error, ErrorCode};

    macro_rules! treemap {
        ($($k:expr => $v:expr),*) => ({
            let mut _m = BTreeMap::new();
            $(_m.insert($k, $v);)*
            _m
        })
    }

    fn test_parse_ok<'a, T>(errors: Vec<(&'a str, T)>)
        where T: PartialEq + Debug + Deserialize,
    {
        for (s, value) in errors {
            let v: Result<T, Error> = from_str(s);
            assert_eq!(v, Ok(value));

            /*
            let v: Json = from_iter(s.chars()).unwrap();
            assert_eq!(v, value.to_json());
            */
        }
    }

    fn test_parse_err<'a, T>(errors: Vec<(&'a str, Error)>)
        where T: PartialEq + Debug + Deserialize
    {
        for (s, err) in errors {
            let v: Result<T, Error> = from_str(s);
            assert_eq!(v, Err(err));
        }
    }

    #[test]
    fn test_parse_null() {
        test_parse_ok(vec![
            ("null", ()),
        ]);
    }

    #[test]
    fn test_parse_bool() {
        test_parse_err::<bool>(vec![
            ("t", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 2)),
            ("truz", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 4)),
            ("f", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 2)),
            ("faz", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 3)),
            ("truea", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 5)),
            ("falsea", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 6)),
        ]);

        test_parse_ok(vec![
            ("true", true),
            ("false", false),
        ]);
    }

    #[test]
    fn test_parse_numbers() {
        test_parse_err::<f64>(vec![
            ("+", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 1)),
            (".", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 1)),
            ("-", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 2)),
            ("00", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 2)),
            ("1.", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 3)),
            ("1e", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 3)),
            ("1e+", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 4)),
            ("1a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 2)),
        ]);

        test_parse_ok(vec![
            ("3", 3i64),
            ("-2", -2),
            ("-1234", -1234),
        ]);

        test_parse_ok(vec![
            ("3.0", 3.0f64),
            ("3.1", 3.1),
            ("-1.2", -1.2),
            ("0.4", 0.4),
            ("0.4e5", 0.4e5),
            ("0.4e15", 0.4e15),
            ("0.4e-01", 0.4e-01),
        ]);
    }

    #[test]
    fn test_parse_string() {
        test_parse_err::<String>(vec![
            ("\"", Error::SyntaxError(ErrorCode::EOFWhileParsingString, 1, 2)),
            ("\"lol", Error::SyntaxError(ErrorCode::EOFWhileParsingString, 1, 5)),
            ("\"lol\"a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 6)),
        ]);

        test_parse_ok(vec![
            ("\"\"", "".to_string()),
            ("\"foo\"", "foo".to_string()),
            ("\"\\\"\"", "\"".to_string()),
            ("\"\\b\"", "\x08".to_string()),
            ("\"\\n\"", "\n".to_string()),
            ("\"\\r\"", "\r".to_string()),
            ("\"\\t\"", "\t".to_string()),
            ("\"\\u12ab\"", "\u{12ab}".to_string()),
            ("\"\\uAB12\"", "\u{AB12}".to_string()),
        ]);
    }

    #[test]
    fn test_parse_list() {
        test_parse_err::<Vec<f64>>(vec![
            ("[", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 2)),
            ("[ ", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 3)),
            ("[1", Error::SyntaxError(ErrorCode::EOFWhileParsingList,  1, 3)),
            ("[1,", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 4)),
            ("[1,]", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 4)),
            ("[1 2]", Error::SyntaxError(ErrorCode::ExpectedListCommaOrEnd, 1, 4)),
            ("[]a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 3)),
        ]);

        test_parse_ok(vec![
            ("[]", vec![]),
            ("[ ]", vec![]),
            ("[null]", vec![()]),
            ("[ null ]", vec![()]),
        ]);

        test_parse_ok(vec![
            ("[true]", vec![true]),
        ]);

        test_parse_ok(vec![
            ("[3,1]", vec![3, 1]),
            ("[ 3 , 1 ]", vec![3, 1]),
        ]);

        test_parse_ok(vec![
            ("[[3], [1, 2]]", vec![vec![3], vec![1, 2]]),
        ]);

        test_parse_ok(vec![
            ("[]", ()),
        ]);

        test_parse_ok(vec![
            ("[1]", (1,)),
        ]);

        test_parse_ok(vec![
            ("[1, 2]", (1, 2)),
        ]);

        test_parse_ok(vec![
            ("[1, 2, 3]", (1, 2, 3)),
        ]);
    }

    #[test]
    fn test_parse_object() {
        test_parse_err::<BTreeMap<String, i32>>(vec![
            ("{", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 2)),
            ("{ ", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 3)),
            ("{1", Error::SyntaxError(ErrorCode::KeyMustBeAString, 1, 2)),
            ("{ \"a\"", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 6)),
            ("{\"a\"", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 5)),
            ("{\"a\" ", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 6)),
            ("{\"a\" 1", Error::SyntaxError(ErrorCode::ExpectedColon, 1, 6)),
            ("{\"a\":", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 6)),
            ("{\"a\":1", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 7)),
            ("{\"a\":1 1", Error::SyntaxError(ErrorCode::ExpectedObjectCommaOrEnd, 1, 8)),
            ("{\"a\":1,", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 8)),
            ("{}a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 3)),
        ]);

        test_parse_ok(vec![
            ("{}", treemap!()),
            ("{ }", treemap!()),
            (
                "{\"a\":3}",
                treemap!("a".to_string() => 3)
            ),
            (
                "{ \"a\" : 3 }",
                treemap!("a".to_string() => 3)
            ),
            (
                "{\"a\":3,\"b\":4}",
                treemap!("a".to_string() => 3, "b".to_string() => 4)
            ),
            (
                "{ \"a\" : 3 , \"b\" : 4 }",
                treemap!("a".to_string() => 3, "b".to_string() => 4),
            ),
        ]);

        test_parse_ok(vec![
            (
                "{\"a\": {\"b\": 3, \"c\": 4}}",
                treemap!("a".to_string() => treemap!("b".to_string() => 3, "c".to_string() => 4)),
            ),
        ]);
    }

    #[test]
    fn test_parse_trailing_whitespace() {
        test_parse_ok(vec![
            ("[1, 2] ", vec![1, 2]),
            ("[1, 2]\n", vec![1, 2]),
            ("[1, 2]\t", vec![1, 2]),
            ("[1, 2]\t \n", vec![1, 2]),
        ]);
    }
}
