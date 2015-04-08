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
    TagClose,

    AttributeValue,

    EmptyElementEnd,

    EndTagBegin,

    Eq,
}

struct XmlIterator<Iter: Iterator<Item=u8>> {
    rdr: Peekable<Iter>,
    line: usize,
    col: usize,
    buf: Vec<u8>,
}

impl<Iter> XmlIterator<Iter>
    where Iter: Iterator<Item=u8>,
{
    #[inline]
    pub fn new(rdr: Iter) -> XmlIterator<Iter> {
        XmlIterator {
            rdr: rdr.peekable(),
            line: 1,
            col: 1,
            buf: Vec::with_capacity(128),
        }
    }

    fn peek_char(&mut self) -> Result<u8, LexerError> {
        self.rdr.peek().ok_or(LexerError::EOF).map(|&c| c)
    }

    fn next_char(&mut self) -> Result<u8, LexerError> {
        print!(" -> {:?}", self.peek_char().map(|x| x as char));
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
        use self::Lexical::*;
        use self::LexerError::*;
        match try!(self.next_char()) {
            b'<' => match try!(self.peek_char()) {
                b'/' => {
                    // won't panic
                    self.next_char().unwrap();
                    Ok(EndTagBegin)
                },
                b'!' => unimplemented!(),
                b'?' => unimplemented!(),
                _ => Ok(StartTagBegin),
            },
            quote @ b'"' | quote @ b'\'' => {
                assert!(self.buf.is_empty());
                loop {
                    match try!(self.next_char()) {
                        c if c == quote => return Ok(AttributeValue),
                        c => self.buf.push(c),
                    }
                }
            },
            b'=' => Ok(Eq),
            b'>' => Ok(TagClose),
            b'/' => match try!(self.next_char()) {
                b'>' => Ok(EmptyElementEnd),
                _ => Err(ExpectedLT),
            },
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
}

impl<Iter: Iterator<Item=u8>> Iterator for XmlIterator<Iter> {
    type Item = Result<Lexical, LexerError>;
    fn next(&mut self) -> Option<Result<Lexical, LexerError>> {
        match self.rdr.peek() {
            None => None,
            Some(&b'\n') => {
                self.line += 1;
                self.col = 1;
                Some(self.decode())
            },
            Some(_) => {
                self.col += 1;
                Some(self.decode())
            },
        }
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
        let v = try!(self.0.read_value(visitor));
        try!(self.0.read_next_tag());
        Ok(v)
    }

    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("InnerDeserializer::visit_option");
        self.0.rdr.buf.clear();
        match self.0.ch.unwrap() {
            Lexical::EmptyElementEnd => {
                self.0.bump();
                visitor.visit_none()
            },
            Lexical::Text(c) if b"\t\r\n >".contains(&c) => {
                visitor.visit_some(self)
            },
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
    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("{:?} keydeserializer::visit_option", self as *const Self);
        if self.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
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
    pub fn new(rdr: Iter) -> Deserializer<Iter> {
        let mut p = Deserializer {
            rdr: XmlIterator::new(rdr),
            ch: Some(Lexical::Text(b'\0')),
        };
        p.bump();
        return p;
    }

    #[inline]
    pub fn end(&mut self) -> Result<(), Error> {
        self.skip_whitespace();
        assert!(self.eof());
        assert!(self.rdr.buf.is_empty());
        Ok(())
    }

    fn eof(&self) -> bool { self.ch.is_none() }

    fn bump(&mut self) {
        print!("bump: {:?}", (self.rdr.line, self.rdr.col, self.ch));
        if let None = self.ch {
            panic!("iterator overrun");
        }
        self.ch = match self.rdr.next() {
            None => None,
            Some(x) => Some(x.unwrap()),
        };
        println!(" -> {:?}", self.ch);
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
        Error::SyntaxError(reason, self.rdr.line, self.rdr.col)
    }

    fn skip_whitespace(&mut self) {
        while self.ch_is_one_of(&ws()) { self.bump(); }
    }

    fn read_whitespace(&mut self) {
        use self::Lexical::Text;
        while let Some(Text(c)) = self.ch {
            if ws().iter().any(|&ch| ch == Text(c)) {
                self.rdr.buf.push(c);
                self.bump();
            } else {
                return;
            }
        }
    }

    fn skip_until(&mut self, ch: Lexical) -> Result<(), Error> {
        while let Some(c) = self.ch {
            if ch == c {
                return Ok(())
            }
            self.bump();
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
            self.bump();
        }
        Err(self.error(EOF))
    }

    fn read_next_tag(&mut self) -> Result<(), Error> {
        println!("read_next_tag");
        self.skip_whitespace();
        self.read_tag()
    }

    fn read_tag(&mut self) -> Result<(), Error> {
        use self::Lexical::*;
        println!("read_tag");
        match self.ch.unwrap() {
            EndTagBegin => Ok(()),
            StartTagBegin => {
                self.bump();
                while let Some(c) = self.ch {
                    if self.ch_is_one_of(&ws()) {
                        return Ok(());
                    }
                    if self.ch_is_one_of(&[TagClose, EmptyElementEnd]) {
                        return Ok(());
                    }
                    match c {
                        Text(c) => self.rdr.buf.push(c),
                        _ => unimplemented!()
                    }
                    self.bump();
                }
                Err(self.error(EOF))
            },
            x => Err(self.error(Expected(StartTagBegin, x))),
        }
    }

    fn read_attr_name(&mut self) -> Result<(), Error> {
        println!("read_attr_name");
        while let Some(c) = self.ch {
            if self.ch_is_one_of(&ws()) {
                return Ok(());
            }
            if self.ch_is(Lexical::Eq) {
                return Ok(());
            }
            match c {
                Lexical::Text(c) => self.rdr.buf.push(c),
                _ => unimplemented!()
            }
            self.bump();
        }
        Err(self.error(EOF))
    }

    fn parse_inner_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        use self::InnerMapState::*;
        println!("parse_inner_map");
        match try!(self.read_inner_map()) {
            Unit => de::Deserialize::deserialize(&mut UnitDeserializer),
            Value => visitor.visit_map(ContentVisitor::new_value(self)),
            Whitespace => {
                self.rdr.buf.clear();
                de::Deserialize::deserialize(&mut UnitDeserializer)
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
        self.skip_whitespace();
        match self.ch {
            None => Err(self.error(EOF)),
            Some(EmptyElementEnd) => {
                self.bump();
                Ok(InnerMapState::Unit)
            },
            Some(TagClose) => {
                self.bump();
                assert!(self.rdr.buf.is_empty());
                self.read_whitespace();
                if self.ch_is(EndTagBegin) {
                    try!(self.skip_until(TagClose));
                    self.bump();
                    Ok(InnerMapState::Whitespace)
                } else if self.ch_is(StartTagBegin) {
                    self.rdr.buf.clear();
                    try!(self.read_tag());
                    Ok(InnerMapState::Inner)
                } else {
                    // $value map
                    try!(self.read_until(EndTagBegin));
                    try!(self.skip_until(TagClose));
                    self.bump();
                    Ok(InnerMapState::Value)
                }
            }
            _ => {
                assert!(self.rdr.buf.is_empty());
                try!(self.read_attr_name());
                self.skip_whitespace();
                assert!(self.ch_is(Eq));
                self.bump();
                self.skip_whitespace();
                assert!(self.ch_is(AttributeValue));
                Ok(InnerMapState::Attr)
            }
        }
    }

    fn read_value<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor
    {
        use self::InnerMapState::*;
        self.rdr.buf.clear();
        match try!(self.read_inner_map()) {
            Unit => visitor.visit_unit(),
            Value | Whitespace => {
                let v = visitor.visit_str(try!(KeyDeserializer::from_utf8(self)));
                self.rdr.buf.clear();
                v
            },
            Inner => {
                let val = visitor.visit_map(ContentVisitor::new_inner(self));
                self.rdr.buf.clear();
                val
            },
            Attr => Err(self.error(RawValueCannotHaveAttributes)),
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
        self.rdr.buf.clear();
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
        println!("{:?} visit_key: {:?}", self as *const Self, (&self.state, self.de.rdr.line, self.de.rdr.col));
        if self.de.rdr.buf.is_empty() {
            return Ok(None);
        }
        match self.state {
            ContentVisitorState::Element
            | ContentVisitorState::Attribute => KeyDeserializer::decode(self.de),

            ContentVisitorState::Value => KeyDeserializer::value_map(),
        }
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        println!("{:?} visit_value: {:?}", self as *const Self, (&self.state, self.de.rdr.line, self.de.rdr.col));
        match self.state {
            ContentVisitorState::Element => {
                let ids = &mut InnerDeserializer(self.de);
                de::Deserialize::deserialize(ids)
            },

            ContentVisitorState::Value => {
                let val = KeyDeserializer::decode(self.de);
                self.de.rdr.buf.clear();
                val
            },

            ContentVisitorState::Attribute => {
                use self::InnerMapState::*;
                self.de.rdr.buf.clear();
                assert!(self.de.ch_is(Lexical::AttributeValue));
                let quot = self.de.ch.unwrap();
                self.de.bump();
                try!(self.de.read_until(quot));
                self.de.bump();
                let val = try!(KeyDeserializer::decode(self.de));
                self.de.rdr.buf.clear();
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
        println!("{:?} end: {:?}", self as *const Self, (&self.state, self.de.rdr.line, self.de.rdr.col));
        match self.state {
            ContentVisitorState::Element => {
                assert!(self.de.ch_is(Lexical::EndTagBegin));
                self.de.bump();
                try!(self.de.read_until(Lexical::TagClose));
                self.de.bump();
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
        println!("{:?} reading value", self as *const Self);
        // need to copy here
        // could compare closing tag with next opening tag instead
        // but that requires modification of InnerDeserializer
        assert!(!self.de.rdr.buf.is_empty());
        let name = self.de.rdr.buf.clone();
        self.de.rdr.buf.clear();
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
