use std::str;
use std::num;
use std::str::ScalarValue;
use std::char;

use de;

use super::{Error, ErrorCode};
use super::{
    ConversionError,
    EOFWhileParsingList,
    EOFWhileParsingObject,
    EOFWhileParsingString,
    EOFWhileParsingValue,
    ExpectedColon,
    ExpectedEnumEnd,
    ExpectedEnumEndToken,
    ExpectedEnumMapStart,
    ExpectedEnumToken,
    ExpectedEnumVariantString,
    ExpectedListCommaOrEnd,
    ExpectedObjectCommaOrEnd,
    ExpectedSomeIdent,
    ExpectedSomeValue,
    ExpectedTokens,
    InvalidEscape,
    InvalidNumber,
    InvalidUnicodeCodePoint,
    KeyMustBeAString,
    LoneLeadingSurrogateInHexEscape,
    TrailingCharacters,
    UnexpectedEndOfHexEscape,
    UnexpectedName,
    UnknownVariant,
};
use super::{
    SyntaxError,
};

#[deriving(PartialEq, Show)]
enum ParserState {
    // Parse a value.
    ParseValue,
    // Parse a value or ']'.
    ParseListStart,
    // Parse ',' or ']' after an element in a list.
    ParseListCommaOrEnd,
    // Parse a key:value or an ']'.
    ParseObjectStart,
    // Parse ',' or ']' after an element in an object.
    ParseObjectCommaOrEnd,
    // Parse a key in an object.
    //ParseObjectKey,
    // Parse a value in an object.
    ParseObjectValue,
}

/// A streaming JSON parser implemented as an iterator of JsonEvent, consuming
/// an iterator of char.
pub struct Parser<Iter> {
    rdr: Iter,
    ch: Option<u8>,
    line: uint,
    col: uint,
    // A state machine is kept to make it possible to interupt and resume parsing.
    state_stack: Vec<ParserState>,
    buf: Vec<u8>,
}

impl<Iter: Iterator<u8>> Iterator<Result<de::Token, Error>> for Parser<Iter> {
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
                    return Some(self.error(TrailingCharacters));
                }
            }
        };

        match state {
            ParseValue => Some(self.parse_value()),
            ParseListStart => Some(self.parse_list_start()),
            ParseListCommaOrEnd => Some(self.parse_list_comma_or_end()),
            ParseObjectStart => {
                match self.parse_object_start() {
                    Ok(Some(s)) => Some(Ok(de::String(s.to_string()))),
                    Ok(None) => Some(Ok(de::End)),
                    Err(err) => Some(Err(err)),
                }
            }
            ParseObjectCommaOrEnd => {
                match self.parse_object_comma_or_end() {
                    Ok(Some(s)) => Some(Ok(de::String(s.to_string()))),
                    Ok(None) => Some(Ok(de::End)),
                    Err(err) => Some(Err(err)),
                }
            }
            //ParseObjectKey => Some(self.parse_object_key()),
            ParseObjectValue => Some(self.parse_object_value()),
        }
    }
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
            state_stack: vec!(ParseValue),
            buf: Vec::with_capacity(100),
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
    fn error<T>(&self, reason: ErrorCode) -> Result<T, Error> {
        Err(SyntaxError(reason, self.line, self.col))
    }

    #[inline]
    fn parse_whitespace(&mut self) {
        while self.ch_is(b' ') ||
              self.ch_is(b'\n') ||
              self.ch_is(b'\t') ||
              self.ch_is(b'\r') { self.bump(); }
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

            Ok(de::F64(neg * res))
        } else {
            Ok(de::I64(neg * res))
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
                    b'0' ... b'9' => return self.error(InvalidNumber),
                    _ => ()
                }
            },
            b'1' ... b'9' => {
                while !self.eof() {
                    match self.ch_or_null() {
                        c @ b'0' ... b'9' => {
                            res *= 10;
                            res += (c as i64) - ('0' as i64);
                            self.bump();
                        }
                        _ => break,
                    }
                }
            }
            _ => return self.error(InvalidNumber),
        }

        Ok(res)
    }

    #[inline]
    fn parse_decimal(&mut self, res: f64) -> Result<f64, Error> {
        self.bump();

        // Make sure a digit follows the decimal place.
        match self.ch_or_null() {
            b'0' ... b'9' => (),
             _ => return self.error(InvalidNumber)
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
            _ => return self.error(InvalidNumber)
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

        let exp: f64 = num::pow(10u as f64, exp);
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
                _ => return self.error(InvalidEscape)
            };

            i += 1u;
        }

        // Error out if we didn't parse 4 digits.
        if i != 4u {
            return self.error(InvalidEscape);
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
                None => { return self.error(EOFWhileParsingString); }
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
                            0xDC00 ... 0xDFFF => return self.error(LoneLeadingSurrogateInHexEscape),

                            // Non-BMP characters are encoded as a sequence of
                            // two hex escapes, representing UTF-16 surrogates.
                            n1 @ 0xD800 ... 0xDBFF => {
                                let c1 = self.next_char();
                                let c2 = self.next_char();
                                match (c1, c2) {
                                    (Some(b'\\'), Some(b'u')) => (),
                                    _ => return self.error(UnexpectedEndOfHexEscape),
                                }

                                let buf = [n1, try!(self.decode_hex_escape())];
                                match str::utf16_items(buf.as_slice()).next() {
                                    Some(ScalarValue(c)) => c,
                                    _ => return self.error(LoneLeadingSurrogateInHexEscape),
                                }
                            }

                            n => match char::from_u32(n as u32) {
                                Some(c) => c,
                                None => return self.error(InvalidUnicodeCodePoint),
                            }
                        };

                        let mut buf = [0u8, .. 4];
                        let len = c.encode_utf8(buf).unwrap_or(0);
                        self.buf.extend(buf.slice_to(len).iter().map(|b| *b));
                    }
                    _ => return self.error(InvalidEscape),
                }
                escape = false;
            } else {
                match ch {
                    b'"' => {
                        self.bump();
                        return Ok(str::from_utf8(self.buf.as_slice()).unwrap());
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
            Ok(de::End)
        } else {
            self.state_stack.push(ParseListCommaOrEnd);
            self.parse_value()
        }
    }

    #[inline]
    fn parse_list_comma_or_end(&mut self) -> Result<de::Token, Error> {
        self.parse_whitespace();

        if self.ch_is(b',') {
            self.bump();
            self.state_stack.push(ParseListCommaOrEnd);
            self.parse_value()
        } else if self.ch_is(b']') {
            self.bump();
            Ok(de::End)
        } else if self.eof() {
            self.error_event(EOFWhileParsingList)
        } else {
            self.error_event(ExpectedListCommaOrEnd)
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
            self.error_event(EOFWhileParsingObject)
        } else {
            self.error_event(ExpectedObjectCommaOrEnd)
        }
    }

    #[inline]
    fn parse_object_key(&mut self) -> Result<&str, Error> {
        self.parse_whitespace();

        if self.eof() {
            return self.error_event(EOFWhileParsingString);
        }

        match self.ch_or_null() {
            b'"' => {
                self.state_stack.push(ParseObjectValue);

                Ok(try!(self.parse_string()))
            }
            _ => self.error_event(KeyMustBeAString),
        }
    }

    #[inline]
    fn parse_object_value(&mut self) -> Result<de::Token, Error> {
        self.parse_whitespace();

        if self.ch_is(b':') {
            self.bump();
            self.state_stack.push(ParseObjectCommaOrEnd);
            self.parse_value()
        } else if self.eof() {
            self.error_event(EOFWhileParsingObject)
        } else {
            self.error_event(ExpectedColon)
        }
    }

    #[inline]
    fn parse_value(&mut self) -> Result<de::Token, Error> {
        self.parse_whitespace();

        if self.eof() {
            return self.error_event(EOFWhileParsingValue);
        }

        match self.ch_or_null() {
            b'n' => self.parse_ident(b"ull", de::Null),
            b't' => self.parse_ident(b"rue", de::Bool(true)),
            b'f' => self.parse_ident(b"alse", de::Bool(false)),
            b'0' ... b'9' | b'-' => self.parse_number(),
            b'"' => {
                Ok(de::String(try!(self.parse_string()).to_string()))
            }
            b'[' => {
                self.bump();
                self.state_stack.push(ParseListStart);
                Ok(de::SeqStart(0))
            }
            b'{' => {
                self.bump();
                self.state_stack.push(ParseObjectStart);
                Ok(de::MapStart(0))
            }
            _ => {
                self.error_event(ExpectedSomeValue)
            }
        }
    }

    #[inline]
    fn parse_ident(&mut self, ident: &[u8], token: de::Token) -> Result<de::Token, Error> {
        if ident.iter().all(|c| Some(*c) == self.next_char()) {
            self.bump();
            Ok(token)
        } else {
            self.error_event(ExpectedSomeIdent)
        }
    }

    #[inline]
    fn error_event<T>(&mut self, reason: ErrorCode) -> Result<T, Error> {
        self.state_stack.clear();
        Err(SyntaxError(reason, self.line, self.col))
    }
}

impl<Iter: Iterator<u8>> de::Deserializer<Error> for Parser<Iter> {
    fn end_of_stream_error(&mut self) -> Error {
        SyntaxError(EOFWhileParsingValue, self.line, self.col)
    }

    fn syntax_error(&mut self, token: de::Token, expected: &'static [de::TokenKind]) -> Error {
        SyntaxError(ExpectedTokens(token, expected), self.line, self.col)
    }

    fn unexpected_name_error(&mut self, token: de::Token) -> Error {
        SyntaxError(UnexpectedName(token), self.line, self.col)
    }

    fn conversion_error(&mut self, token: de::Token) -> Error {
        SyntaxError(ConversionError(token), self.line, self.col)
    }

    #[inline]
    fn missing_field<
        T: de::Deserialize<Parser<Iter>, Error>
    >(&mut self, _field: &'static str) -> Result<T, Error> {
        // JSON can represent `null` values as a missing value, so this isn't
        // necessarily an error.
        de::Deserialize::deserialize_token(self, de::Null)
    }

    // Special case treating options as a nullable value.
    #[inline]
    fn expect_option<
        U: de::Deserialize<Parser<Iter>, Error>
    >(&mut self, token: de::Token) -> Result<Option<U>, Error> {
        match token {
            de::Null => Ok(None),
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
                         variants: &[&str]) -> Result<uint, Error> {
        match token {
            de::MapStart(_) => { }
            _ => { return self.error(ExpectedEnumMapStart); }
        };

        // Enums only have one field in them, which is the variant name.
        let variant = match try!(self.expect_token()) {
            de::String(variant) => variant,
            _ => { return self.error(ExpectedEnumVariantString); }
        };

        // The variant's field is a list of the values.
        match try!(self.expect_token()) {
            de::SeqStart(_) => { }
            _ => { return self.error(ExpectedEnumToken); }
        }

        match variants.iter().position(|v| *v == variant.as_slice()) {
            Some(idx) => Ok(idx),
            None => self.error(UnknownVariant),
        }
    }

    fn expect_enum_end(&mut self) -> Result<(), Error> {
        // There will be one `End` for the list, and one for the object.
        match try!(self.expect_token()) {
            de::End => {
                match try!(self.expect_token()) {
                    de::End => Ok(()),
                    _ => self.error(ExpectedEnumEndToken),
                }
            }
            _ => self.error(ExpectedEnumEnd),
        }
    }

    #[inline]
    fn expect_struct_start(&mut self, token: de::Token, _name: &str) -> Result<(), Error> {
        match token {
            de::MapStart(_) => Ok(()),
            _ => {
                static EXPECTED_TOKENS: &'static [de::TokenKind] = [
                    de::MapStartKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_struct_field_or_end(&mut self,
                                  fields: &'static [&'static str]
                                 ) -> Result<Option<Option<uint>>, Error> {
        let result = match self.state_stack.pop() {
            Some(ParseObjectStart) => {
                try!(self.parse_object_start())
            }
            Some(ParseObjectCommaOrEnd) => {
                try!(self.parse_object_comma_or_end())
            }
            _ => panic!("invalid internal state"),
        };

        let s = match result {
            Some(s) => s,
            None => { return Ok(None); }
        };

        Ok(Some(fields.iter().position(|field| *field == s.as_slice())))
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
    match parser.next() {
        Some(Ok(_token)) => parser.error(TrailingCharacters),
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
