#![deny(unused_must_use)]
use super::error::*;
use super::error::ErrorCode::*;
use de;

use std::iter::Peekable;

use std::str::from_utf8;

enum InnerMapState {
    Unit,
    Value,
    Inner,
    Attr,
    Whitespace,
}

macro_rules! next {
    ($sel:expr) => (
        match $sel.rdr.next() {
            None => return Err($sel.error(EOF)),
            Some(Err(e)) => return Err($sel.error(LexingError(e))),
            Some(Ok(x)) => x,
        }
    )
}

macro_rules! expect {
    ($sel:expr, $what:expr) => (
        match $sel.rdr.next() {
            None => return Err($sel.error(Expected($what, EOF)))
            Err(e) => return Err($sel.error(LexingError(e))),
            Ok(x) if x == $what => {},
            Ok(x) => return Err($sel.error(Expected($what, x))),
        }
    )
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Lexical {
    StartTagBegin,
    Text(u8),
    StartTagClose,

    StartTagName,

    AttributeName,
    AttributeValue,

    EmptyElementEnd,

    EndTagBegin,
    EndTagName,
}

enum LexerState {
    Start,
    TagName,
    AttributeName,
    AttributeValue,
    EndTag,
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

struct XmlIterator<Iter: Iterator<Item=u8>> {
    rdr: LineColIterator<Iter>,
    buf: Vec<u8>,
    state: LexerState,
}

impl<Iter> XmlIterator<Iter>
    where Iter: Iterator<Item=u8>,
{
    #[inline]
    pub fn new(rdr: Iter) -> XmlIterator<Iter> {
        XmlIterator {
            rdr: LineColIterator {
                rdr: rdr.peekable(),
                line: 1,
                col: 1,
            },
            buf: Vec::with_capacity(128),
            state: LexerState::Start,
        }
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

    fn next_non_whitespace_char(&mut self) -> Result<u8, LexerError> {
        loop {
            match try!(self.next_char()) {
                b' ' | b'\n' | b'\r' | b'\t' => {},
                c => return Ok(c),
            }
        }
    }

    fn decode(&mut self) -> Result<Lexical, LexerError> {
        match self.state {
            LexerState::Start => self.decode_normal(),
            LexerState::TagName => self.decode_tag_name(),
            LexerState::AttributeName => self.decode_attr_name(),
            LexerState::AttributeValue => self.decode_attr_val(),
            LexerState::EndTag => self.decode_end_tag(),
        }
    }

    fn decode_end_tag(&mut self) -> Result<Lexical, LexerError> {
        for c in &mut self.rdr {
            if c == b'>' {
                self.state = LexerState::Start;
                return Ok(Lexical::EndTagName)
            }
            self.buf.push(c);
        }
        Err(LexerError::EOF)
    }

    fn decode_attr_val(&mut self) -> Result<Lexical, LexerError> {
        let quot = self.rdr.find(|&ch| ch == b'\'' || ch == b'"');
        let quot = try!(quot.ok_or(LexerError::EOF));
        assert!(self.buf.is_empty());
        self.buf.extend(self.rdr.by_ref().take_while(|&ch| ch != quot));
        // hack to detect EOF in take_while
        try!(self.peek_char());
        self.state = LexerState::AttributeName;
        Ok(Lexical::AttributeValue)
    }

    fn decode_attr_name(&mut self) -> Result<Lexical, LexerError> {
        use self::Lexical::*;
        use self::LexerError::*;
        assert!(self.buf.is_empty());
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

    fn decode_tag_name(&mut self) -> Result<Lexical, LexerError> {
        use self::Lexical::*;
        loop {
            match try!(self.peek_char()) {
                b'\n' | b'\r' | b'\t' | b' ' | b'/' | b'>' => {
                    assert!(!self.buf.is_empty());
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

    fn decode_normal(&mut self) -> Result<Lexical, LexerError> {
        use self::Lexical::*;
        match try!(self.next_char()) {
            b'<' => match try!(self.peek_char()) {
                b'/' => {
                    // won't panic
                    self.next_char().unwrap();
                    self.state = LexerState::EndTag;
                    Ok(EndTagBegin)
                },
                b'!' => unimplemented!(),
                b'?' => unimplemented!(),
                _ => {
                    self.state = LexerState::TagName;
                    Ok(StartTagBegin)
                },
            },
            // error: not in this state
            b'"' | b'\'' | b'=' | b'>' | b'/' => unimplemented!(),
            b'&' => unimplemented!(),
            c => Ok(Text(c)),
        }
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

impl<Iter: Iterator<Item=u8>> Iterator for XmlIterator<Iter> {
    type Item = Result<Lexical, Error>;
    fn next(&mut self) -> Option<Result<Lexical, Error>> {
        self.rdr.peek().map(
            |_| self.decode()
        ).map(|res| res.map_err(|e|
            Error::SyntaxError(
                LexingError(e),
                self.rdr.line,
                self.rdr.col,
            )
        ))
    }
}

pub struct Deserializer<Iter: Iterator<Item=u8>> {
    rdr: XmlIterator<Iter>,
    ch: Option<Lexical>,
}

pub struct InnerDeserializer<'a, Iter: Iterator<Item=u8> + 'a> (
    &'a mut Deserializer<Iter>,
);

// TODO: this type should expect self.0.buf.is_empty()
// but that can only be done after a SeqVisitor rehaul to get rid of the alloc
impl<'a, Iter> de::Deserializer for InnerDeserializer<'a, Iter>
where Iter: Iterator<Item=u8>,
{
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("InnerDeserializer::visit");
        self.0.rdr.buf.clear();
        try!(self.0.bump());
        try!(self.0.skip_whitespace());
        let v = try!(self.0.read_value(visitor));
        try!(self.0.read_next_tag());
        Ok(v)
    }

    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("InnerDeserializer::visit_option");
        self.0.rdr.buf.clear();
        try!(self.0.bump());
        try!(self.0.skip_whitespace());
        match try!(self.0.ch()) {
            Lexical::EmptyElementEnd => {
                try!(self.0.bump());
                visitor.visit_none()
            },
            Lexical::StartTagClose => visitor.visit_some(self),
            _ => Err(self.0.error(InvalidOptionalElement)),
        }
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("InnerDeserializer::visit_seq");
        visitor.visit_seq(SeqVisitor::new(self.0))
    }

    fn visit_map<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("InnerDeserializer::visit_map");
        self.0.rdr.buf.clear();
        println!("{:?}", self.0.rdr.buf);
        let v = try!(self.0.parse_inner_map(visitor));
        try!(self.0.read_next_tag());
        Ok(v)
    }

    fn visit_named_unit<V>(&mut self, _name: &str, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }

    fn visit_named_seq<V>(&mut self, _name: &str, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }
}

pub struct KeyDeserializer<'a> (
    &'a str,
);

impl<'a> KeyDeserializer<'a> {
    fn decode<T, Iter>(de: &Deserializer<Iter>) -> Result<T, Error>
        where Iter: Iterator<Item=u8>,
        T: de::Deserialize,
    {
        let s = from_utf8(&de.rdr.buf);
        match s {
            Ok(text) => {
                let kds = &mut KeyDeserializer(text);
                let val = de::Deserialize::deserialize(kds);
                if val.is_err() {
                    println!("decode, err");
                }
                Ok(try!(val))
            },
            Err(_) => Err(de.error(NotUtf8))
        }
    }

    fn value_map<T>() -> Result<T, Error>
        where T: de::Deserialize,
    {
        let kds = &mut KeyDeserializer("$value");
        de::Deserialize::deserialize(kds)
    }

    fn from_utf8<Iter>(de: &Deserializer<Iter>) -> Result<&str, Error>
        where Iter: Iterator<Item=u8>,
    {
        let s = from_utf8(&de.rdr.buf);
        s.or(Err(de.error(NotUtf8)))
    }
}

impl<'a> de::Deserializer for KeyDeserializer<'a> {
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("{:?} keydeserializer::visit", self as *const Self);
        println!("{:?} {:?}", self as *const Self, self.0);
        match visitor.visit_str(self.0) {
            Ok(x) => Ok(x),
            Err(x) => {println!("err"); Err(x)},
        }
    }

    #[inline]
    fn visit_option<V>(&mut self, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }

    #[inline]
    fn visit_enum<V>(&mut self, _enum: &str, _visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        unimplemented!()
    }

    #[inline]
    fn visit_seq<V>(&mut self, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }
}

fn ws() -> [Lexical; 4] {
    use self::Lexical::*;
    [Text(b' '), Text(b'\r'), Text(b'\t'), Text(b'\n')]
}

impl<Iter> Deserializer<Iter>
    where Iter: Iterator<Item=u8>,
{
    /// Creates the Xml parser.
    #[inline]
    pub fn new(rdr: Iter) -> Result<Deserializer<Iter>, Error> {
        let mut p = Deserializer {
            rdr: XmlIterator::new(rdr),
            ch: Some(Lexical::Text(b'\0')),
        };
        try!(p.bump());
        return Ok(p);
    }

    #[inline]
    pub fn end(&mut self) -> Result<(), Error> {
        try!(self.skip_whitespace());
        assert!(self.eof());
        assert!(self.rdr.buf.is_empty());
        Ok(())
    }

    fn eof(&self) -> bool { self.ch.is_none() }

    fn bump(&mut self) -> Result<(), Error> {
        print!("bump: {:?}", (self.rdr.rdr.line, self.rdr.rdr.col, self.ch));
        if let None = self.ch {
            panic!("iterator overrun");
        }
        self.ch = match self.rdr.next() {
            None => None,
            Some(x) => Some(try!(x)),
        };
        println!(" -> {:?}", self.ch);
        Ok(())
    }

    fn expect(&self, c: Lexical) -> Result<(), Error> {
        match self.ch {
            None => Err(self.error(EOF)),
            Some(ch) if ch == c => Ok(()),
            Some(ch) => Err(self.error(Expected(c, ch))),
        }
    }

    fn ch(&self) -> Result<Lexical, Error> {
        self.ch.ok_or(self.error(EOF))
    }

    fn ch_is(&self, c: Lexical) -> bool {
        self.ch == Some(c)
    }

    fn ch_is_one_of(&self, c: &[Lexical]) -> bool {
        for &c in c {
            if Some(c) == self.ch {
                return true;
            }
        }
        return false;
    }

    fn error(&self, reason: ErrorCode) -> Error {
        Error::SyntaxError(reason, self.rdr.rdr.line, self.rdr.rdr.col)
    }

    fn skip_whitespace(&mut self) -> Result<(), Error> {
        while self.ch_is_one_of(&ws()) { try!(self.bump()); }
        Ok(())
    }

    fn read_whitespace(&mut self) -> Result<(), Error> {
        use self::Lexical::Text;
        while let Some(Text(c)) = self.ch {
            if ws().iter().any(|&ch| ch == Text(c)) {
                self.rdr.buf.push(c);
                try!(self.bump());
            } else {
                return Ok(());
            }
        }
        Ok(())
    }

    fn skip_until(&mut self, ch: Lexical) -> Result<(), Error> {
        while let Some(c) = self.ch {
            if ch == c {
                return Ok(())
            }
            try!(self.bump());
        }
        Err(self.error(EOF))
    }

    fn read_until(&mut self, ch: Lexical) -> Result<(), Error> {
        while let Some(c) = self.ch {
            if ch == c {
                return Ok(())
            }
            match c {
                Lexical::Text(c) => self.rdr.buf.push(c),
                _ => unimplemented!()
            }
            try!(self.bump());
        }
        Err(self.error(EOF))
    }

    fn read_next_tag(&mut self) -> Result<(), Error> {
        println!("read_next_tag");
        try!(self.skip_whitespace());
        self.read_tag()
    }

    fn read_tag(&mut self) -> Result<(), Error> {
        use self::Lexical::*;
        println!("read_tag");
        match self.ch.unwrap() {
            EndTagBegin => Ok(()),
            StartTagBegin => {
                try!(self.bump());
                assert!(self.ch_is(StartTagName));
                Ok(())
            },
            x => Err(self.error(Expected(StartTagBegin, x))),
        }
    }

    fn parse_inner_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        use self::InnerMapState::*;
        println!("parse_inner_map");
        match try!(self.read_inner_map()) {
            Unit => visitor.visit_map(EmptyMapVisitor),
            Value => visitor.visit_map(ContentVisitor::new_value(self)),
            Whitespace => {
                self.rdr.buf.clear();
                visitor.visit_map(EmptyMapVisitor)
            }
            Inner => {
                let val = visitor.visit_map(ContentVisitor::new_inner(self));
                self.rdr.buf.clear();
                val
            },
            Attr => visitor.visit_map(ContentVisitor::new_attr(self)),
        }
    }

    fn read_inner_map(&mut self) -> Result<InnerMapState, Error> {
        use self::Lexical::*;
        try!(self.skip_whitespace());
        match try!(self.ch()) {
            EmptyElementEnd => {
                try!(self.bump());
                Ok(InnerMapState::Unit)
            },
            StartTagClose => {
                try!(self.bump());
                assert!(self.rdr.buf.is_empty());
                try!(self.read_whitespace());
                if self.ch_is(EndTagBegin) {
                    try!(self.bump());
                    assert!(self.ch_is(EndTagName));
                    self.rdr.buf.clear();
                    try!(self.bump());
                    Ok(InnerMapState::Whitespace)
                } else if self.ch_is(StartTagBegin) {
                    self.rdr.buf.clear();
                    try!(self.read_tag());
                    Ok(InnerMapState::Inner)
                } else {
                    // $value map
                    try!(self.read_until(EndTagBegin));
                    Ok(InnerMapState::Value)
                }
            }
            AttributeName => Ok(InnerMapState::Attr),
            c => Err(self.error(Expected(StartTagClose, c))),
        }
    }

    fn read_value<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor
    {
        use self::Lexical::*;
        println!("read_value");
        match try!(self.ch()) {
            StartTagClose => {
                try!(self.bump());
                try!(self.read_until(EndTagBegin));
                // try! is broken here
                let v = match visitor.visit_str(try!(KeyDeserializer::from_utf8(self))) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                self.rdr.buf.clear();
                try!(self.bump());
                assert!(self.ch_is(EndTagName));
                self.rdr.buf.clear();
                try!(self.bump());
                Ok(v)
            },
            EmptyElementEnd => {
                // try! is broken here
                let v = match visitor.visit_unit() {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                try!(self.bump());
                Ok(v)
            },
            c => Err(self.error(Expected(StartTagClose, c))),
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
        try!(self.read_next_tag());
        self.rdr.buf.clear();
        try!(self.bump());
        self.read_value(visitor)
    }

    #[inline]
    fn visit_option<V>(&mut self, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }

    #[inline]
    fn visit_enum<V>(&mut self, _enum: &str, _visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        unimplemented!()
    }

    #[inline]
    fn visit_seq<V>(&mut self, _visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        unimplemented!()
    }

    #[inline]
    fn visit_map<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        try!(self.read_next_tag());
        self.rdr.buf.clear(); // TODO: visit_named_map
        try!(self.bump());
        self.parse_inner_map(visitor)
    }
}

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

    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(EmptySeqVisitor)
    }

    fn visit_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_map(EmptyMapVisitor)
    }
}

struct EmptySeqVisitor;
impl de::SeqVisitor for EmptySeqVisitor {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        Ok(None)
    }

    fn end(&mut self) -> Result<(), Error> { Ok(()) }
}

struct EmptyMapVisitor;
impl de::MapVisitor for EmptyMapVisitor {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: de::Deserialize,
    { Ok(None) }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    { unreachable!() }

    fn end(&mut self) -> Result<(), Error> { Ok(()) }

    fn missing_field<V>(&mut self, _field: &'static str) -> Result<V, Error>
        where V: de::Deserialize,
    {
        Ok(try!(de::Deserialize::deserialize(&mut UnitDeserializer)))
    }
}

struct ContentVisitor<'a, Iter: 'a>
    where Iter: Iterator<Item=u8>,
{
    de: &'a mut Deserializer<Iter>,
    state: ContentVisitorState,
}

#[derive(Debug)]
enum ContentVisitorState {
    Attribute,
    Element,
    Value,
}

impl <'a, Iter> ContentVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>,
{
    fn new_inner(de: &'a mut Deserializer<Iter>) -> Self {
        ContentVisitor {
            de: de,
            state: ContentVisitorState::Element,
        }
    }

    fn new_value(de: &'a mut Deserializer<Iter>) -> Self {
        ContentVisitor {
            de: de,
            state: ContentVisitorState::Value,
        }
    }

    fn new_attr(de: &'a mut Deserializer<Iter>) -> Self {
        ContentVisitor {
            de: de,
            state: ContentVisitorState::Attribute,
        }
    }
}

impl<'a, Iter> de::MapVisitor for ContentVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>
{
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: de::Deserialize,
    {
        use self::Lexical::*;
        println!("{:?} visit_key: {:?}", self as *const Self, (&self.state, self.de.rdr.rdr.line, self.de.rdr.rdr.col));
        if self.de.rdr.buf.is_empty() {
            return Ok(None);
        }
        match self.state {
            ContentVisitorState::Element => {
                assert!(self.de.ch_is(StartTagName));
                KeyDeserializer::decode(self.de)
            }
            ContentVisitorState::Attribute => KeyDeserializer::decode(self.de),

            ContentVisitorState::Value => KeyDeserializer::value_map(),
        }.map(|x| Some(x))
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        println!("{:?} visit_value: {:?}", self as *const Self, (&self.state, self.de.rdr.rdr.line, self.de.rdr.rdr.col));
        match self.state {
            ContentVisitorState::Element => {
                let ids = &mut InnerDeserializer(self.de);
                de::Deserialize::deserialize(ids)
            },

            ContentVisitorState::Value => {
                let val = KeyDeserializer::decode(self.de);
                self.de.rdr.buf.clear();
                try!(self.de.bump());
                assert!(self.de.ch_is(Lexical::EndTagName));
                self.de.rdr.buf.clear();
                try!(self.de.bump());
                val
            },

            ContentVisitorState::Attribute => {
                use self::InnerMapState::*;
                try!(self.de.expect(Lexical::AttributeName));
                self.de.rdr.buf.clear();
                try!(self.de.bump());
                try!(self.de.expect(Lexical::AttributeValue));
                let val = try!(KeyDeserializer::decode(self.de));
                self.de.rdr.buf.clear();
                try!(self.de.bump());
                match try!(self.de.read_inner_map()) {
                    Value => self.state = ContentVisitorState::Value,
                    Inner => self.state = ContentVisitorState::Element,
                    Attr | Unit | Whitespace => {},
                }
                Ok(val)
            },
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        println!("{:?} end: {:?}", self as *const Self, (&self.state, self.de.rdr.rdr.line, self.de.rdr.rdr.col));
        match self.state {
            ContentVisitorState::Element => {
                try!(self.de.expect(Lexical::EndTagBegin));
                try!(self.de.bump());
                try!(self.de.read_until(Lexical::EndTagName));
                try!(self.de.bump());
                Ok(())
            },

            ContentVisitorState::Value => Ok(()),

            ContentVisitorState::Attribute => Ok(()),
        }
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V, Error>
        where V: de::Deserialize,
    {
        println!("missing field: {}", field);
        // See if the type can deserialize from a unit.
        de::Deserialize::deserialize(&mut UnitDeserializer)
    }
}

struct SeqVisitor<'a, Iter: 'a + Iterator<Item=u8>> {
    de: &'a mut Deserializer<Iter>,
    done: bool,
}

impl<'a, Iter> SeqVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>,
{
    fn new(de: &'a mut Deserializer<Iter>) -> Self {
        SeqVisitor {
            de: de,
            done: false,
        }
    }
}

impl<'a, Iter> de::SeqVisitor for SeqVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>,
{
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        println!("SeqVisitor::visit: {:?}", (self.done, self.de.ch));
        if self.done {
            return Ok(None);
        }
        // need to copy here
        // could compare closing tag with next opening tag instead
        // but that requires modification of InnerDeserializer
        assert!(!self.de.rdr.buf.is_empty());
        let name = self.de.rdr.buf.clone();
        println!("{:?} reading value", self as *const Self);
        let val = {
            println!("{:?} reading inner", self as *const Self);
            let ids = &mut InnerDeserializer(self.de);
            try!(de::Deserialize::deserialize(ids))
        };
        println!("{:?} got seq valu", self as *const Self);
        if self.de.rdr.buf.is_empty() {
            println!("{:?} buf empty", self as *const Self);
            // last of the sequence and last of the map
            self.done = true;
        } else {
            // compare next element name to current
            assert!(!self.de.rdr.buf.is_empty());
            if self.de.rdr.buf != name {
                println!("seq done: {:?} != {:?}", self.de.rdr.buf, name);
                self.done = true;
            }
        }
        Ok(Some(val))
    }

    fn end(&mut self) -> Result<(), Error> {
        println!("SeqVisitor::end: {:?}", self.de.rdr.buf);
        Ok(())
    }
}

/// Decodes an xml value from an `Iterator<u8>`.
pub fn from_iter<I, T>(iter: I) -> Result<T, Error>
    where I: Iterator<Item=u8>,
          T: de::Deserialize
{
    let mut de = try!(Deserializer::new(iter));
    let value = try!(de::Deserialize::deserialize(&mut de));

    // Make sure the whole stream has been consumed.
    try!(de.end());
    Ok(value)
}

/// Decodes an xml value from a string
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
    where T: de::Deserialize
{
    from_iter(s.bytes())
}
