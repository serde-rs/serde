use std::char;
use std::num::Float;
use std::str::ScalarValue;
use std::str;

use de;
use de::Deserializer;
use super::error::{Error, ErrorCode};

pub struct Parser<Iter> {
    rdr: Iter,
    ch: Option<u8>,
    line: uint,
    col: uint,
    buf: Vec<u8>,
}

impl<Iter: Iterator<u8>> Parser<Iter> {
    /// Creates the JSON parser.
    #[inline]
    pub fn new(rdr: Iter) -> Parser<Iter> {
        let mut p = Parser {
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
        R,
        V: de::Visitor<Parser<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
        self.parse_whitespace();

        if self.eof() {
            return Err(self.error(ErrorCode::EOFWhileParsingValue));
        }

        match self.ch_or_null() {
            b'n' => {
                try!(self.parse_ident(b"ull"));
                visitor.visit_null()
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
                //let s = String::from_utf8(self.buf.clone()).unwrap();
                let s = str::from_utf8(self.buf.as_slice()).unwrap();
                visitor.visit_str(s)
            }
            b'[' => {
                self.bump();
                visitor.visit_seq(SeqVisitor { parser: self, first: true })
            }
            b'{' => {
                self.bump();
                visitor.visit_map(MapVisitor { parser: self, first: true })
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
        R,
        V: de::Visitor<Parser<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
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
                    res += (((c as int) - (b'0' as int)) as f64) * dec;
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

        let mut exp = 0u;
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
                    exp += (c as uint) - (b'0' as uint);

                    self.bump();
                }
                _ => break
            }
        }

        let exp: f64 = 10_f64.powi(exp as i32);
        if neg_exp {
            res /= exp;
        } else {
            res *= exp;
        }

        Ok(res)
    }

    #[inline]
    fn decode_hex_escape(&mut self) -> Result<u16, Error> {
        let mut i = 0u;
        let mut n = 0u16;
        while i < 4u && !self.eof() {
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

            i += 1u;
        }

        // Error out if we didn't parse 4 digits.
        if i != 4u {
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
                                match str::utf16_items(buf.as_slice()).next() {
                                    Some(ScalarValue(c)) => c,
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

                        let buf = &mut [0u8, .. 4];
                        let len = c.encode_utf8(buf).unwrap_or(0);
                        self.buf.extend(buf.slice_to(len).iter().map(|b| *b));
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

impl<Iter: Iterator<u8>> Deserializer<Error> for Parser<Iter> {
    #[inline]
    fn visit<
        R,
        V: de::Visitor<Parser<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
        self.parse_value(visitor)
    }
}

struct SeqVisitor<'a, Iter: 'a> {
    parser: &'a mut Parser<Iter>,
    first: bool,
}

impl<'a, Iter: Iterator<u8>> de::SeqVisitor<Parser<Iter>, Error> for SeqVisitor<'a, Iter> {
    fn visit<
        T: de::Deserialize<Parser<Iter>, Error>,
    >(&mut self) -> Result<Option<T>, Error> {
        self.parser.parse_whitespace();

        if self.parser.ch_is(b']') {
            self.parser.bump();
            return Ok(None);
        }

        if self.first {
            self.first = false;
        } else {
            if self.parser.ch_is(b',') {
                self.parser.bump();
            } else if self.parser.eof() {
                return Err(self.parser.error(ErrorCode::EOFWhileParsingList));
            } else {
                return Err(self.parser.error(ErrorCode::ExpectedListCommaOrEnd));
            }
        }

        let value = try!(de::Deserialize::deserialize(self.parser));
        Ok(Some(value))
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.parser.ch_is(b']') {
            self.parser.bump();
            Ok(())
        } else if self.parser.eof() {
            Err(self.parser.error(ErrorCode::EOFWhileParsingList))
        } else {
            Err(self.parser.error(ErrorCode::TrailingCharacters))
        }
    }
}

struct MapVisitor<'a, Iter: 'a> {
    parser: &'a mut Parser<Iter>,
    first: bool,
}

impl<'a, Iter: Iterator<u8>> de::MapVisitor<Parser<Iter>, Error> for MapVisitor<'a, Iter> {
    fn visit_key<
        K: de::Deserialize<Parser<Iter>, Error>,
    >(&mut self) -> Result<Option<K>, Error> {
        self.parser.parse_whitespace();

        if self.parser.ch_is(b'}') {
            self.parser.bump();
            return Ok(None);
        }

        if self.first {
            self.first = false;
        } else {
            if self.parser.ch_is(b',') {
                self.parser.bump();
                self.parser.parse_whitespace();
            } else if self.parser.eof() {
                return Err(self.parser.error(ErrorCode::EOFWhileParsingObject));
            } else {
                return Err(self.parser.error(ErrorCode::ExpectedObjectCommaOrEnd));
            }
        }

        if self.parser.eof() {
            return Err(self.parser.error(ErrorCode::EOFWhileParsingValue));
        }

        if !self.parser.ch_is(b'"') {
            return Err(self.parser.error(ErrorCode::KeyMustBeAString));
        }

        Ok(Some(try!(de::Deserialize::deserialize(self.parser))))
    }

    fn visit_value<
        V: de::Deserialize<Parser<Iter>, Error>,
    >(&mut self) -> Result<V, Error> {
        self.parser.parse_whitespace();

        if self.parser.ch_is(b':') {
            self.parser.bump();
        } else if self.parser.eof() {
            return Err(self.parser.error(ErrorCode::EOFWhileParsingObject));
        } else {
            return Err(self.parser.error(ErrorCode::ExpectedColon));
        }

        self.parser.parse_whitespace();

        Ok(try!(de::Deserialize::deserialize(self.parser)))
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.parser.ch_is(b']') {
            self.parser.bump();
            Ok(())
        } else if self.parser.eof() {
            Err(self.parser.error(ErrorCode::EOFWhileParsingList))
        } else {
            Err(self.parser.error(ErrorCode::TrailingCharacters))
        }
    }
}

/// Decodes a json value from an `Iterator<u8>`.
pub fn from_iter<
    Iter: Iterator<u8>,
    T: de::Deserialize<Parser<Iter>, Error>
>(iter: Iter) -> Result<T, Error> {
    let mut parser = Parser::new(iter);
    let value = try!(de::Deserialize::deserialize(&mut parser));

    // Make sure the whole stream has been consumed.
    try!(parser.end());
    Ok(value)
}

/// Decodes a json value from a string
pub fn from_str<
    'a,
    T: de::Deserialize<Parser<str::Bytes<'a>>, Error>
>(s: &'a str) -> Result<T, Error> {
    from_iter(s.bytes())
}

#[cfg(test)]
mod tests {
    use std::str;
    use std::fmt::Show;
    use std::collections::TreeMap;

    use de::Deserialize;
    use super::{Parser, from_str};
    use super::super::error::{Error, ErrorCode};

    macro_rules! treemap {
        ($($k:expr => $v:expr),*) => ({
            let mut _m = TreeMap::new();
            $(_m.insert($k, $v);)*
            _m
        })
    }

    fn test_parse_ok<
        'a,
        T: PartialEq + Show + Deserialize<Parser<str::Bytes<'a>>, Error>,
    >(errors: Vec<(&'a str, T)>) {
        for (s, value) in errors.into_iter() {
            let v: Result<T, Error> = from_str(s);
            assert_eq!(v, Ok(value));

            /*
            let v: Json = from_iter(s.chars()).unwrap();
            assert_eq!(v, value.to_json());
            */
        }
    }

    fn test_parse_err<
        'a,
        T: PartialEq + Show + Deserialize<Parser<str::Bytes<'a>>, Error>
    >(errors: Vec<(&'a str, Error)>) {
        for (s, err) in errors.into_iter() {
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
            ("[3,1]", vec![3i, 1]),
            ("[ 3 , 1 ]", vec![3i, 1]),
        ]);

        test_parse_ok(vec![
            ("[[3], [1, 2]]", vec![vec![3i], vec![1, 2]]),
        ]);

        test_parse_ok(vec![
            ("[]", ()),
        ]);

        test_parse_ok(vec![
            ("[1]", (1u,)),
        ]);

        test_parse_ok(vec![
            ("[1, 2]", (1u, 2u)),
        ]);

        test_parse_ok(vec![
            ("[1, 2, 3]", (1u, 2u, 3u)),
        ]);
    }

    #[test]
    fn test_parse_object() {
        test_parse_err::<TreeMap<String, int>>(vec![
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
                treemap!("a".to_string() => 3i)
            ),
            (
                "{ \"a\" : 3 }",
                treemap!("a".to_string() => 3i)
            ),
            (
                "{\"a\":3,\"b\":4}",
                treemap!("a".to_string() => 3i, "b".to_string() => 4)
            ),
            (
                "{ \"a\" : 3 , \"b\" : 4 }",
                treemap!("a".to_string() => 3i, "b".to_string() => 4),
            ),
        ]);

        test_parse_ok(vec![
            (
                "{\"a\": {\"b\": 3, \"c\": 4}}",
                treemap!("a".to_string() => treemap!("b".to_string() => 3i, "c".to_string() => 4i)),
            ),
        ]);
    }
}
