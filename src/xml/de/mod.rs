#![deny(unused_must_use, unused_imports)]
use xml::error::*;
use xml::error::ErrorCode::*;
use de;

mod lexer;
use self::lexer::Lexical::*;
pub use self::lexer::LexerError;

macro_rules! expect {
    ($sel:expr, $pat:pat, $err:expr) => {{
        match try!($sel.bump()) {
            $pat => {},
            _ => return Err($sel.expected($err)),
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
            _ => return Err($sel.expected($err)),
        }
    }}
}

pub struct Deserializer<Iter: Iterator<Item=u8>> {
    rdr: lexer::XmlIterator<Iter>,
}

pub struct InnerDeserializer<'a, Iter: Iterator<Item=u8> + 'a> (
    &'a mut lexer::XmlIterator<Iter>, &'a mut bool
);

impl<'a, Iter: Iterator<Item=u8> + 'a> InnerDeserializer<'a, Iter> {
    fn decode<T>(
        xi: &mut lexer::XmlIterator<Iter>
    ) -> (bool, Result<T, Error>)
    where T : de::Deserialize
    {
        let mut is_seq = false;
        let deser = de::Deserialize::deserialize(&mut InnerDeserializer(xi, &mut is_seq));
        (is_seq, deser)
    }
}

impl<'a, Iter> de::Deserializer for InnerDeserializer<'a, Iter>
where Iter: Iterator<Item=u8>,
{
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("InnerDeserializer::visit");
        match try!(self.0.ch()) {
            StartTagClose => {
                match {
                    let v = expect_val!(self.0, Text, "text");
                    let v = try!(self.0.from_utf8(v));
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
            _ => Err(self.0.expected("start tag close")),
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

    #[inline]
    fn visit_enum<V>(&mut self, _enum: &str, mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        println!("InnerDeserializer::visit_enum");
        visitor.visit(VariantVisitor(&mut self.0))
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
        println!("keydeserializer::visit");
        println!("{:?}", self.0);
        match visitor.visit_str(self.0) {
            Ok(x) => Ok(x),
            Err(x) => { println!("err"); Err(x) },
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
            rdr: lexer::XmlIterator::new(rdr),
        }
    }

    fn ch(&self) -> Result<lexer::Lexical, Error> {
        self.rdr.ch()
    }

    fn end(&mut self) -> Result<(), Error> {
        match try!(self.ch()) {
            EndOfFile => Ok(()),
            _ => Err(self.rdr.error(ExpectedEOF)),
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
        println!("Deserializer::visit");
        expect!(self.rdr, StartTagName(_), "start tag name");
        try!(self.rdr.bump());
        let is_seq = &mut false;
        let v = try!(InnerDeserializer(&mut self.rdr, is_seq).visit(visitor));
        assert!(!*is_seq);
        match try!(self.rdr.ch()) {
            EndTagName(_) => {},
            EmptyElementEnd(_) => {},
            _ => return Err(self.rdr.expected("end tag")),
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
    fn visit_enum<V>(&mut self, _enum: &str, mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        expect!(self.rdr, StartTagName(_), "start tag name");
        try!(self.rdr.bump());
        let v = visitor.visit(VariantVisitor(&mut self.rdr));
        let v = try!(v);
        expect!(self.rdr, EndOfFile, "end of file");
        Ok(v)
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
        println!("Deserializer::visit_map");
        expect!(self.rdr, StartTagName(_), "start tag name"); // TODO: named map
        try!(self.rdr.bump());
        let is_seq = &mut false;
        let v = try!(InnerDeserializer(&mut self.rdr, is_seq).visit_map(visitor));
        assert!(!*is_seq);
        match try!(self.ch()) {
            EndTagName(_) | EmptyElementEnd(_) => {},
            _ => return Err(self.rdr.expected("end tag")),
        }
        expect!(self.rdr, EndOfFile, "end of file");
        Ok(v)
    }
}

struct VariantVisitor<'a, Iter: Iterator<Item=u8> + 'a>
(
    &'a mut lexer::XmlIterator<Iter>,
);

impl<'a, Iter: 'a> de::VariantVisitor for VariantVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>
{
    type Error = Error;

    fn visit_variant<V>(&mut self) -> Result<V, Self::Error>
        where V: de::Deserialize
    {
        if b"xsi:type" != is_val!(self.0, AttributeName, "attribute name") {
            return Err(self.0.error(Expected("attribute xsi:type")));
        }
        let v = expect_val!(self.0, AttributeValue, "attribute value");
        let v = try!(self.0.from_utf8(v));
        KeyDeserializer::visit(v)
    }

    /// `visit_unit` is called when deserializing a variant with no values.
    fn visit_unit(&mut self) -> Result<(), Self::Error> {
        expect!(self.0, EmptyElementEnd(_), "empty element end");
        Ok(())
    }

    /// `visit_seq` is called when deserializing a tuple-like variant.
    fn visit_seq<V>(&mut self, _visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        unimplemented!()
    }

    /// `visit_map` is called when deserializing a struct-like variant.
    fn visit_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        try!(self.0.bump());
        visitor.visit_map(ContentVisitor::new_attr(&mut self.0))
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
    de: &'a mut lexer::XmlIterator<Iter>,
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
    fn new_attr(de: &'a mut lexer::XmlIterator<Iter>) -> Self {
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
        use self::ContentVisitorState::*;
        println!("visit_key: {:?}", (&self.state, try!(self.de.ch())));
        match match (&self.state, try!(self.de.ch())) {
            (&Attribute, EmptyElementEnd(_)) => return Ok(None),
            (&Attribute, StartTagClose) => 0,
            (&Attribute, AttributeName(n)) => return Ok(Some(try!(KeyDeserializer::visit(try!(self.de.from_utf8(n)))))),
            (&Element, StartTagName(n)) => return Ok(Some(try!(KeyDeserializer::visit(try!(self.de.from_utf8(n)))))),
            (&Inner, Text(_)) => 1,
            (&Inner, _) => 4,
            (&Value, EndTagName(_)) => return Ok(None),
            (&Value, Text(txt)) if (|| txt.iter().all(|&c| b" \t\n\r".contains(&c)))() => 3,
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
                // hack for Inner, Text
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
            3 => match KeyDeserializer::value_map() {
                Err(Error::UnknownField(_)) => {
                    try!(self.de.bump());
                    Ok(None)
                },
                Err(e) => Err(e),
                Ok(x) => Ok(Some(x)),
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
        use self::ContentVisitorState::*;
        println!("visit_value: {:?}", &self.state);
        match self.state {
            Attribute => {
                let v = {
                    let v = expect_val!(self.de, AttributeValue, "attribute value");
                    let v = try!(self.de.from_utf8(v));
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
                        _ => return Err(self.de.expected("tag close")),
                    }
                    try!(self.de.bump());
                }
                Ok(v)
            },
            Value => {
                let v = {
                    let v = is_val!(self.de, Text, "text");
                    let v = try!(self.de.from_utf8(v));
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
    de: &'a mut lexer::XmlIterator<Iter>,
    done: bool,
}

impl<'a, Iter> SeqVisitor<'a, Iter>
    where Iter: Iterator<Item=u8>,
{
    fn new(de: &'a mut lexer::XmlIterator<Iter>) -> Self {
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
        println!("SeqVisitor::visit: {:?}", (self.done, self.de.ch()));
        if self.done {
            return Ok(None);
        }
        let (is_seq, v) = InnerDeserializer::decode(&mut self.de);
        let v = try!(v);
        if is_seq {
            return Err(self.de.error(XmlDoesntSupportSeqofSeq));
        }
        match try!(self.de.ch()) {
            EndTagName(_) | EmptyElementEnd(_) => {},
            _ => return Err(self.de.expected("end tag")),
        }
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
