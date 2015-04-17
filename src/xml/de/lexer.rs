#![deny(unused_must_use)]
use xml::error::*;
use self::LexerError::*;

use std::iter::Peekable;

use std::{str, char};

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Lexical<'a> {
    StartTagClose,

    Text(&'a [u8]),

    StartTagName(&'a [u8]),

    AttributeName(&'a [u8]),
    AttributeValue(&'a [u8]),

    EmptyElementEnd(&'a [u8]),

    EndTagName(&'a [u8]),
    StartOfFile,
    EndOfFile,
}

#[derive(PartialEq)]
enum InternalLexical {
    StartTagClose,

    Text,

    StartTagName,

    AttributeName,
    AttributeValue,

    EmptyElementEnd,

    EndTagName,
    StartOfFile,
    EndOfFile,
}

enum LexerState {
    Start,
    AttributeName,
    AttributeValue,
    Tag,
}

struct LineColIterator<Iter: Iterator<Item=u8>> {
    rdr: Peekable<Iter>,
    line: usize,
    col: usize,
}

impl<Iter: Iterator<Item=u8>> LineColIterator<Iter> {
    fn peek(&mut self) -> Option<u8> {
        self.rdr.peek().map(|&c| c)
    }
}

impl<Iter: Iterator<Item=u8>> Iterator for LineColIterator<Iter> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        match self.rdr.next() {
            None => None,
            Some(b'\n') => {
                self.line += 1;
                self.col = 1;
                print!(" -> \\n");
                Some(b'\n')
            },
            Some(c) => {
                print!(" -> {:?}", c as char);
                self.col += 1;
                Some(c)
            },
        }
    }
}

pub struct XmlIterator<Iter: Iterator<Item=u8>> {
    rdr: LineColIterator<Iter>,
    buf: Vec<u8>,
    stash: Vec<u8>,
    state: LexerState,
    ch: InternalLexical,
}

impl<Iter> XmlIterator<Iter>
    where Iter: Iterator<Item=u8>,
{

    pub fn expected(&self, reason: &'static str) -> Error {
        self.error(ErrorCode::Expected(reason))
    }

    pub fn error(&self, reason: ErrorCode) -> Error {
        Error::SyntaxError(reason, self.rdr.line, self.rdr.col)
    }

    fn lexer_error(&self, reason: LexerError) -> Error {
        self.error(ErrorCode::LexingError(reason))
    }

    pub fn from_utf8<'a>(&self, txt: &'a[u8]) -> Result<&'a str, Error> {
        let txt = str::from_utf8(txt);
        txt.or(Err(self.error(ErrorCode::NotUtf8)))
    }

    #[inline]
    pub fn new(rdr: Iter) -> XmlIterator<Iter> {
        XmlIterator {
            rdr: LineColIterator {
                rdr: rdr.peekable(),
                line: 1,
                col: 1,
            },
            buf: Vec::with_capacity(128),
            stash: Vec::new(),
            state: LexerState::Start,
            ch: InternalLexical::StartOfFile,
        }
    }

    pub fn stash(&mut self) {
        use std::mem::swap;
        swap(&mut self.buf, &mut self.stash);
    }

    pub fn stash_view(&self) -> &[u8] {
        &self.stash
    }

    fn peek_char(&mut self) -> Result<u8, LexerError> {
        self.rdr.peek().ok_or(LexerError::EOF)
    }

    fn next_char(&mut self) -> Result<u8, LexerError> {
        self.rdr.next().ok_or(LexerError::EOF)
    }

    fn decode(&mut self) -> Result<InternalLexical, LexerError> {
        use self::LexerState::*;
        if let FirstAttribute = self.state {
            // for empty elements since they don't have a closing tag
            // but we need the tag name to determine sequences
            self.stash();
        }
        self.buf.clear();
        match self.state {
            Start => self.decode_normal(),
            Tag => self.decode_tag(),
            AttributeName | FirstAttribute => self.decode_attr_name(),
            AttributeValue => self.decode_attr_val(),
        }
    }

    fn decode_attr_val(&mut self) -> Result<InternalLexical, LexerError> {
        let quot = self.rdr.find(|&ch| ch == b'\'' || ch == b'"');
        let quot = try!(quot.ok_or(LexerError::EOF));
        debug_assert!(self.buf.is_empty());
        self.buf.extend(self.rdr.by_ref().take_while(|&ch| ch != quot));
        // hack to detect EOF in take_while
        try!(self.peek_char());
        self.state = LexerState::AttributeName;
        Ok(InternalLexical::AttributeValue)
    }

    fn decode_attr_name(&mut self) -> Result<InternalLexical, LexerError> {
        use self::InternalLexical::*;
        use self::LexerError::*;
        loop {
            return match try!(self.next_char()) {
                b'/' => match try!(self.next_char()) {
                    b'>' => {
                        self.state = LexerState::Start;
                        self.stash();
                        debug_assert!(!self.buf.is_empty());
                        Ok(EmptyElementEnd)
                    },
                    _ => Err(ExpectedLT),
                },
                b'>' => {
                    self.state = LexerState::Start;
                    Ok(StartTagClose)
                },
                b'\n' | b'\r' | b'\t' | b' ' => continue,
                c => {
                    self.buf.push(c);
                    break;
                },
            }
        }
        fn next<T: Iterator<Item=u8>>(sel: &mut XmlIterator<T>) -> Result<InternalLexical, LexerError> {
            sel.buf.clear();
            try!(sel.decode_attr_val());
            sel.buf.clear();
            // recursion!
            sel.decode_attr_name()
        }
        fn done<T: Iterator<Item=u8>>(sel: &mut XmlIterator<T>) -> Result<InternalLexical, LexerError> {
            debug_assert!(!sel.buf.is_empty());
            if sel.buf == b"xmlns" {
                next(sel)
            } else {
                sel.state = LexerState::AttributeValue;
                Ok(AttributeName)
            }
        }
        loop {
            match try!(self.next_char()) {
                b'=' => return done(self),

                // whitespace -> search for `=`
                b'\n' | b'\r' | b'\t' | b' ' => break,

                // other namespaces are forwarded
                b':' if self.buf == b"xmlns" => {
                    let eq = self.rdr.find(|&ch| ch == b'=');
                    let _ = try!(eq.ok_or(LexerError::EOF));
                    return next(self)
                },

                c => self.buf.push(c),
            }
        }
        loop {
            match try!(self.next_char()) {
                b'=' => return done(self),

                // whitespace -> search for `=`
                b'\n' | b'\r' | b'\t' | b' ' => continue,

                // this is not the character you are looking for
                _ => return Err(ExpectedEq),
            }
        }
    }
    fn decode_tag(&mut self) -> Result<InternalLexical, LexerError> {
        use self::InternalLexical::*;
        for c in &mut self.rdr {
            if c == b':' {
                self.buf.clear();
                continue;
            }
            if c == b'>' {
                self.state = LexerState::Start;
                return Ok(EndTagName)
            }
            self.buf.push(c);
        }
        Err(LexerError::EOF)
    }

    fn decode_tag_name(&mut self) -> Result<InternalLexical, LexerError> {
        use self::InternalLexical::*;
        loop {
            match try!(self.peek_char()) {
                b'\n' | b'\r' | b'\t' | b' ' | b'/' | b'>' => {
                    debug_assert!(!self.buf.is_empty());
                    self.state = LexerState::FirstAttribute;
                    return Ok(StartTagName);
                },
                b':' => {
                    self.buf.clear();
                    self.rdr.next();
                    continue;
                }
                c => {
                    self.buf.push(c);
                    self.rdr.next();
                }
            }
        }
    }

    fn decode_comment_or_cdata(&mut self) -> Result<(), LexerError> {
        match try!(self.next_char()) {
            b'-' => self.decode_comment(),
            b'[' => self.decode_cdata(),
            _ => Err(BadCommentOrCDATA),
        }
    }

    fn decode_cdata(&mut self) -> Result<(), LexerError> {
        if try!(self.next_char()) != b'C' {
            return Err(BadCDATA);
        }
        if try!(self.next_char()) != b'D' {
            return Err(BadCDATA);
        }
        if try!(self.next_char()) != b'A' {
            return Err(BadCDATA);
        }
        if try!(self.next_char()) != b'T' {
            return Err(BadCDATA);
        }
        if try!(self.next_char()) != b'A' {
            return Err(BadCDATA);
        }
        if try!(self.next_char()) != b'[' {
            return Err(BadCDATA);
        }
        loop {
            self.buf.extend(self.rdr.by_ref().take_while(|&c| c != b']'));
            match try!(self.next_char()) {
                b']' => {},
                c => {
                    self.buf.push(b']');
                    self.buf.push(c);
                    continue;
                },
            }
            match try!(self.next_char()) {
                b'>' => {},
                c => {
                    self.buf.push(b']');
                    self.buf.push(b']');
                    self.buf.push(c);
                    continue;
                },
            }
            return Ok(())
        }
    }

    fn decode_comment(&mut self) -> Result<(), LexerError> {
        use self::LexerError::*;
        if try!(self.next_char()) != b'-' {
            return Err(BadComment);
        }
        loop {
            while try!(self.next_char()) != b'-' {}
            if try!(self.next_char()) != b'-' {
                continue;
            }
            if try!(self.next_char()) != b'>' {
                continue;
            }
            return Ok(())
        }
    }

    fn decode_declaration(&mut self) -> Result<(), LexerError> {
        use self::LexerError::*;
        if try!(self.next_char()) != b'x' {
            return Err(BadDeclaration);
        }
        if try!(self.next_char()) != b'm' {
            return Err(BadDeclaration);
        }
        if try!(self.next_char()) != b'l' {
            return Err(BadDeclaration);
        }
        loop {
            while try!(self.next_char()) != b'?' {}
            if try!(self.next_char()) != b'>' {
                continue;
            }
            return Ok(())
        }
    }

    fn decode_rest(&mut self, rest: &[u8], good: u8) -> Result<(), LexerError> {
        for &c in rest {
            if try!(self.next_char()) != c {
                return Err(BadEscapeSequence);
            }
        }
        self.buf.push(good);
        Ok(())
    }

    fn decode_escaped_hex(&mut self) -> Result<(), LexerError> {
        let mut n = 0;
        let mut leading_zero = true;
        loop {
            match try!(self.next_char()) {
                b';' => {
                    //let mut buf = [0; 4];
                    let ch = char::from_u32(n);
                    let ch = try!(ch.ok_or(EscapedNotUtf8));
                    //let bytes = ch.encode_utf8(&mut buf);
                    //let bytes = try!(bytes.ok_or(EscapedNotUtf8));
                    //self.buf.extend(buf[..bytes].iter().map(|&c| c));
                    // FIXME: this allocation is required in order to be compatible with stable rust, which
                    // doesn't support encoding a `char` into a stack buffer.
                    self.buf.extend(ch.to_string().bytes());
                    return Ok(());
                },
                b'0' if leading_zero => {},
                c => {
                    leading_zero = false;
                    n = try!(n.checked_mul(16).ok_or(EscapedNotUtf8));
                    let num = try!(hex_ch_to_num(c));
                    n = try!(n.checked_add(num).ok_or(EscapedNotUtf8));
                }
            }
        }
    }

    fn decode_escaped_num(&mut self) -> Result<(), LexerError> {
        let mut n = match try!(self.next_char()) {
            b'x' => return self.decode_escaped_hex(),
            c => try!(ch_to_num(c)),
        };
        let mut leading_zero = n == 0;
        loop {
            match try!(self.next_char()) {
                b';' => {
                    //let mut buf = [0; 4];
                    let ch = char::from_u32(n);
                    let ch = try!(ch.ok_or(EscapedNotUtf8));
                    //let bytes = ch.encode_utf8(&mut buf);
                    //let bytes = try!(bytes.ok_or(EscapedNotUtf8));
                    //self.buf.extend(buf[..bytes].iter().map(|&c| c));
                    // FIXME: this allocation is required in order to be compatible with stable rust, which
                    // doesn't support encoding a `char` into a stack buffer.
                    self.buf.extend(ch.to_string().bytes());
                    return Ok(());
                },
                b'0' if leading_zero => {},
                c => {
                    leading_zero = false;
                    n = try!(n.checked_mul(10).ok_or(EscapedNotUtf8));
                    let num = try!(ch_to_num(c));
                    n = try!(n.checked_add(num).ok_or(EscapedNotUtf8));
                }
            }
        }
    }

    fn decode_escaped(&mut self) -> Result<(), LexerError> {
        match try!(self.next_char()) {
            b'#' => return self.decode_escaped_num(),
            b'l' => try!(self.decode_rest(b"t", b'<')),
            b'g' => try!(self.decode_rest(b"t", b'>')),
            b'a' => match try!(self.next_char()) {
                b'p' => try!(self.decode_rest(b"os", b'\'')),
                b'm' => try!(self.decode_rest(b"p", b'&')),
                _ => return Err(BadEscapeSequence),
            },
            b'q' => try!(self.decode_rest(b"uot", b'"')),
            _ => return Err(BadEscapeSequence),
        }
        if try!(self.next_char()) == b';' {
            Ok(())
        } else {
            Err(BadEscapeSequence)
        }
    }

    fn decode_normal(&mut self) -> Result<InternalLexical, LexerError> {
        use self::InternalLexical::*;
        match self.rdr.next() {
            Some(b'<') => match try!(self.next_char()) {
                b'/' => {
                    self.state = LexerState::Tag;
                    Ok(Text)
                },
                b'!' => {
                    try!(self.decode_comment_or_cdata());
                    self.decode_normal()
                },
                b'?' => {
                    try!(self.decode_declaration());
                    self.decode_normal()
                },
                c => {
                    if self.buf.iter().all(|&c| b" \t\n\r".contains(&c)) {
                        self.buf.clear();
                        self.buf.push(c);
                        self.decode_tag_name()
                    } else {
                        Err(LexerError::MixedElementsAndText)
                    }
                }
            },
            Some(b'&') => {
                try!(self.decode_escaped());
                self.decode_normal()
            },
            Some(c) => {
                self.buf.push(c);
                self.decode_normal()
            },
            None => Ok(EndOfFile),
        }
    }

    pub fn ch(&self) -> Result<Lexical, Error> {
        Ok(match self.ch {
            InternalLexical::StartTagClose =>
                Lexical::StartTagClose,
            InternalLexical::Text =>
                Lexical::Text(&self.buf),
            InternalLexical::StartTagName =>
                Lexical::StartTagName(&self.buf),
            InternalLexical::AttributeName =>
                Lexical::AttributeName(&self.buf),
            InternalLexical::AttributeValue =>
                Lexical::AttributeValue(&self.buf),
            InternalLexical::EmptyElementEnd =>
                Lexical::EmptyElementEnd(&self.buf),
            InternalLexical::EndTagName =>
                Lexical::EndTagName(&self.buf),
            InternalLexical::StartOfFile =>
                Lexical::StartOfFile,
            InternalLexical::EndOfFile =>
                Lexical::EndOfFile,
        })
    }

    pub fn bump(&mut self) -> Result<Lexical, Error> {
        print!("bump");
        assert!(self.ch != InternalLexical::EndOfFile);
        self.ch = match self.decode() {
            Ok(ch) => ch,
            Err(e) => return Err(self.lexer_error(e)),
        };
        println!(" -> {:?}", self.ch());
        self.ch()
    }
}

fn hex_ch_to_num(ch: u8) -> Result<u32, LexerError> {
    match ch {
        b'0'...b'9' => Ok((ch as u32) - (b'0' as u32)),
        b'a'...b'f' => Ok((ch as u32) + 10 - (b'a' as u32)),
        b'A'...b'F' => Ok((ch as u32) + 10 - (b'A' as u32)),
        _ => Err(NotAHex(ch)),
    }
}

fn ch_to_num(ch: u8) -> Result<u32, LexerError> {
    match ch {
        b'0'...b'9' => Ok((ch as u32) - (b'0' as u32)),
        _ => Err(NotANumber(ch)),
    }
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum LexerError {
    EOF,
    ExpectedLT,
    ExpectedQuotes,
    Utf8,
    MixedElementsAndText,
    ExpectedEOF,
    ExpectedEq,
    BadComment,
    BadCDATA,
    BadCommentOrCDATA,
    BadDeclaration,
    BadEscapeSequence,
    Unexpected(u8),
    NotANumber(u8),
    NotAHex(u8),
    EscapedNotUtf8,
}
