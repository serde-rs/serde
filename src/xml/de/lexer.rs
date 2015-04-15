#![deny(unused_must_use)]
use xml::error::*;
use xml::error::ErrorCode::*;

use std::iter::Peekable;

use std::str;

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
        self.error(Expected(reason))
    }

    pub fn error(&self, reason: ErrorCode) -> Error {
        Error::SyntaxError(reason, self.rdr.line, self.rdr.col)
    }

    fn lexer_error(&self, reason: LexerError) -> Error {
        self.error(LexingError(reason))
    }

    pub fn from_utf8<'a>(&self, txt: &'a[u8]) -> Result<&'a str, Error> {
        let txt = str::from_utf8(txt);
        txt.or(Err(self.error(NotUtf8)))
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

    fn end(&mut self) -> Result<(), LexerError> {
        while let Some(c) = self.rdr.next() {
            if !b" \n\t\r".contains(&c) {
                return Err(LexerError::ExpectedEOF);
            }
        }
        Ok(())
    }

    fn decode(&mut self) -> Result<InternalLexical, LexerError> {
        self.buf.clear();
        match self.state {
            LexerState::Start => self.decode_normal(),
            LexerState::Tag => self.decode_tag(),
            LexerState::AttributeName => self.decode_attr_name(),
            LexerState::AttributeValue => self.decode_attr_val(),
        }
    }

    fn decode_attr_val(&mut self) -> Result<InternalLexical, LexerError> {
        let quot = self.rdr.find(|&ch| ch == b'\'' || ch == b'"');
        let quot = try!(quot.ok_or(LexerError::EOF));
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
        for c in &mut self.rdr {
            if c == b'=' {
                assert!(!self.buf.is_empty());
                self.state = LexerState::AttributeValue;
                return Ok(AttributeName);
            }
            if b"\n\r\t ".contains(&c) {
                break;
            }
            self.buf.push(c);
        }
        for c in &mut self.rdr {
            if c == b'=' {
                assert!(!self.buf.is_empty());
                self.state = LexerState::AttributeValue;
                return Ok(AttributeName);
            }
            if !b"\n\r\t ".contains(&c) {
                return Err(ExpectedEq);
            }
        }
        Err(EOF)
    }
    fn decode_tag(&mut self) -> Result<InternalLexical, LexerError> {
        use self::InternalLexical::*;
        for c in &mut self.rdr {
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
                    self.state = LexerState::AttributeName;
                    return Ok(StartTagName);
                },
                c => {
                    self.buf.push(c);
                    self.rdr.next();
                }
            }
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
                b'!' => unimplemented!(),
                b'?' => unimplemented!(),
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
            // error: not in this state
            Some(c) if b"'\"=>/".contains(&c) => unimplemented!(),
            Some(b'&') => unimplemented!(),
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

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum LexerError {
    EOF,
    ExpectedLT,
    ExpectedQuotes,
    Utf8,
    MixedElementsAndText,
    ExpectedEOF,
    ExpectedEq,
}
