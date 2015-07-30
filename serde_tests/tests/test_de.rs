use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::iter;
use std::vec;

use serde::de::{self, Deserialize, Deserializer, Visitor};

#[derive(Debug)]
enum Token {
    Bool(bool),
    Isize(isize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Usize(usize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    Str(&'static str),
    String(String),

    Option(bool),

    Name(&'static str),

    Unit,

    SeqStart(usize),
    SeqSep,
    SeqEnd,

    MapStart(usize),
    MapSep,
    MapEnd,

    EnumStart(&'static str),
    EnumUnit,
    EnumSimple,
    EnumSeq,
    EnumMap,
    EnumEnd,
}

struct TokenDeserializer {
    tokens: iter::Peekable<vec::IntoIter<Token>>,
}

impl<'a> TokenDeserializer {
    fn new(tokens: Vec<Token>) -> TokenDeserializer {
        TokenDeserializer {
            tokens: tokens.into_iter().peekable(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
enum Error {
    SyntaxError,
    EndOfStreamError,
    UnknownFieldError(String),
    MissingFieldError(&'static str),
    InvalidName(&'static str),
}

impl de::Error for Error {
    fn syntax_error() -> Error { Error::SyntaxError }

    fn end_of_stream_error() -> Error { Error::EndOfStreamError }

    fn unknown_field_error(field: &str) -> Error {
        Error::UnknownFieldError(field.to_string())
    }

    fn missing_field_error(field: &'static str) -> Error {
        Error::MissingFieldError(field)
    }
}

impl Deserializer for TokenDeserializer {
    type Error = Error;

    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        match self.tokens.next() {
            Some(Token::Bool(v)) => visitor.visit_bool(v),
            Some(Token::Isize(v)) => visitor.visit_isize(v),
            Some(Token::I8(v)) => visitor.visit_i8(v),
            Some(Token::I16(v)) => visitor.visit_i16(v),
            Some(Token::I32(v)) => visitor.visit_i32(v),
            Some(Token::I64(v)) => visitor.visit_i64(v),
            Some(Token::Usize(v)) => visitor.visit_usize(v),
            Some(Token::U8(v)) => visitor.visit_u8(v),
            Some(Token::U16(v)) => visitor.visit_u16(v),
            Some(Token::U32(v)) => visitor.visit_u32(v),
            Some(Token::U64(v)) => visitor.visit_u64(v),
            Some(Token::F32(v)) => visitor.visit_f32(v),
            Some(Token::F64(v)) => visitor.visit_f64(v),
            Some(Token::Char(v)) => visitor.visit_char(v),
            Some(Token::Str(v)) => visitor.visit_str(v),
            Some(Token::String(v)) => visitor.visit_string(v),
            Some(Token::Option(false)) => visitor.visit_none(),
            Some(Token::Option(true)) => visitor.visit_some(self),
            Some(Token::Unit) => visitor.visit_unit(),
            Some(Token::SeqStart(len)) => {
                visitor.visit_seq(TokenDeserializerSeqVisitor {
                    de: self,
                    len: len,
                })
            }
            Some(Token::MapStart(len)) => {
                visitor.visit_map(TokenDeserializerMapVisitor {
                    de: self,
                    len: len,
                })
            }
            Some(Token::Name(_)) => self.visit(visitor),
            Some(_) => Err(Error::SyntaxError),
            None => Err(Error::EndOfStreamError),
        }
    }

    /// Hook into `Option` deserializing so we can treat `Unit` as a
    /// `None`, or a regular value as `Some(value)`.
    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::Option(false)) => {
                self.tokens.next();
                visitor.visit_none()
            }
            Some(&Token::Option(true)) => {
                self.tokens.next();
                visitor.visit_some(self)
            }
            Some(&Token::Unit) => {
                self.tokens.next();
                visitor.visit_none()
            }
            Some(_) => visitor.visit_some(self),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_enum<V>(&mut self,
                     name: &str,
                     _variants: &'static [&'static str],
                     mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        match self.tokens.next() {
            Some(Token::EnumStart(n)) => {
                if name == n {
                    visitor.visit(TokenDeserializerVariantVisitor {
                        de: self,
                    })
                } else {
                    Err(Error::SyntaxError)
                }
            }
            Some(_) => Err(Error::SyntaxError),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_unit_struct<V>(&mut self, name: &str, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::Name(n)) => {
                if name == n {
                    self.tokens.next();
                    self.visit_seq(visitor)
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(_) => self.visit(visitor),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_tuple_struct<V>(&mut self,
                             name: &str,
                             _len: usize,
                             visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::Name(n)) => {
                if name == n {
                    self.tokens.next();
                    self.visit_seq(visitor)
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(_) => self.visit_seq(visitor),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_struct<V>(&mut self,
                       name: &str,
                       _fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::Name(n)) => {
                if name == n {
                    self.tokens.next();
                    self.visit_map(visitor)
                } else {
                    Err(Error::InvalidName(n))
                }
            }
            Some(_) => self.visit_map(visitor),
            None => Err(Error::EndOfStreamError),
        }
    }
}

//////////////////////////////////////////////////////////////////////////

struct TokenDeserializerSeqVisitor<'a> {
    de: &'a mut TokenDeserializer,
    len: usize,
}

impl<'a> de::SeqVisitor for TokenDeserializerSeqVisitor<'a> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::SeqSep) => {
                self.len -= 1;
                self.de.tokens.next();
                Ok(Some(try!(Deserialize::deserialize(self.de))))
            }
            Some(&Token::SeqEnd) => Ok(None),
            Some(_) => {
                Err(Error::SyntaxError)
            }
            None => Err(Error::EndOfStreamError),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        assert_eq!(self.len, 0);
        match self.de.tokens.next() {
            Some(Token::SeqEnd) => Ok(()),
            Some(_) => Err(Error::SyntaxError),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

//////////////////////////////////////////////////////////////////////////

struct TokenDeserializerMapVisitor<'a> {
    de: &'a mut TokenDeserializer,
    len: usize,
}

impl<'a> de::MapVisitor for TokenDeserializerMapVisitor<'a> {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: Deserialize,
    {
        match self.de.tokens.peek() {
            Some(&Token::MapSep) => {
                self.de.tokens.next();
                self.len -= 1;
                Ok(Some(try!(Deserialize::deserialize(self.de))))
            }
            Some(&Token::MapEnd) => Ok(None),
            Some(_) => Err(Error::SyntaxError),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: Deserialize,
    {
        Ok(try!(Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<(), Error> {
        assert_eq!(self.len, 0);
        match self.de.tokens.next() {
            Some(Token::MapEnd) => Ok(()),
            Some(_) => Err(Error::SyntaxError),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

//////////////////////////////////////////////////////////////////////////

struct TokenDeserializerVariantVisitor<'a> {
    de: &'a mut TokenDeserializer,
}

impl<'a> de::VariantVisitor for TokenDeserializerVariantVisitor<'a> {
    type Error = Error;

    fn visit_variant<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        de::Deserialize::deserialize(self.de)
    }

    fn visit_unit(&mut self) -> Result<(), Error> {
        match self.de.tokens.next() {
            Some(Token::EnumUnit) => {
                de::Deserialize::deserialize(self.de)
            }
            Some(_) => Err(Error::SyntaxError),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_simple<T>(&mut self) -> Result<T, Self::Error>
        where T: de::Deserialize,
    {
        match self.de.tokens.next() {
            Some(Token::EnumSimple) => {
                de::Deserialize::deserialize(self.de)
            }
            Some(_) => Err(Error::SyntaxError),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_tuple<V>(&mut self,
                      _len: usize,
                      visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.de.tokens.next() {
            Some(Token::EnumSeq) => {
                de::Deserializer::visit(self.de, visitor)
            }
            Some(_) => Err(Error::SyntaxError),
            None => Err(Error::EndOfStreamError),
        }
    }

    fn visit_struct<V>(&mut self,
                       _fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.de.tokens.next() {
            Some(Token::EnumMap) => {
                de::Deserializer::visit(self.de, visitor)
            }
            Some(_) => Err(Error::SyntaxError),
            None => Err(Error::EndOfStreamError),
        }
    }
}

//////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
struct UnitStruct;

#[derive(PartialEq, Debug, Deserialize)]
struct TupleStruct(i32, i32, i32);

#[derive(PartialEq, Debug, Deserialize)]
struct Struct {
    a: i32,
    b: i32,
    c: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
enum Enum {
    Unit,
    Simple(i32),
    Seq(i32, i32, i32),
    Map { a: i32, b: i32, c: i32 }
}

//////////////////////////////////////////////////////////////////////////

macro_rules! btreeset {
    () => {
        BTreeSet::new()
    };
    ($($value:expr),+) => {
        {
            let mut set = BTreeSet::new();
            $(set.insert($value);)+
            set
        }
    }
}

macro_rules! btreemap {
    () => {
        BTreeMap::new()
    };
    ($($key:expr => $value:expr),+) => {
        {
            let mut map = BTreeMap::new();
            $(map.insert($key, $value);)+
            map
        }
    }
}

macro_rules! hashset {
    () => {
        HashSet::new()
    };
    ($($value:expr),+) => {
        {
            let mut set = HashSet::new();
            $(set.insert($value);)+
            set
        }
    }
}

macro_rules! hashmap {
    () => {
        HashMap::new()
    };
    ($($key:expr => $value:expr),+) => {
        {
            let mut map = HashMap::new();
            $(map.insert($key, $value);)+
            map
        }
    }
}

macro_rules! declare_test {
    ($name:ident { $($value:expr => $tokens:expr,)+ }) => {
        #[test]
        fn $name() {
            $(
                let mut de = TokenDeserializer::new($tokens);
                let value: Result<_, Error> = Deserialize::deserialize(&mut de);
                assert_eq!(value, Ok($value));
            )+
        }
    }
}

macro_rules! declare_tests {
    ($($name:ident { $($value:expr => $tokens:expr,)+ })+) => {
        $(
            declare_test!($name { $($value => $tokens,)+ });
        )+
    }
}

//////////////////////////////////////////////////////////////////////////

declare_tests! {
    test_bool {
        true => vec![Token::Bool(true)],
        false => vec![Token::Bool(false)],
    }
    test_isize {
        0isize => vec![Token::Isize(0)],
        0isize => vec![Token::I8(0)],
        0isize => vec![Token::I16(0)],
        0isize => vec![Token::I32(0)],
        0isize => vec![Token::I64(0)],
        0isize => vec![Token::Usize(0)],
        0isize => vec![Token::U8(0)],
        0isize => vec![Token::U16(0)],
        0isize => vec![Token::U32(0)],
        0isize => vec![Token::U64(0)],
        0isize => vec![Token::F32(0.)],
        0isize => vec![Token::F64(0.)],
    }
    test_ints {
        0isize => vec![Token::Isize(0)],
        0i8 => vec![Token::I8(0)],
        0i16 => vec![Token::I16(0)],
        0i32 => vec![Token::I32(0)],
        0i64 => vec![Token::I64(0)],
    }
    test_uints {
        0usize => vec![Token::Usize(0)],
        0u8 => vec![Token::U8(0)],
        0u16 => vec![Token::U16(0)],
        0u32 => vec![Token::U32(0)],
        0u64 => vec![Token::U64(0)],
    }
    test_floats {
        0f32 => vec![Token::F32(0.)],
        0f64 => vec![Token::F64(0.)],
    }
    test_char {
        'a' => vec![Token::Char('a')],
        'a' => vec![Token::Str("a")],
        'a' => vec![Token::String("a".to_string())],
    }
    test_string {
        "abc".to_string() => vec![Token::Str("abc")],
        "abc".to_string() => vec![Token::String("abc".to_string())],
        "a".to_string() => vec![Token::Char('a')],
    }
    test_option {
        None::<i32> => vec![Token::Unit],
        None::<i32> => vec![Token::Option(false)],
        Some(1) => vec![Token::I32(1)],
        Some(1) => vec![
            Token::Option(true),
            Token::I32(1),
        ],
    }
    test_result {
        Ok::<i32, i32>(0) => vec![
            Token::EnumStart("Result"),
                Token::Str("Ok"),

                Token::EnumSimple,
                Token::I32(0),
            Token::SeqEnd,
        ],
        Err::<i32, i32>(1) => vec![
            Token::EnumStart("Result"),
                Token::Str("Err"),

                Token::EnumSimple,
                Token::I32(1),
            Token::SeqEnd,
        ],
    }
    test_unit {
        () => vec![Token::Unit],
        () => vec![
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
        () => vec![
            Token::Name("Anything"),
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
    }
    test_unit_struct {
        UnitStruct => vec![Token::Unit],
        UnitStruct => vec![
            Token::Name("UnitStruct"),
            Token::Unit,
        ],
        UnitStruct => vec![
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
    }
    test_tuple_struct {
        TupleStruct(1, 2, 3) => vec![
            Token::SeqStart(3),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
        TupleStruct(1, 2, 3) => vec![
            Token::Name("TupleStruct"),
            Token::SeqStart(3),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_btreeset {
        BTreeSet::<isize>::new() => vec![
            Token::Unit,
        ],
        BTreeSet::<isize>::new() => vec![
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
        btreeset![btreeset![], btreeset![1], btreeset![2, 3]] => vec![
            Token::SeqStart(3),
                Token::SeqSep,
                Token::SeqStart(0),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(1),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(2),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        BTreeSet::<isize>::new() => vec![
            Token::Name("Anything"),
            Token::Unit,
        ],
        BTreeSet::<isize>::new() => vec![
            Token::Name("Anything"),
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
    }
    test_hashset {
        HashSet::<isize>::new() => vec![
            Token::Unit,
        ],
        HashSet::<isize>::new() => vec![
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
        hashset![1, 2, 3] => vec![
            Token::SeqStart(3),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
        HashSet::<isize>::new() => vec![
            Token::Name("Anything"),
            Token::Unit,
        ],
        HashSet::<isize>::new() => vec![
            Token::Name("Anything"),
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
    }
    test_vec {
        Vec::<isize>::new() => vec![
            Token::Unit,
        ],
        Vec::<isize>::new() => vec![
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
        vec![vec![], vec![1], vec![2, 3]] => vec![
            Token::SeqStart(3),
                Token::SeqSep,
                Token::SeqStart(0),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(1),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(2),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        Vec::<isize>::new() => vec![
            Token::Name("Anything"),
            Token::Unit,
        ],
        Vec::<isize>::new() => vec![
            Token::Name("Anything"),
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
    }
    test_array {
        [0; 0] => vec![
            Token::Unit,
        ],
        [0; 0] => vec![
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
        ([0; 0], [1], [2, 3]) => vec![
            Token::SeqStart(3),
                Token::SeqSep,
                Token::SeqStart(0),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(1),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(2),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        [0; 0] => vec![
            Token::Name("Anything"),
            Token::Unit,
        ],
        [0; 0] => vec![
            Token::Name("Anything"),
            Token::SeqStart(0),
            Token::SeqEnd,
        ],
    }
    test_tuple {
        (1,) => vec![
            Token::SeqStart(1),
                Token::SeqSep,
                Token::I32(1),
            Token::SeqEnd,
        ],
        (1, 2, 3) => vec![
            Token::SeqStart(3),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_btreemap {
        BTreeMap::<isize, isize>::new() => vec![
            Token::Unit,
        ],
        BTreeMap::<isize, isize>::new() => vec![
            Token::MapStart(0),
            Token::MapEnd,
        ],
        btreemap![1 => 2] => vec![
            Token::MapStart(1),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        btreemap![1 => 2, 3 => 4] => vec![
            Token::MapStart(2),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),

                Token::MapSep,
                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]] => vec![
            Token::MapStart(2),
                Token::MapSep,
                Token::I32(1),
                Token::MapStart(0),
                Token::MapEnd,

                Token::MapSep,
                Token::I32(2),
                Token::MapStart(2),
                    Token::MapSep,
                    Token::I32(3),
                    Token::I32(4),

                    Token::MapSep,
                    Token::I32(5),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
        BTreeMap::<isize, isize>::new() => vec![
            Token::Name("Anything"),
            Token::Unit,
        ],
        BTreeMap::<isize, isize>::new() => vec![
            Token::Name("Anything"),
            Token::MapStart(0),
            Token::MapEnd,
        ],
    }
    test_hashmap {
        HashMap::<isize, isize>::new() => vec![
            Token::Unit,
        ],
        HashMap::<isize, isize>::new() => vec![
            Token::MapStart(0),
            Token::MapEnd,
        ],
        hashmap![1 => 2] => vec![
            Token::MapStart(1),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        hashmap![1 => 2, 3 => 4] => vec![
            Token::MapStart(2),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),

                Token::MapSep,
                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        hashmap![1 => hashmap![], 2 => hashmap![3 => 4, 5 => 6]] => vec![
            Token::MapStart(2),
                Token::MapSep,
                Token::I32(1),
                Token::MapStart(0),
                Token::MapEnd,

                Token::MapSep,
                Token::I32(2),
                Token::MapStart(2),
                    Token::MapSep,
                    Token::I32(3),
                    Token::I32(4),

                    Token::MapSep,
                    Token::I32(5),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
        HashMap::<isize, isize>::new() => vec![
            Token::Name("Anything"),
            Token::Unit,
        ],
        HashMap::<isize, isize>::new() => vec![
            Token::Name("Anything"),
            Token::MapStart(0),
            Token::MapEnd,
        ],
    }
    test_struct {
        Struct { a: 1, b: 2, c: 3 } => vec![
            Token::MapStart(3),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("b"),
                Token::I32(2),

                Token::MapSep,
                Token::Str("c"),
                Token::I32(3),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 3 } => vec![
            Token::Name("Struct"),
            Token::MapStart(3),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("b"),
                Token::I32(2),

                Token::MapSep,
                Token::Str("c"),
                Token::I32(3),
            Token::MapEnd,
        ],
    }
    test_enum_unit {
        Enum::Unit => vec![
            Token::EnumStart("Enum"),
                Token::Str("Unit"),

                Token::EnumUnit,
                Token::Unit,
            Token::EnumEnd,
        ],
    }
    test_enum_simple {
        Enum::Simple(1) => vec![
            Token::EnumStart("Enum"),
                Token::Str("Simple"),

                Token::EnumSimple,
                Token::I32(1),
            Token::EnumEnd,
        ],
    }
    test_enum_seq {
        Enum::Seq(1, 2, 3) => vec![
            Token::EnumStart("Enum"),
                Token::Str("Seq"),

                Token::EnumSeq,
                Token::SeqStart(3),
                    Token::SeqSep,
                    Token::I32(1),

                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::EnumEnd,
        ],
    }
    test_enum_map {
        Enum::Map { a: 1, b: 2, c: 3 } => vec![
            Token::EnumStart("Enum"),
                Token::Str("Map"),

                Token::EnumMap,
                Token::MapStart(3),
                    Token::MapSep,
                    Token::Str("a"),
                    Token::I32(1),

                    Token::MapSep,
                    Token::Str("b"),
                    Token::I32(2),

                    Token::MapSep,
                    Token::Str("c"),
                    Token::I32(3),
                Token::MapEnd,
            Token::EnumEnd,
        ],
    }
}
