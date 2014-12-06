use std::char;
use std::fmt;
use std::num::Float;
use std::str::ScalarValue;
use std::str;

use de;
use de::Deserializer;

#[deriving(Clone, PartialEq, Eq)]
pub enum ErrorCode {
    EOFWhileParsingList,
    EOFWhileParsingObject,
    EOFWhileParsingString,
    EOFWhileParsingValue,
    ExpectedColon,
    InvalidEscape,
    InvalidNumber,
    InvalidSyntax(SyntaxExpectation),
    InvalidUnicodeCodePoint,
    KeyMustBeAString,
    LoneLeadingSurrogateInHexEscape,
    MissingField(&'static str),
    NotFourDigit,
    NotUtf8,
    TrailingCharacters,
    UnexpectedEndOfHexEscape,
    UnknownVariant,
    UnrecognizedHex,
}

/// The failed expectation of InvalidSyntax
#[deriving(Clone, PartialEq, Eq, Show)]
pub enum SyntaxExpectation {
    ListCommaOrEnd,
    ObjectCommaOrEnd,
    SomeValue,
    SomeIdent,
    EnumMapStart,
    EnumVariantString,
    EnumToken,
    EnumEndToken,
    EnumEnd,
}

impl fmt::Show for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::EOFWhileParsingList => "EOF While parsing list".fmt(f),
            ErrorCode::EOFWhileParsingObject => "EOF While parsing object".fmt(f),
            ErrorCode::EOFWhileParsingString => "EOF While parsing string".fmt(f),
            ErrorCode::EOFWhileParsingValue => "EOF While parsing value".fmt(f),
            ErrorCode::ExpectedColon => "expected `:`".fmt(f),
            ErrorCode::InvalidEscape => "invalid escape".fmt(f),
            ErrorCode::InvalidNumber => "invalid number".fmt(f),
            ErrorCode::InvalidSyntax(expect) => {
                write!(f, "invalid syntax, expected: {}", expect)
            }
            ErrorCode::InvalidUnicodeCodePoint => "invalid unicode code point".fmt(f),
            ErrorCode::KeyMustBeAString => "key must be a string".fmt(f),
            ErrorCode::LoneLeadingSurrogateInHexEscape => "lone leading surrogate in hex escape".fmt(f),
            ErrorCode::MissingField(field) => {
                write!(f, "missing field \"{}\"", field)
            }
            ErrorCode::NotFourDigit => "invalid \\u escape (not four digits)".fmt(f),
            ErrorCode::NotUtf8 => "contents not utf-8".fmt(f),
            ErrorCode::TrailingCharacters => "trailing characters".fmt(f),
            ErrorCode::UnexpectedEndOfHexEscape => "unexpected end of hex escape".fmt(f),
            ErrorCode::UnknownVariant => "unknown variant".fmt(f),
            ErrorCode::UnrecognizedHex => "invalid \\u escape (unrecognized hex)".fmt(f),
        }
    }
}

#[deriving(PartialEq, Eq, Show)]
pub enum Error {
    SyntaxError(ErrorCode, uint, uint),
}

pub struct Parser<Iter> {
    rdr: Iter,
    ch: Option<char>,
    line: uint,
    col: uint,
}

impl<
    Iter: Iterator<char>,
> Parser<Iter> {
    /// Creates the JSON parser.
    pub fn new(rdr: Iter) -> Parser<Iter> {
        let mut p = Parser {
            rdr: rdr,
            ch: Some('\x00'),
            line: 1,
            col: 0,
        };
        p.bump();
        return p;
    }

    pub fn end(&mut self) -> Result<(), Error> {
        if self.eof() {
            Ok(())
        } else {
            Err(self.error(ErrorCode::TrailingCharacters))
        }
    }

    fn eof(&self) -> bool { self.ch.is_none() }

    fn ch_or_null(&self) -> char { self.ch.unwrap_or('\x00') }

    fn bump(&mut self) {
        self.ch = self.rdr.next();

        if self.ch_is('\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.bump();
        self.ch
    }

    fn ch_is(&self, c: char) -> bool {
        self.ch == Some(c)
    }

    fn parse_whitespace(&mut self) {
        while self.ch_is(' ') ||
              self.ch_is('\n') ||
              self.ch_is('\t') ||
              self.ch_is('\r') { self.bump(); }
    }

    fn error(&mut self, reason: ErrorCode) -> Error {
        //self.state_stack.clear();
        Error::SyntaxError(reason, self.line, self.col)
    }

    fn parse_value<
        R,
        V: de::Visitor<Parser<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
        self.parse_whitespace();

        if self.eof() {
            return Err(self.error(ErrorCode::EOFWhileParsingValue));
        }

        match self.ch_or_null() {
            'n' => {
                try!(self.parse_ident("ull"));
                visitor.visit_null(self)
            }
            't' => {
                try!(self.parse_ident("rue"));
                visitor.visit_bool(self, true)
            }
            'f' => {
                try!(self.parse_ident("alse"));
                visitor.visit_bool(self, false)
            }
            '0' ... '9' | '-' => self.parse_number(visitor),
            '"' => {
                let s = try!(self.parse_string());
                visitor.visit_string(self, s)
            }
            '[' => {
                self.bump();
                visitor.visit_seq(self, SeqVisitor { first: true })
            }
            '{' => {
                self.bump();
                visitor.visit_map(self, MapVisitor { first: true })
            }
            _ => {
                Err(self.error(ErrorCode::InvalidSyntax(SyntaxExpectation::SomeValue)))
            }
        }
    }

    fn parse_ident(&mut self, ident: &str) -> Result<(), Error> {
        if ident.chars().all(|c| Some(c) == self.next_char()) {
            self.bump();
            Ok(())
        } else {
            Err(self.error(ErrorCode::InvalidSyntax(SyntaxExpectation::SomeIdent)))
        }
    }

    fn parse_number<
        R,
        V: de::Visitor<Parser<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
        let mut neg = 1;

        if self.ch_is('-') {
            self.bump();
            neg = -1;
        }

        let res = try!(self.parse_integer());

        if self.ch_is('.') || self.ch_is('e') || self.ch_is('E') {
            let neg = neg as f64;
            let mut res = res as f64;

            if self.ch_is('.') {
                res = try!(self.parse_decimal(res));
            }

            if self.ch_is('e') || self.ch_is('E') {
                res = try!(self.parse_exponent(res));
            }

            visitor.visit_f64(self, neg * res)
        } else {
            visitor.visit_i64(self, neg * res)
        }
    }

    fn parse_integer(&mut self) -> Result<i64, Error> {
        let mut res = 0;

        match self.ch_or_null() {
            '0' => {
                self.bump();

                // There can be only one leading '0'.
                match self.ch_or_null() {
                    '0' ... '9' => {
                        return Err(self.error(ErrorCode::InvalidNumber));
                    }
                    _ => ()
                }
            },
            '1' ... '9' => {
                while !self.eof() {
                    match self.ch_or_null() {
                        c @ '0' ... '9' => {
                            res *= 10;
                            res += (c as i64) - ('0' as i64);
                            self.bump();
                        }
                        _ => break,
                    }
                }
            }
            _ => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
        }

        Ok(res)
    }

    fn parse_decimal(&mut self, res: f64) -> Result<f64, Error> {
        self.bump();

        // Make sure a digit follows the decimal place.
        match self.ch_or_null() {
            '0' ... '9' => (),
             _ => {
                 return Err(self.error(ErrorCode::InvalidNumber));
             }
        }

        let mut res = res;
        let mut dec = 1.0;
        while !self.eof() {
            match self.ch_or_null() {
                c @ '0' ... '9' => {
                    dec /= 10.0;
                    res += (((c as int) - ('0' as int)) as f64) * dec;
                    self.bump();
                }
                _ => break,
            }
        }

        Ok(res)
    }

    fn parse_exponent(&mut self, mut res: f64) -> Result<f64, Error> {
        self.bump();

        let mut exp = 0u;
        let mut neg_exp = false;

        if self.ch_is('+') {
            self.bump();
        } else if self.ch_is('-') {
            self.bump();
            neg_exp = true;
        }

        // Make sure a digit follows the exponent place.
        match self.ch_or_null() {
            '0' ... '9' => (),
            _ => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
        }
        while !self.eof() {
            match self.ch_or_null() {
                c @ '0' ... '9' => {
                    exp *= 10;
                    exp += (c as uint) - ('0' as uint);

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

    fn decode_hex_escape(&mut self) -> Result<u16, Error> {
        let mut i = 0u;
        let mut n = 0u16;
        while i < 4u && !self.eof() {
            self.bump();
            n = match self.ch_or_null() {
                c @ '0' ... '9' => n * 16_u16 + ((c as u16) - ('0' as u16)),
                'a' | 'A' => n * 16_u16 + 10_u16,
                'b' | 'B' => n * 16_u16 + 11_u16,
                'c' | 'C' => n * 16_u16 + 12_u16,
                'd' | 'D' => n * 16_u16 + 13_u16,
                'e' | 'E' => n * 16_u16 + 14_u16,
                'f' | 'F' => n * 16_u16 + 15_u16,
                _ => {
                    return Err(self.error(ErrorCode::InvalidEscape));
                }
            };

            i += 1u;
        }

        // Error out if we didn't parse 4 digits.
        if i != 4u {
            return Err(self.error(ErrorCode::InvalidEscape));
        }

        Ok(n)
    }

    fn parse_string(&mut self) -> Result<String, Error> {
        let mut escape = false;
        let mut res = String::new();

        loop {
            self.bump();
            if self.eof() {
                return Err(self.error(ErrorCode::EOFWhileParsingString));
            }

            if escape {
                match self.ch_or_null() {
                    '"' => res.push('"'),
                    '\\' => res.push('\\'),
                    '/' => res.push('/'),
                    'b' => res.push('\x08'),
                    'f' => res.push('\x0c'),
                    'n' => res.push('\n'),
                    'r' => res.push('\r'),
                    't' => res.push('\t'),
                    'u' => match try!(self.decode_hex_escape()) {
                        0xDC00 ... 0xDFFF => {
                            return Err(self.error(ErrorCode::LoneLeadingSurrogateInHexEscape));
                        }

                        // Non-BMP characters are encoded as a sequence of
                        // two hex escapes, representing UTF-16 surrogates.
                        n1 @ 0xD800 ... 0xDBFF => {
                            let c1 = self.next_char();
                            let c2 = self.next_char();
                            match (c1, c2) {
                                (Some('\\'), Some('u')) => (),
                                _ => {
                                    return Err(self.error(ErrorCode::UnexpectedEndOfHexEscape));
                                }
                            }

                            let buf = [n1, try!(self.decode_hex_escape())];
                            match str::utf16_items(buf.as_slice()).next() {
                                Some(ScalarValue(c)) => res.push(c),
                                _ => {
                                    return Err(self.error(ErrorCode::LoneLeadingSurrogateInHexEscape));
                                }
                            }
                        }

                        n => match char::from_u32(n as u32) {
                            Some(c) => res.push(c),
                            None => {
                                return Err(self.error(ErrorCode::InvalidUnicodeCodePoint));
                            }
                        },
                    },
                    _ => {
                        return Err(self.error(ErrorCode::InvalidEscape));
                    }
                }
                escape = false;
            } else if self.ch_is('\\') {
                escape = true;
            } else {
                match self.ch {
                    Some('"') => {
                        self.bump();
                        return Ok(res);
                    },
                    Some(c) => res.push(c),
                    None => unreachable!()
                }
            }
        }
    }
}

impl<Iter: Iterator<char>> Deserializer<Error> for Parser<Iter> {
    #[inline]
    fn visit<
        R,
        V: de::Visitor<Parser<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
        self.parse_value(visitor)
    }

    fn syntax_error(&mut self) -> Error {
        Error::SyntaxError(ErrorCode::InvalidSyntax(SyntaxExpectation::SomeValue), self.line, self.col)
    }

    fn end_of_stream_error(&mut self) -> Error {
        Error::SyntaxError(ErrorCode::EOFWhileParsingValue, self.line, self.col)
    }
}

struct SeqVisitor {
    first: bool,
}

impl<Iter: Iterator<char>> de::SeqVisitor<Parser<Iter>, Error> for SeqVisitor {
    fn visit<
        T: de::Deserialize<Parser<Iter>, Error>,
    >(&mut self, d: &mut Parser<Iter>) -> Result<Option<T>, Error> {
        d.parse_whitespace();

        if d.ch_is(']') {
            d.bump();
            return Ok(None);
        }

        if self.first {
            self.first = false;
        } else {
            if d.ch_is(',') {
                d.bump();
            } else if d.eof() {
                return Err(d.error(ErrorCode::EOFWhileParsingList));
            } else {
                return Err(d.error(ErrorCode::InvalidSyntax(SyntaxExpectation::ListCommaOrEnd)));
            }
        }

        let value = try!(de::Deserialize::deserialize(d));
        Ok(Some(value))
    }

    fn end(&mut self, d: &mut Parser<Iter>) -> Result<(), Error> {
        if d.ch_is(']') {
            d.bump();
            Ok(())
        } else if d.eof() {
            Err(d.error(ErrorCode::EOFWhileParsingList))
        } else {
            Err(d.error(ErrorCode::TrailingCharacters))
        }
    }
}

struct MapVisitor {
    first: bool,
}

impl<Iter: Iterator<char>> de::MapVisitor<Parser<Iter>, Error> for MapVisitor {
    fn visit<
        K: de::Deserialize<Parser<Iter>, Error>,
        V: de::Deserialize<Parser<Iter>, Error>,
    >(&mut self, d: &mut Parser<Iter>) -> Result<Option<(K, V)>, Error> {
        d.parse_whitespace();

        if d.ch_is('}') {
            d.bump();
            return Ok(None);
        }

        if self.first {
            self.first = false;
        } else {
            if d.ch_is(',') {
                d.bump();
                d.parse_whitespace();
            } else if d.eof() {
                return Err(d.error(ErrorCode::EOFWhileParsingObject));
            } else {
                return Err(d.error(ErrorCode::InvalidSyntax(SyntaxExpectation::ObjectCommaOrEnd)));
            }
        }

        if d.eof() {
            return Err(d.error(ErrorCode::EOFWhileParsingValue));
        }

        if !d.ch_is('"') {
            return Err(d.error(ErrorCode::KeyMustBeAString));
        }

        let key = try!(de::Deserialize::deserialize(d));

        d.parse_whitespace();

        if d.ch_is(':') {
            d.bump();
        } else if d.eof() {
            return Err(d.error(ErrorCode::EOFWhileParsingObject));
        } else {
            return Err(d.error(ErrorCode::ExpectedColon));
        }

        d.parse_whitespace();

        let value = try!(de::Deserialize::deserialize(d));

        Ok(Some((key, value)))
    }

    fn end(&mut self, d: &mut Parser<Iter>) -> Result<(), Error> {
        if d.ch_is(']') {
            d.bump();
            Ok(())
        } else if d.eof() {
            Err(d.error(ErrorCode::EOFWhileParsingList))
        } else {
            Err(d.error(ErrorCode::TrailingCharacters))
        }
    }
}

/// Decodes a json value from an `Iterator<Char>`.
pub fn from_iter<
    Iter: Iterator<char>,
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
    T: de::Deserialize<Parser<str::Chars<'a>>, Error>
>(s: &'a str) -> Result<T, Error> {
    from_iter(s.chars())
}


#[cfg(test)]
mod tests {
    use std::str;
    use std::fmt::Show;
    use std::collections::TreeMap;

    use de::Deserialize;
    use super::{Parser, Error, ErrorCode, SyntaxExpectation, from_str};

    macro_rules! treemap {
        ($($k:expr => $v:expr),*) => ({
            let mut _m = TreeMap::new();
            $(_m.insert($k, $v);)*
            _m
        })
    }

    fn test_parse_ok<
        'a,
        T: PartialEq + Show + Deserialize<Parser<str::Chars<'a>>, Error>,
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
        T: PartialEq + Show + Deserialize<Parser<str::Chars<'a>>, Error>
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
            ("t", Error::SyntaxError(ErrorCode::InvalidSyntax(SyntaxExpectation::SomeIdent), 1, 2)),
            ("truz", Error::SyntaxError(ErrorCode::InvalidSyntax(SyntaxExpectation::SomeIdent), 1, 4)),
            ("f", Error::SyntaxError(ErrorCode::InvalidSyntax(SyntaxExpectation::SomeIdent), 1, 2)),
            ("faz", Error::SyntaxError(ErrorCode::InvalidSyntax(SyntaxExpectation::SomeIdent), 1, 3)),
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
            ("+", Error::SyntaxError(ErrorCode::InvalidSyntax(SyntaxExpectation::SomeValue), 1, 1)),
            (".", Error::SyntaxError(ErrorCode::InvalidSyntax(SyntaxExpectation::SomeValue), 1, 1)),
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
            ("\"\\u12ab\"", "\u12ab".to_string()),
            ("\"\\uAB12\"", "\uAB12".to_string()),
        ]);
    }

    #[test]
    fn test_parse_list() {
        test_parse_err::<Vec<f64>>(vec![
            ("[", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 2)),
            ("[ ", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 3)),
            ("[1", Error::SyntaxError(ErrorCode::EOFWhileParsingList,  1, 3)),
            ("[1,", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 4)),
            ("[1,]", Error::SyntaxError(ErrorCode::InvalidSyntax(SyntaxExpectation::SomeValue), 1, 4)),
            ("[1 2]", Error::SyntaxError(ErrorCode::InvalidSyntax(SyntaxExpectation::ListCommaOrEnd), 1, 4)),
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
            ("{\"a\":1 1", Error::SyntaxError(ErrorCode::InvalidSyntax(SyntaxExpectation::ObjectCommaOrEnd), 1, 8)),
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
