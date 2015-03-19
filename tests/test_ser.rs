#![feature(custom_derive, plugin, test)]
#![plugin(serde_macros)]

extern crate test;
extern crate serde;

use std::vec;
use std::collections::BTreeMap;

use serde::ser::{Serialize, Serializer, SeqVisitor, MapVisitor};

#[derive(Clone, PartialEq, Debug)]
pub enum Token<'a> {
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
    Str(&'a str),

    Option(bool),

    Unit,
    NamedUnit(&'a str),
    EnumUnit(&'a str, &'a str),

    SeqStart(Option<usize>),
    NamedSeqStart(&'a str, Option<usize>),
    EnumSeqStart(&'a str, &'a str, Option<usize>),
    SeqSep,
    SeqEnd,

    MapStart(Option<usize>),
    NamedMapStart(&'a str, Option<usize>),
    EnumMapStart(&'a str, &'a str, Option<usize>),
    MapSep,
    MapEnd,
}

struct AssertSerializer<'a> {
    iter: vec::IntoIter<Token<'a>>,
}

impl<'a> AssertSerializer<'a> {
    fn new(values: Vec<Token<'a>>) -> AssertSerializer {
        AssertSerializer {
            iter: values.into_iter(),
        }
    }

    fn visit_sequence<V>(&mut self, mut visitor: V) -> Result<(), ()>
        where V: SeqVisitor
    {
        while let Some(()) = try!(visitor.visit(self)) { }

        assert_eq!(self.iter.next(), Some(Token::SeqEnd));

        Ok(())
    }

    fn visit_mapping<V>(&mut self, mut visitor: V) -> Result<(), ()>
        where V: MapVisitor
    {
        while let Some(()) = try!(visitor.visit(self)) { }

        assert_eq!(self.iter.next(), Some(Token::MapEnd));

        Ok(())
    }
}

impl<'a> Serializer for AssertSerializer<'a> {
    type Error = ();

    fn visit_unit(&mut self) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::Unit));
        Ok(())
    }

    fn visit_named_unit(&mut self, name: &str) -> Result<(), ()> {
        assert_eq!(self.iter.next().unwrap(), Token::NamedUnit(name));
        Ok(())
    }

    fn visit_enum_unit(&mut self, name: &str, variant: &str) -> Result<(), ()> {
        assert_eq!(self.iter.next().unwrap(), Token::EnumUnit(name, variant));
        Ok(())
    }

    fn visit_bool(&mut self, v: bool) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::Bool(v)));
        Ok(())
    }

    fn visit_isize(&mut self, v: isize) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::Isize(v)));
        Ok(())
    }

    fn visit_i8(&mut self, v: i8) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::I8(v)));
        Ok(())
    }

    fn visit_i16(&mut self, v: i16) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::I16(v)));
        Ok(())
    }

    fn visit_i32(&mut self, v: i32) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::I32(v)));
        Ok(())
    }

    fn visit_i64(&mut self, v: i64) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::I64(v)));
        Ok(())
    }

    fn visit_usize(&mut self, v: usize) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::Usize(v)));
        Ok(())
    }

    fn visit_u8(&mut self, v: u8) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::U8(v)));
        Ok(())
    }

    fn visit_u16(&mut self, v: u16) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::U16(v)));
        Ok(())
    }

    fn visit_u32(&mut self, v: u32) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::U32(v)));
        Ok(())
    }

    fn visit_u64(&mut self, v: u64) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::U64(v)));
        Ok(())
    }

    fn visit_f32(&mut self, v: f32) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::F32(v)));
        Ok(())
    }

    fn visit_f64(&mut self, v: f64) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::F64(v)));
        Ok(())
    }

    fn visit_char(&mut self, v: char) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::Char(v)));
        Ok(())
    }

    fn visit_str(&mut self, v: &str) -> Result<(), ()> {
        assert_eq!(self.iter.next().unwrap(), Token::Str(v));
        Ok(())
    }

    fn visit_none(&mut self) -> Result<(), ()> {
        assert_eq!(self.iter.next(), Some(Token::Option(false)));
        Ok(())
    }

    fn visit_some<V>(&mut self, value: V) -> Result<(), ()>
        where V: Serialize,
    {
        assert_eq!(self.iter.next(), Some(Token::Option(true)));
        value.serialize(self)
    }


    fn visit_seq<V>(&mut self, visitor: V) -> Result<(), ()>
        where V: SeqVisitor
    {
        let len = visitor.len();

        assert_eq!(self.iter.next(), Some(Token::SeqStart(len)));

        self.visit_sequence(visitor)
    }

    fn visit_named_seq<V>(&mut self, name: &str, visitor: V) -> Result<(), ()>
        where V: SeqVisitor
    {
        let len = visitor.len();

        assert_eq!(self.iter.next().unwrap(), Token::NamedSeqStart(name, len));

        self.visit_sequence(visitor)
    }

    fn visit_enum_seq<V>(&mut self,
                         name: &str,
                         variant: &str,
                         visitor: V) -> Result<(), ()>
        where V: SeqVisitor
    {
        let len = visitor.len();

        assert_eq!(self.iter.next().unwrap(), Token::EnumSeqStart(name, variant, len));

        self.visit_sequence(visitor)
    }

    fn visit_seq_elt<T>(&mut self, value: T) -> Result<(), ()>
        where T: Serialize
    {
        assert_eq!(self.iter.next(), Some(Token::SeqSep));
        value.serialize(self)
    }

    fn visit_map<V>(&mut self, visitor: V) -> Result<(), ()>
        where V: MapVisitor
    {
        let len = visitor.len();

        assert_eq!(self.iter.next(), Some(Token::MapStart(len)));

        self.visit_mapping(visitor)
    }

    fn visit_named_map<V>(&mut self, name: &str, visitor: V) -> Result<(), ()>
        where V: MapVisitor
    {
        let len = visitor.len();

        assert_eq!(self.iter.next().unwrap(), Token::NamedMapStart(name, len));

        self.visit_mapping(visitor)
    }

    fn visit_enum_map<V>(&mut self, name: &str, variant: &str, visitor: V) -> Result<(), ()>
        where V: MapVisitor
    {
        let len = visitor.len();

        assert_eq!(self.iter.next().unwrap(), Token::EnumMapStart(name, variant, len));

        self.visit_mapping(visitor)
    }

    fn visit_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), ()>
        where K: Serialize,
              V: Serialize,
    {
        assert_eq!(self.iter.next(), Some(Token::MapSep));

        try!(key.serialize(self));
        value.serialize(self)
    }
}

//////////////////////////////////////////////////////////////////////////

#[derive(Serialize)]
struct NamedUnit;

#[derive(Serialize)]
struct NamedSeq(i32, i32, i32);

#[derive(Serialize)]
struct NamedMap {
    a: i32,
    b: i32,
    c: i32,
}

#[derive(Serialize)]
enum Enum {
    Unit,
    Seq(i32, i32),
    Map { a: i32, b: i32 },
}

//////////////////////////////////////////////////////////////////////////

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

macro_rules! declare_test {
    ($name:ident { $($value:expr => $tokens:expr,)+ }) => {
        #[test]
        fn $name() {
            $(
                let mut ser = AssertSerializer::new($tokens);
                assert_eq!($value.serialize(&mut ser), Ok(()));
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

declare_tests! {
    test_unit {
        () => vec![Token::Unit],
    }
    test_bool {
        true => vec![Token::Bool(true)],
        false => vec![Token::Bool(false)],
    }
    test_isizes {
        0isize => vec![Token::Isize(0)],
        0i8 => vec![Token::I8(0)],
        0i16 => vec![Token::I16(0)],
        0i32 => vec![Token::I32(0)],
        0i64 => vec![Token::I64(0)],
    }
    test_usizes {
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
    }
    test_str {
        "abc" => vec![Token::Str("abc")],
        "abc".to_string() => vec![Token::Str("abc")],
    }
    test_option {
        None::<i32> => vec![Token::Option(false)],
        Some(1) => vec![
            Token::Option(true),
            Token::I32(1),
        ],
    }
    test_slice {
        &[0][..0] => vec![
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        &[1, 2, 3][..] => vec![
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_vec {
        Vec::<isize>::new() => vec![
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        vec![vec![], vec![1], vec![2, 3]] => vec![
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::SeqStart(Some(0)),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(1)),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(2)),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
    }
    test_tuple {
        (1,) => vec![
            Token::SeqStart(Some(1)),
                Token::SeqSep,
                Token::I32(1),
            Token::SeqEnd,
        ],
        (1, 2, 3) => vec![
            Token::SeqStart(Some(3)),
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
        btreemap![1 => 2] => vec![
            Token::MapStart(Some(1)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        btreemap![1 => 2, 3 => 4] => vec![
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),

                Token::MapSep,
                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]] => vec![
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::MapStart(Some(0)),
                Token::MapEnd,

                Token::MapSep,
                Token::I32(2),
                Token::MapStart(Some(2)),
                    Token::MapSep,
                    Token::I32(3),
                    Token::I32(4),

                    Token::MapSep,
                    Token::I32(5),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
    }
    test_named_unit {
        NamedUnit => vec![Token::NamedUnit("NamedUnit")],
    }
    test_named_seq {
        NamedSeq(1, 2, 3) => vec![
            Token::NamedSeqStart("NamedSeq", Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_named_map {
        NamedMap { a: 1, b: 2, c: 3 } => vec![
            Token::NamedMapStart("NamedMap", Some(3)),
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
    test_enum {
        Enum::Unit => vec![Token::EnumUnit("Enum", "Unit")],
        Enum::Seq(1, 2) => vec![
            Token::EnumSeqStart("Enum", "Seq", Some(2)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),
            Token::SeqEnd,
        ],
        Enum::Map { a: 1, b: 2 } => vec![
            Token::EnumMapStart("Enum", "Map", Some(2)),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("b"),
                Token::I32(2),
            Token::MapEnd,
        ],
    }
}
