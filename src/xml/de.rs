#![deny(unused_must_use)]
use super::error::*;
use super::error::ErrorCode::*;
use de;

use std::iter::Peekable;

use std::str;

macro_rules! expect {
    ($sel:expr, $pat:pat, $err:expr) => {{
        match try!($sel.bump()) {
            $pat => {},
            _ => return Err($sel.rdr.expected($err)),
        }
    }}
}

macro_rules! expect_val {
    ($sel:expr, $i:ident, $err:expr) => {{
        try!($sel.bump());
        is_val!($sel, $i, $err)
    }}
}

macro_rules! is_val {
    ($sel:expr, $i:ident, $err:expr) => {{
        match try!($sel.ch()) {
            $i(x) => x,
            _ => return Err($sel.rdr.expected($err)),
        }
    }}
}

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

    fn expected(&self, reason: &'static str) -> Error {
        self.error(Expected(reason))
    }

    fn error(&self, reason: ErrorCode) -> Error {
        Error::SyntaxError(reason, self.line, self.col)
    }

    fn lexer_error(&self, reason: LexerError) -> Error {
        Error::SyntaxError(LexingError(reason), self.line, self.col)
    }

    fn from_utf8<'a>(&self, txt: &'a[u8]) -> Result<&'a str, Error> {
        let txt = str::from_utf8(txt);
        txt.or(Err(Error::SyntaxError(NotUtf8, self.line, self.col)))
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
    stash: Vec<u8>,
    state: LexerState,
    ch: InternalLexical,
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
            stash: Vec::new(),
            state: LexerState::Start,
            ch: InternalLexical::StartOfFile,
        }
    }

    fn stash(&mut self) {
        use std::mem::swap;
        swap(&mut self.buf, &mut self.stash);
    }

    fn stash_view(&self) -> &[u8] {
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

    fn ch(&self) -> Result<Lexical, Error> {
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

    fn bump(&mut self) -> Result<Lexical, Error> {
        print!("bump");
        assert!(self.ch != InternalLexical::EndOfFile);
        self.ch = match self.decode() {
            Ok(ch) => ch,
            Err(e) => return Err(self.rdr.lexer_error(e)),
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

pub struct Deserializer<Iter: Iterator<Item=u8>> {
    rdr: XmlIterator<Iter>,
}

pub struct InnerDeserializer<'a, Iter: Iterator<Item=u8> + 'a> (
    &'a mut XmlIterator<Iter>, &'a mut bool
);

impl<'a, Iter: Iterator<Item=u8> + 'a> InnerDeserializer<'a, Iter> {
    fn decode<T>(
        xi: &mut XmlIterator<Iter>
    ) -> (bool, Result<T, Error>)
    where T : de::Deserialize
    {
        let mut is_seq = false;
        let deser = de::Deserialize::deserialize(&mut InnerDeserializer(xi, &mut is_seq));
        (is_seq, deser)
    }
}

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
        use self::Lexical::*;
        println!("InnerDeserializer::visit");
        match try!(self.0.ch()) {
            StartTagClose => {
                match {
                    let v = expect_val!(self.0, Text, "text");
                    let v = try!(self.0.rdr.from_utf8(v));
                    visitor.visit_str(v)
                } { // try! is broken sometimes
                    Ok(v) => {
                        try!(self.0.bump());
                        Ok(v)
                    },
                    Err(e) => Err(e),
                }
            },
            EmptyElementEnd(_) => visitor.visit_unit(),
            _ => Err(self.0.rdr.expected("start tag close")),
        }
    }

    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("InnerDeserializer::visit_option");
        visitor.visit_some(self)
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("InnerDeserializer::visit_seq");
        *self.1 = true;
        visitor.visit_seq(SeqVisitor::new(self.0))
    }

    fn visit_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("InnerDeserializer::visit_map");
        visitor.visit_map(ContentVisitor::new_attr(&mut self.0))
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
    fn visit<T>(text: &str) -> Result<T, Error>
        where T: de::Deserialize,
    {
        let kds = &mut KeyDeserializer(text);
        de::Deserialize::deserialize(kds)
    }

    fn value_map<T>() -> Result<T, Error>
        where T: de::Deserialize,
    {
        let kds = &mut KeyDeserializer("$value");
        de::Deserialize::deserialize(kds)
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

impl<Iter> Deserializer<Iter>
    where Iter: Iterator<Item=u8>,
{
    /// Creates the Xml parser.
    #[inline]
    pub fn new(rdr: Iter) -> Deserializer<Iter> {
        Deserializer {
            rdr: XmlIterator::new(rdr),
        }
    }

    fn ch(&self) -> Result<Lexical, Error> {
        self.rdr.ch()
    }

    fn error(&self, reason: ErrorCode) -> Error {
        Error::SyntaxError(reason, self.rdr.rdr.line, self.rdr.rdr.col)
    }

    fn end(&mut self) -> Result<(), Error> {
        match try!(self.ch()) {
            Lexical::EndOfFile => Ok(()),
            _ => unimplemented!(),
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
        use self::Lexical::*;
        println!("Deserializer::visit");
        expect!(self.rdr, StartTagName(_), "start tag name");
        try!(self.rdr.bump());
        let is_seq = &mut false;
        let v = try!(InnerDeserializer(&mut self.rdr, is_seq).visit(visitor));
        assert!(!*is_seq);
        match try!(self.rdr.ch()) {
            EndTagName(_) => {},
            EmptyElementEnd(_) => {},
            _ => return Err(self.rdr.rdr.expected("end tag")),
        }
        expect!(self.rdr, EndOfFile, "end of file");
        Ok(v)
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
        use self::Lexical::*;
        println!("Deserializer::visit_map");
        expect!(self.rdr, StartTagName(_), "start tag name"); // TODO: named map
        try!(self.rdr.bump());
        let is_seq = &mut false;
        let v = try!(InnerDeserializer(&mut self.rdr, is_seq).visit_map(visitor));
        assert!(!*is_seq);
        match try!(self.ch()) {
            EndTagName(_) | EmptyElementEnd(_) => {},
            _ => return Err(self.rdr.rdr.expected("end tag")),
        }
        expect!(self.rdr, EndOfFile, "end of file");
        Ok(v)
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
    de: &'a mut XmlIterator<Iter>,
    state: ContentVisitorState,
}

#[derive(Debug)]
enum ContentVisitorState {
    Attribute,
    Element,
    Value,
    Inner,
}

impl <'a, Iter> ContentVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>,
{
    fn new_attr(de: &'a mut XmlIterator<Iter>) -> Self {
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
        use self::ContentVisitorState::*;
        println!("visit_key: {:?}", (&self.state, try!(self.de.ch())));
        match match (&self.state, try!(self.de.ch())) {
            (&Attribute, EmptyElementEnd(_)) => return Ok(None),
            (&Attribute, StartTagClose) => 0,
            (&Attribute, AttributeName(n)) => return Ok(Some(try!(KeyDeserializer::visit(try!(self.de.rdr.from_utf8(n)))))),
            (&Element, StartTagName(n)) => return Ok(Some(try!(KeyDeserializer::visit(try!(self.de.rdr.from_utf8(n)))))),
            (&Inner, Text(_)) => 1,
            (&Inner, _) => 4,
            (&Value, EndTagName(_)) => return Ok(None),
            (&Value, Text([])) => 3,
            (&Value, Text(_)) => return Ok(Some(try!(KeyDeserializer::value_map()))),
            (&Element, EmptyElementEnd(_)) => 2,
            // need closure to work around https://github.com/rust-lang/rfcs/issues/1006
            (&Element, Text(txt)) if (|| txt.iter().all(|&c| b" \t\n\r".contains(&c)))() => 5,
            (&Element, EndTagName(_)) => return Ok(None),
            _ => unimplemented!()
        } {
            0 => {
                // hack for Attribute, StartTagClose
                try!(self.de.bump());
                self.state = Inner;
                self.visit_key()
            },
            1 => {
                // hack for Element, Text
                self.state = Value;
                self.visit_key()
            },
            2 => {
                // hack for Element, EmptyElementEnd
                // happens when coming out of an empty element which is an inner value
                // maybe catch in visit_value?
                try!(self.de.bump());
                self.visit_key()
            },
            3 => {
                // hack for Value, Text([])
                // happens when coming out of an empty element which is an inner value
                // and then directly running into the `</`
                try!(self.de.bump());
                self.visit_key()
            },
            4 => {
                self.state = Element;
                self.visit_key()
            },
            5 => {
                try!(self.de.bump());
                self.visit_key()
            }
            _ => unreachable!()
        }
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        use self::Lexical::*;
        use self::ContentVisitorState::*;
        println!("visit_value: {:?}", &self.state);
        match self.state {
            Attribute => {
                let v = {
                    let v = expect_val!(self.de, AttributeValue, "attribute value");
                    let v = try!(self.de.rdr.from_utf8(v));
                    try!(KeyDeserializer::visit(v))
                };
                try!(self.de.bump());
                Ok(v)
            },
            Element => {
                try!(self.de.bump());
                let (is_seq, v) = InnerDeserializer::decode(&mut self.de);
                let v = try!(v);
                println!("is_seq: {}", is_seq);
                if !is_seq {
                    match try!(self.de.ch()) {
                        EmptyElementEnd(_) => {},
                        EndTagName(_) => {},
                        _ => return Err(self.de.rdr.expected("tag close")),
                    }
                    try!(self.de.bump());
                }
                Ok(v)
            },
            Value => {
                let v = {
                    let v = is_val!(self.de, Text, "text");
                    let v = try!(self.de.rdr.from_utf8(v));
                    try!(KeyDeserializer::visit(v))
                };
                try!(self.de.bump());
                Ok(v)
            },
            Inner => unreachable!(),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        println!("end: {:?}", &self.state);
        Ok(())
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
    de: &'a mut XmlIterator<Iter>,
    done: bool,
}

impl<'a, Iter> SeqVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>,
{
    fn new(de: &'a mut XmlIterator<Iter>) -> Self {
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
        use self::Lexical::*;
        println!("SeqVisitor::visit: {:?}", (self.done, self.de.ch()));
        if self.done {
            return Ok(None);
        }
        let (is_seq, v) = InnerDeserializer::decode(&mut self.de);
        let v = try!(v);
        if is_seq {
            return Err(self.de.rdr.error(XmlDoesntSupportSeqofSeq));
        }
        is_val!(self.de, EndTagName, "end tag");
        self.de.stash();
        try!(self.de.bump());
        // cannot match on bump here due to rust-bug in functions
        // with &mut self arg and & return value
        match match try!(self.de.ch()) {
            StartTagName(n) if n == self.de.stash_view() => 0,
            StartTagName(_) => 1,
            Text(txt) if (|| txt.iter().all(|&c| b" \t\n\r".contains(&c)))() => 2,
            _ => unimplemented!()
        } {
            0 => { try!(self.de.bump()); },
            1 => self.done = true,
            2 => match try!(self.de.bump()) {
                EndTagName(_) => self.done = true,
                _ => unimplemented!(),
            },
            _ => unreachable!()
        }
        Ok(Some(v))
    }

    fn end(&mut self) -> Result<(), Error> {
        println!("SeqVisitor::end");
        Ok(())
    }
}

/// Decodes an xml value from an `Iterator<u8>`.
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

/// Decodes an xml value from a string
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
    where T: de::Deserialize
{
    from_iter(s.bytes())
}
