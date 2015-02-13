#![feature(plugin)]
#![plugin(serde_macros)]

extern crate serde;

use std::collections::BTreeMap;
use std::{option, string};

use serde::de::{Deserializer, Deserialize, Token, TokenKind, IgnoreTokens};

macro_rules! treemap {
    ($($k:expr => $v:expr),*) => ({
        let mut _m = ::std::collections::BTreeMap::new();
        $(_m.insert($k, $v);)*
        _m
    })
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug)]
struct Inner {
    a: (),
    b: usize,
    c: BTreeMap<string::String, option::Option<char>>,
}

impl<
    D: Deserializer<E>,
    E
> Deserialize<D, E> for Inner {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Inner, E> {
        try!(d.expect_struct_start(token, "Inner"));

        let mut a = None;
        let mut b = None;
        let mut c = None;

        static FIELDS: &'static [&'static str] = &["a", "b", "c"];

        loop {
            let idx = match try!(d.expect_struct_field_or_end(FIELDS)) {
                Some(idx) => idx,
                None => { break; }
            };

            match idx {
                Some(0) => { a = Some(try!(d.expect_struct_value())); }
                Some(1) => { b = Some(try!(d.expect_struct_value())); }
                Some(2) => { c = Some(try!(d.expect_struct_value())); }
                Some(_) => unreachable!(),
                None => { let _: IgnoreTokens = try!(Deserialize::deserialize(d)); }
            }
        }

        Ok(Inner { a: a.unwrap(), b: b.unwrap(), c: c.unwrap() })
    }
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug)]
struct Outer {
    inner: Vec<Inner>,
}

impl<D: Deserializer<E>, E> Deserialize<D, E> for Outer {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Outer, E> {
        try!(d.expect_struct_start(token, "Outer"));

        static FIELDS: &'static [&'static str] = &["inner"];

        let mut inner = None;

        loop {
            let idx = match try!(d.expect_struct_field_or_end(FIELDS)) {
                Some(idx) => idx,
                None => { break; }
            };

            match idx {
                Some(0) => { inner = Some(try!(d.expect_struct_value())); }
                Some(_) => unreachable!(),
                None => { let _: IgnoreTokens = try!(Deserialize::deserialize(d)); }
            }
        }

        Ok(Outer { inner: inner.unwrap() })
    }
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug)]
enum Animal {
    Dog,
    Frog(string::String, isize)
}

impl<D: Deserializer<E>, E> Deserialize<D, E> for Animal {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Animal, E> {
        match try!(d.expect_enum_start(token, "Animal", &["Dog", "Frog"])) {
            0 => {
                try!(d.expect_enum_end());
                Ok(Animal::Dog)
            }
            1 => {
                let x0 = try!(Deserialize::deserialize(d));
                let x1 = try!(Deserialize::deserialize(d));
                try!(d.expect_enum_end());
                Ok(Animal::Frog(x0, x1))
            }
            _ => unreachable!(),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum Error {
    EndOfStream,
    SyntaxError(Vec<TokenKind>),
    UnexpectedName,
    ConversionError,
    MissingField(&'static str),
}

//////////////////////////////////////////////////////////////////////////////

struct TokenDeserializer<Iter> {
    tokens: Iter,
}

impl<Iter: Iterator<Item=Token>> TokenDeserializer<Iter> {
    #[inline]
    fn new(tokens: Iter) -> TokenDeserializer<Iter> {
        TokenDeserializer {
            tokens: tokens,
        }
    }
}

impl<Iter: Iterator<Item=Token>> Iterator for TokenDeserializer<Iter> {
    type Item = Result<Token, Error>;

    #[inline]
    fn next(&mut self) -> option::Option<Result<Token, Error>> {
        self.tokens.next().map(|token| Ok(token))
    }
}

impl<Iter: Iterator<Item=Token>> Deserializer<Error> for TokenDeserializer<Iter> {
    fn end_of_stream_error(&mut self) -> Error {
        Error::EndOfStream
    }

    fn syntax_error(&mut self, _token: Token, expected: &[TokenKind]) -> Error {
        Error::SyntaxError(expected.to_vec())
    }

    fn unexpected_name_error(&mut self, _token: Token) -> Error {
        Error::UnexpectedName
    }

    fn conversion_error(&mut self, _token: Token) -> Error {
        Error::ConversionError
    }

    #[inline]
    fn missing_field<
        T: Deserialize<TokenDeserializer<Iter>, Error>
    >(&mut self, field: &'static str) -> Result<T, Error> {
        Err(Error::MissingField(field))
    }
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! test_value {
    ($name:ident, [$($tokens:expr => $value:expr, $ty:ty),*]) => {
        #[test]
        fn $name() {
            $(
                let mut deserializer = TokenDeserializer::new($tokens.into_iter());
                let value: $ty = Deserialize::deserialize(&mut deserializer).unwrap();

                assert_eq!(value, $value);
            )+
        }
    }
}

test_value!(test_primitives, [
    vec!(Token::Null) => (), (),
    vec!(Token::Bool(true)) => true, bool,
    vec!(Token::Bool(false)) => false, bool,
    vec!(Token::Isize(5)) => 5, isize,
    vec!(Token::I8(5)) => 5, i8,
    vec!(Token::I16(5)) => 5, i16,
    vec!(Token::I32(5)) => 5, i32,
    vec!(Token::I64(5)) => 5, i64,
    vec!(Token::Usize(5)) => 5, usize,
    vec!(Token::U8(5)) => 5, u8,
    vec!(Token::U16(5)) => 5, u16,
    vec!(Token::U32(5)) => 5, u32,
    vec!(Token::U64(5)) => 5, u64,
    vec!(Token::F32(5.0)) => 5.0, f32,
    vec!(Token::F64(5.0)) => 5.0, f64,
    vec!(Token::Char('c')) => 'c', char,
    vec!(Token::Str("abc")) => "abc", &str,
    vec!(Token::String("abc".to_string())) => "abc".to_string(), string::String
]);

test_value!(test_tuples, [
    vec!(
        Token::TupleStart(0),
        Token::End,
    ) => (), (),

    vec!(
        Token::TupleStart(2),
            Token::Isize(5),

            Token::Str("a"),
        Token::End,
    ) => (5, "a"), (isize, &'static str),

    vec!(
        Token::TupleStart(3),
            Token::Null,

            Token::TupleStart(0),
            Token::End,

            Token::TupleStart(2),
                Token::Isize(5),

                Token::Str("a"),
            Token::End,
        Token::End,
    ) => ((), (), (5, "a")), ((), (), (isize, &'static str))
]);

test_value!(test_options, [
    vec!(Token::Option(false)) => None, option::Option<isize>,

    vec!(
        Token::Option(true),
        Token::Isize(5),
    ) => Some(5), option::Option<isize>
]);

test_value!(test_structs, [
    vec!(
        Token::StructStart("Outer", 1),
            Token::Str("inner"),
            Token::SeqStart(0),
            Token::End,
        Token::End,
    ) => Outer { inner: vec!() }, Outer,

    vec!(
        Token::StructStart("Outer", 1),
            Token::Str("inner"),
            Token::SeqStart(1),
                Token::StructStart("Inner", 3),
                    Token::Str("a"),
                    Token::Null,

                    Token::Str("b"),
                    Token::Usize(5),

                    Token::Str("c"),
                    Token::MapStart(1),
                        Token::String("abc".to_string()),

                        Token::Option(true),
                        Token::Char('c'),
                    Token::End,
                Token::End,
            Token::End,
        Token::End,
    ) => Outer {
        inner: vec!(
            Inner {
                a: (),
                b: 5,
                c: treemap!("abc".to_string() => Some('c')),
            },
        ),
    }, Outer
]);

test_value!(test_enums, [
    vec!(
        Token::EnumStart("Animal", "Dog", 0),
        Token::End,
    ) => Animal::Dog, Animal,

    vec!(
        Token::EnumStart("Animal", "Frog", 2),
            Token::String("Henry".to_string()),
            Token::Isize(349),
        Token::End,
    ) => Animal::Frog("Henry".to_string(), 349), Animal
]);

test_value!(test_vecs, [
    vec!(
        Token::SeqStart(0),
        Token::End,
    ) => vec!(), Vec<isize>,

    vec!(
        Token::SeqStart(3),
            Token::Isize(5),

            Token::Isize(6),

            Token::Isize(7),
        Token::End,
    ) => vec!(5, 6, 7), Vec<isize>,


    vec!(
        Token::SeqStart(3),
            Token::SeqStart(1),
                Token::Isize(1),
            Token::End,

            Token::SeqStart(2),
                Token::Isize(2),

                Token::Isize(3),
            Token::End,

            Token::SeqStart(3),
                Token::Isize(4),

                Token::Isize(5),

                Token::Isize(6),
            Token::End,
        Token::End,
    ) => vec!(vec!(1), vec!(2, 3), vec!(4, 5, 6)), Vec<Vec<isize>>
]);

test_value!(test_treemaps, [
    vec!(
        Token::MapStart(0),
        Token::End,
    ) => treemap!(), BTreeMap<isize, string::String>,

    vec!(
        Token::MapStart(2),
            Token::Isize(5),
            Token::String("a".to_string()),

            Token::Isize(6),
            Token::String("b".to_string()),
        Token::End,
    ) => treemap!(5is => "a".to_string(), 6is => "b".to_string()), BTreeMap<isize, string::
    String>
]);
