use std::str;
use std::num::Float;
use unicode::str::Utf16Item;
use std::char;

use de;

use super::error::{Error, ErrorCode};

#[derive(PartialEq, Show)]
enum State {
    // Parse a value.
    Value,
    // Parse a value or ']'.
    ListStart,
    // Parse ',' or ']' after an element in a list.
    ListCommaOrEnd,
    // Parse a key:value or an ']'.
    ObjectStart,
    // Parse ',' or ']' after an element in an object.
    ObjectCommaOrEnd,
    // Parse a key in an object.
    //ObjectKey,
    // Parse a value in an object.
    ObjectValue,
}

/// A streaming JSON parser implemented as an iterator of JsonEvent, consuming
/// an iterator of char.
pub struct Parser<Iter> {
    rdr: Iter,
    ch: Option<u8>,
    line: usize,
    col: usize,
    // A state machine is kept to make it possible to interupt and resume parsing.
    state_stack: Vec<State>,
    buf: Vec<u8>,
}

impl<Iter: Iterator<Item=u8>> Iterator for Parser<Iter> {
    type Item = Result<de::Token, Error>;

    #[inline]
    fn next(&mut self) -> Option<Result<de::Token, Error>> {
        let state = match self.state_stack.pop() {
            Some(state) => state,
            None => {
                // If we have no state left, then we're expecting the structure
                // to be done, so make sure there are no trailing characters.

                self.parse_whitespace();

                if self.eof() {
                    return None;
                } else {
                    return Some(Err(self.error(ErrorCode::TrailingCharacters)));
                }
            }
        };

        match state {
            State::Value => Some(self.parse_value()),
            State::ListStart => Some(self.parse_list_start()),
            State::ListCommaOrEnd => Some(self.parse_list_comma_or_end()),
            State::ObjectStart => {
                match self.parse_object_start() {
                    Ok(Some(s)) => Some(Ok(de::Token::String(s.to_string()))),
                    Ok(None) => Some(Ok(de::Token::End)),
                    Err(err) => Some(Err(err)),
                }
            }
            State::ObjectCommaOrEnd => {
                match self.parse_object_comma_or_end() {
                    Ok(Some(s)) => Some(Ok(de::Token::String(s.to_string()))),
                    Ok(None) => Some(Ok(de::Token::End)),
                    Err(err) => Some(Err(err)),
                }
            }
            //State::ObjectKey => Some(self.parse_object_key()),
            State::ObjectValue => Some(self.parse_object_value()),
        }
    }
}

impl<Iter: Iterator<Item=u8>> Parser<Iter> {
    /// Creates the JSON parser.
    #[inline]
    pub fn new(rdr: Iter) -> Parser<Iter> {
        let mut p = Parser {
            rdr: rdr,
            ch: Some(b'\x00'),
            line: 1,
            col: 0,
            state_stack: vec!(State::Value),
            buf: Vec::with_capacity(128),
        };
        p.bump();
        return p;
    }

    #[inline(always)]
    fn eof(&self) -> bool { self.ch.is_none() }

    #[inline]
    fn ch_or_null(&self) -> u8 { self.ch.unwrap_or(b'\x00') }

    #[inline(always)]
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

    #[inline(always)]
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
    fn parse_value(&mut self) -> Result<de::Token, Error> {
        self.parse_whitespace();

        if self.eof() {
            return Err(self.error(ErrorCode::EOFWhileParsingValue));
        }

        match self.ch_or_null() {
            b'n' => self.parse_ident(b"ull", de::Token::Null),
            b't' => self.parse_ident(b"rue", de::Token::Bool(true)),
            b'f' => self.parse_ident(b"alse", de::Token::Bool(false)),
            b'0' ... b'9' | b'-' => self.parse_number(),
            b'"' => {
                Ok(de::Token::String(try!(self.parse_string()).to_string()))
            }
            b'[' => {
                self.bump();
                self.state_stack.push(State::ListStart);
                Ok(de::Token::SeqStart(0))
            }
            b'{' => {
                self.bump();
                self.state_stack.push(State::ObjectStart);
                Ok(de::Token::MapStart(0))
            }
            _ => {
                Err(self.error(ErrorCode::ExpectedSomeValue))
            }
        }
    }

    #[inline]
    fn parse_ident(&mut self, ident: &[u8], token: de::Token) -> Result<de::Token, Error> {
        if ident.iter().all(|c| Some(*c) == self.next_char()) {
            self.bump();
            Ok(token)
        } else {
            Err(self.error(ErrorCode::ExpectedSomeIdent))
        }
    }

    #[inline]
    fn parse_number(&mut self) -> Result<de::Token, Error> {
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

            Ok(de::Token::F64(neg * res))
        } else {
            Ok(de::Token::I64(neg * res))
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
                    res += (((c as isize) - (b'0' as isize)) as f64) * dec;
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

        let mut exp = 0us;
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
                    exp += (c as usize) - (b'0' as usize);

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
        let mut i = 0us;
        let mut n = 0u16;
        while i < 4us && !self.eof() {
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

            i += 1us;
        }

        // Error out if we didn't parse 4 digits.
        if i != 4us {
            return Err(self.error(ErrorCode::InvalidEscape));
        }

        Ok(n)
    }

    #[inline]
    fn parse_string(&mut self) -> Result<&str, Error> {
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

                        let buf = &mut [0u8; 4];
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
                        return Ok(str::from_utf8(&self.buf).unwrap());
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

    #[inline]
    fn parse_list_start(&mut self) -> Result<de::Token, Error> {
        self.parse_whitespace();

        if self.ch_is(b']') {
            self.bump();
            Ok(de::Token::End)
        } else {
            self.state_stack.push(State::ListCommaOrEnd);
            self.parse_value()
        }
    }

    #[inline]
    fn parse_list_comma_or_end(&mut self) -> Result<de::Token, Error> {
        self.parse_whitespace();

        if self.ch_is(b',') {
            self.bump();
            self.state_stack.push(State::ListCommaOrEnd);
            self.parse_value()
        } else if self.ch_is(b']') {
            self.bump();
            Ok(de::Token::End)
        } else if self.eof() {
            Err(self.error(ErrorCode::EOFWhileParsingList))
        } else {
            Err(self.error(ErrorCode::ExpectedListCommaOrEnd))
        }
    }

    #[inline]
    fn parse_object_start(&mut self) -> Result<Option<&str>, Error> {
        self.parse_whitespace();

        if self.ch_is(b'}') {
            self.bump();
            Ok(None)
        } else {
            Ok(Some(try!(self.parse_object_key())))
        }
    }

    #[inline]
    fn parse_object_comma_or_end(&mut self) -> Result<Option<&str>, Error> {
        self.parse_whitespace();

        if self.ch_is(b',') {
            self.bump();
            Ok(Some(try!(self.parse_object_key())))
        } else if self.ch_is(b'}') {
            self.bump();
            Ok(None)
        } else if self.eof() {
            Err(self.error(ErrorCode::EOFWhileParsingObject))
        } else {
            Err(self.error(ErrorCode::ExpectedObjectCommaOrEnd))
        }
    }

    #[inline]
    fn parse_object_key(&mut self) -> Result<&str, Error> {
        self.parse_whitespace();

        if self.eof() {
            return Err(self.error(ErrorCode::EOFWhileParsingString));
        }

        match self.ch_or_null() {
            b'"' => {
                self.state_stack.push(State::ObjectValue);

                Ok(try!(self.parse_string()))
            }
            _ => Err(self.error(ErrorCode::KeyMustBeAString)),
        }
    }

    #[inline]
    fn parse_object_value(&mut self) -> Result<de::Token, Error> {
        self.parse_whitespace();

        if self.ch_is(b':') {
            self.bump();
            self.state_stack.push(State::ObjectCommaOrEnd);
            self.parse_value()
        } else if self.eof() {
            Err(self.error(ErrorCode::EOFWhileParsingObject))
        } else {
            Err(self.error(ErrorCode::ExpectedColon))
        }
    }
}

impl<Iter: Iterator<Item=u8>> de::Deserializer<Error> for Parser<Iter> {
    fn end_of_stream_error(&mut self) -> Error {
        Error::SyntaxError(ErrorCode::EOFWhileParsingValue, self.line, self.col)
    }

    fn syntax_error(&mut self, token: de::Token, expected: &'static [de::TokenKind]) -> Error {
        Error::SyntaxError(ErrorCode::ExpectedTokens(token, expected), self.line, self.col)
    }

    fn unexpected_name_error(&mut self, token: de::Token) -> Error {
        Error::SyntaxError(ErrorCode::UnexpectedName(token), self.line, self.col)
    }

    fn conversion_error(&mut self, token: de::Token) -> Error {
        Error::SyntaxError(ErrorCode::ConversionError(token), self.line, self.col)
    }

    #[inline]
    fn missing_field<
        T: de::Deserialize<Parser<Iter>, Error>
    >(&mut self, _field: &'static str) -> Result<T, Error> {
        // JSON can represent `null` values as a missing value, so this isn't
        // necessarily an error.
        de::Deserialize::deserialize_token(self, de::Token::Null)
    }

    // Special case treating options as a nullable value.
    #[inline]
    fn expect_option<
        U: de::Deserialize<Parser<Iter>, Error>
    >(&mut self, token: de::Token) -> Result<Option<U>, Error> {
        match token {
            de::Token::Null => Ok(None),
            token => {
                let value: U = try!(de::Deserialize::deserialize_token(self, token));
                Ok(Some(value))
            }
        }
    }

    // Special case treating enums as a `{"<variant-name>": [<fields>]}`.
    #[inline]
    fn expect_enum_start(&mut self,
                         token: de::Token,
                         _name: &str,
                         variants: &[&str]) -> Result<usize, Error> {
        match token {
            de::Token::MapStart(_) => { }
            _ => { return Err(self.error(ErrorCode::ExpectedEnumMapStart)); }
        };

        // Enums only have one field in them, which is the variant name.
        let variant = match try!(self.expect_token()) {
            de::Token::String(variant) => variant,
            _ => { return Err(self.error(ErrorCode::ExpectedEnumVariantString)); }
        };

        // The variant's field is a list of the values.
        match try!(self.expect_token()) {
            de::Token::SeqStart(_) => { }
            _ => { return Err(self.error(ErrorCode::ExpectedEnumToken)); }
        }

        match variants.iter().position(|v| *v == &variant[]) {
            Some(idx) => Ok(idx),
            None => Err(self.error(ErrorCode::UnknownVariant)),
        }
    }

    fn expect_enum_end(&mut self) -> Result<(), Error> {
        // There will be one `End` for the list, and one for the object.
        match try!(self.expect_token()) {
            de::Token::End => {
                match try!(self.expect_token()) {
                    de::Token::End => Ok(()),
                    _ => Err(self.error(ErrorCode::ExpectedEnumEndToken)),
                }
            }
            _ => Err(self.error(ErrorCode::ExpectedEnumEnd)),
        }
    }

    #[inline]
    fn expect_struct_start(&mut self, token: de::Token, _name: &str) -> Result<(), Error> {
        match token {
            de::Token::MapStart(_) => Ok(()),
            _ => {
                static EXPECTED_TOKENS: &'static [de::TokenKind] = &[
                    de::TokenKind::MapStartKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_struct_field_or_end(&mut self,
                                  fields: &'static [&'static str]
                                 ) -> Result<Option<Option<usize>>, Error> {
        let result = match self.state_stack.pop() {
            Some(State::ObjectStart) => {
                try!(self.parse_object_start())
            }
            Some(State::ObjectCommaOrEnd) => {
                try!(self.parse_object_comma_or_end())
            }
            _ => panic!("invalid internal state"),
        };

        let s = match result {
            Some(s) => s,
            None => { return Ok(None); }
        };

        Ok(Some(fields.iter().position(|field| *field == &s[])))
    }
}

/// Decodes a json value from an `Iterator<u8>`.
pub fn from_iter<
    Iter: Iterator<Item=u8>,
    T: de::Deserialize<Parser<Iter>, Error>
>(iter: Iter) -> Result<T, Error> {
    let mut parser = Parser::new(iter);
    let value = try!(de::Deserialize::deserialize(&mut parser));

    // Make sure the whole stream has been consumed.
    match parser.next() {
        Some(Ok(_token)) => Err(parser.error(ErrorCode::TrailingCharacters)),
        Some(Err(err)) => Err(err),
        None => Ok(value),
    }
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
}
