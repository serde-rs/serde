#![feature(plugin)]
#![plugin(serde_macros)]

extern crate serde;

use std::collections::{HashMap, BTreeMap};
use std::{option, string};

use serde::ser::{Serializer, Serialize};

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug)]
#[derive_serialize]
struct Inner {
    a: (),
    b: usize,
    c: HashMap<string::String, option::Option<char>>,
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug)]
#[derive_serialize]
struct Outer {
    inner: Vec<Inner>,
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug)]
#[derive_serialize]
enum Animal {
    Dog,
    Frog(String, isize)
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug)]
pub enum Token<'a> {
    Null,
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

    TupleStart(usize),
    TupleSep,
    TupleEnd,

    StructStart(&'a str, usize),
    StructSep(&'a str),
    StructEnd,

    EnumStart(&'a str, &'a str, usize),
    EnumSep,
    EnumEnd,

    Option(bool),

    SeqStart(usize),
    SeqEnd,

    MapStart(usize),
    MapEnd,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Error {
    EndOfStream,
    SyntaxError,
}

//////////////////////////////////////////////////////////////////////////////

struct AssertSerializer<Iter> {
    iter: Iter,
}

impl<'a, Iter: Iterator<Item=Token<'a>>> AssertSerializer<Iter> {
    fn new(iter: Iter) -> AssertSerializer<Iter> {
        AssertSerializer {
            iter: iter,
        }
    }

    fn serialize<'b>(&mut self, token: Token<'b>) -> Result<(), Error> {
        let t = match self.iter.next() {
            Some(t) => t,
            None => { panic!(); }
        };

        assert_eq!(t, token);

        Ok(())
    }
}

impl<'a, Iter: Iterator<Item=Token<'a>>> Serializer<Error> for AssertSerializer<Iter> {
    fn serialize_null(&mut self) -> Result<(), Error> {
        self.serialize(Token::Null)
    }
    fn serialize_bool(&mut self, v: bool) -> Result<(), Error> {
        self.serialize(Token::Bool(v))
    }
    fn serialize_isize(&mut self, v: isize) -> Result<(), Error> {
        self.serialize(Token::Isize(v))
    }

    fn serialize_i8(&mut self, v: i8) -> Result<(), Error> {
        self.serialize(Token::I8(v))
    }

    fn serialize_i16(&mut self, v: i16) -> Result<(), Error> {
        self.serialize(Token::I16(v))
    }

    fn serialize_i32(&mut self, v: i32) -> Result<(), Error> {
        self.serialize(Token::I32(v))
    }

    fn serialize_i64(&mut self, v: i64) -> Result<(), Error> {
        self.serialize(Token::I64(v))
    }

    fn serialize_usize(&mut self, v: usize) -> Result<(), Error> {
        self.serialize(Token::Usize(v))
    }

    fn serialize_u8(&mut self, v: u8) -> Result<(), Error> {
        self.serialize(Token::U8(v))
    }

    fn serialize_u16(&mut self, v: u16) -> Result<(), Error> {
        self.serialize(Token::U16(v))
    }

    fn serialize_u32(&mut self, v: u32) -> Result<(), Error> {
        self.serialize(Token::U32(v))
    }

    fn serialize_u64(&mut self, v: u64) -> Result<(), Error> {
        self.serialize(Token::U64(v))
    }

    fn serialize_f32(&mut self, v: f32) -> Result<(), Error> {
        self.serialize(Token::F32(v))
    }

    fn serialize_f64(&mut self, v: f64) -> Result<(), Error> {
        self.serialize(Token::F64(v))
    }

    fn serialize_char(&mut self, v: char) -> Result<(), Error> {
        self.serialize(Token::Char(v))
    }

    fn serialize_str(&mut self, v: &str) -> Result<(), Error> {
        self.serialize(Token::Str(v))
    }

    fn serialize_tuple_start(&mut self, len: usize) -> Result<(), Error> {
        self.serialize(Token::TupleStart(len))
    }

    fn serialize_tuple_elt<
        T: Serialize<AssertSerializer<Iter>, Error>
    >(&mut self, value: &T) -> Result<(), Error> {
        try!(self.serialize(Token::TupleSep));
        value.serialize(self)
    }

    fn serialize_tuple_end(&mut self) -> Result<(), Error> {
        self.serialize(Token::TupleEnd)
    }

    fn serialize_struct_start(&mut self, name: &str, len: usize) -> Result<(), Error> {
        self.serialize(Token::StructStart(name, len))
    }

    fn serialize_struct_elt<
        T: Serialize<AssertSerializer<Iter>, Error>
    >(&mut self, name: &str, value: &T) -> Result<(), Error> {
        try!(self.serialize(Token::StructSep(name)));
        value.serialize(self)
    }

    fn serialize_struct_end(&mut self) -> Result<(), Error> {
        self.serialize(Token::StructEnd)
    }

    fn serialize_enum_start(&mut self, name: &str, variant: &str, len: usize) -> Result<(), Error> {
        self.serialize(Token::EnumStart(name, variant, len))
    }

    fn serialize_enum_elt<
        T: Serialize<AssertSerializer<Iter>, Error>
    >(&mut self, value: &T) -> Result<(), Error> {
        try!(self.serialize(Token::EnumSep));
        value.serialize(self)
    }

    fn serialize_enum_end(&mut self) -> Result<(), Error> {
        self.serialize(Token::EnumEnd)
    }

    fn serialize_option<
        T: Serialize<AssertSerializer<Iter>, Error>
    >(&mut self, v: &option::Option<T>) -> Result<(), Error> {
        match *v {
            Some(ref v) => {
                try!(self.serialize(Token::Option(true)));
                v.serialize(self)
            }
            None => {
                self.serialize(Token::Option(false))
            }
        }
    }

    fn serialize_seq<
        T: Serialize<AssertSerializer<Iter>, Error>,
        SeqIter: Iterator<Item=T>
    >(&mut self, iter: SeqIter) -> Result<(), Error> {
        let (len, _) = iter.size_hint();
        try!(self.serialize(Token::SeqStart(len)));
        for elt in iter {
            try!(elt.serialize(self));
        }
        self.serialize(Token::SeqEnd)
    }

    fn serialize_map<
        K: Serialize<AssertSerializer<Iter>, Error>,
        V: Serialize<AssertSerializer<Iter>, Error>,
        MapIter: Iterator<Item=(K, V)>
    >(&mut self, iter: MapIter) -> Result<(), Error> {
        let (len, _) = iter.size_hint();
        try!(self.serialize(Token::MapStart(len)));
        for (key, value) in iter {
            try!(key.serialize(self));
            try!(value.serialize(self));
        }
        self.serialize(Token::MapEnd)
    }
}

//////////////////////////////////////////////////////////////////////////////

#[test]
fn test_tokens_int() {
    let tokens = vec!(
        Token::Isize(5)
    );
    let mut serializer = AssertSerializer::new(tokens.into_iter());
    5is.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_str() {
    let tokens = vec!(
        Token::Str("a"),
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    "a".serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_null() {
    let tokens = vec!(
        Token::Null,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    ().serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_option_none() {
    let tokens = vec!(
        Token::Option(false),
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    None::<isize>.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_option_some() {
    let tokens = vec!(
        Token::Option(true),
        Token::Isize(5),
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    Some(5is).serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_tuple() {
    let tokens = vec!(
        Token::TupleStart(2),
            Token::TupleSep,
            Token::Isize(5),

            Token::TupleSep,
            Token::Str("a"),
        Token::TupleEnd,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    (5is, "a").serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_tuple_compound() {
    let tokens = vec!(
        Token::TupleStart(3),
            Token::TupleSep,
            Token::Null,

            Token::TupleSep,
            Token::Null,

            Token::TupleSep,
            Token::TupleStart(2),
                Token::TupleSep,
                Token::Isize(5),

                Token::TupleSep,
                Token::Str("a"),
            Token::TupleEnd,
        Token::TupleEnd,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    ((), (), (5is, "a")).serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_struct_empty() {
    let tokens = vec!(
        Token::StructStart("Outer", 1),
            Token::StructSep("inner"),
            Token::SeqStart(0),
            Token::SeqEnd,
        Token::StructEnd,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    Outer { inner: vec!() }.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_struct() {
    let tokens = vec!(
        Token::StructStart("Outer", 1),
            Token::StructSep("inner"),
            Token::SeqStart(1),
                Token::StructStart("Inner", 3),
                    Token::StructSep("a"),
                    Token::Null,

                    Token::StructSep("b"),
                    Token::Usize(5),

                    Token::StructSep("c"),
                    Token::MapStart(1),
                        Token::Str("abc"),
                        Token::Option(true),
                        Token::Char('c'),
                    Token::MapEnd,
                Token::StructEnd,
            Token::SeqEnd,
        Token::StructEnd,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());

    let mut map = HashMap::new();
    map.insert("abc".to_string(), Some('c'));

    Outer {
        inner: vec!(
            Inner {
                a: (),
                b: 5,
                c: map,
            },
        )
    }.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_enum() {
    let tokens = vec!(
        Token::EnumStart("Animal", "Dog", 0),
        Token::EnumEnd,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    Animal::Dog.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);

    let tokens = vec!(
        Token::EnumStart("Animal", "Frog", 2),
            Token::EnumSep,
            Token::Str("Henry"),

            Token::EnumSep,
            Token::Isize(349),
        Token::EnumEnd,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    Animal::Frog("Henry".to_string(), 349).serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_vec_empty() {
    let tokens = vec!(
        Token::SeqStart(0),
        Token::SeqEnd,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    let v: Vec<isize> = vec!();
    v.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_vec() {
    let tokens = vec!(
        Token::SeqStart(3),
            Token::Isize(5),
            Token::Isize(6),
            Token::Isize(7),
        Token::SeqEnd,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    (vec!(5is, 6, 7)).serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_vec_compound() {
    let tokens = vec!(
        Token::SeqStart(3),
            Token::SeqStart(1),
                Token::Isize(1),
            Token::SeqEnd,

            Token::SeqStart(2),
                Token::Isize(2),
                Token::Isize(3),
            Token::SeqEnd,

            Token::SeqStart(3),
                Token::Isize(4),
                Token::Isize(5),
                Token::Isize(6),
            Token::SeqEnd,
        Token::SeqEnd,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());
    (vec!(vec!(1is), vec!(2, 3), vec!(4, 5, 6))).serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}

#[test]
fn test_tokens_treemap() {
    let tokens = vec!(
        Token::MapStart(2),
            Token::Isize(5),
            Token::Str("a"),

            Token::Isize(6),
            Token::Str("b"),
        Token::MapEnd,
    );

    let mut serializer = AssertSerializer::new(tokens.into_iter());

    let mut map = BTreeMap::new();
    map.insert(5is, "a".to_string());
    map.insert(6is, "b".to_string());

    map.serialize(&mut serializer).unwrap();
    assert_eq!(serializer.iter.next(), None);
}
