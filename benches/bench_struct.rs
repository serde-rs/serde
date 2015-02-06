#![feature(plugin)]

#[plugin]
extern crate serde_macros;

extern crate serde;
extern crate "rustc-serialize" as rustc_serialize;
extern crate test;

use std::collections::HashMap;
use test::Bencher;

use rustc_serialize::{Decoder, Decodable};

use serde::de::{Deserializer, Deserialize};

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug, RustcDecodable)]
#[derive_deserialize]
struct Inner {
    a: (),
    b: usize,
    c: HashMap<String, Option<char>>,
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug, RustcDecodable)]
#[derive_deserialize]
struct Outer {
    inner: Vec<Inner>,
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Error {
    EndOfStream,
    SyntaxError(String),
    UnexpectedName(String),
    ConversionError(String),
    MissingField(&'static str),
    OtherError(String),
}

mod decoder {
    use std::collections::HashMap;
    use rustc_serialize::Decoder;

    use super::{Outer, Inner, Error};

    use self::State::{
        OuterState,
        InnerState,
        NullState,
        UsizeState,
        CharState,
        StringState,
        FieldState,
        VecState,
        MapState,
        OptionState,
    };

    #[derive(Debug)]
    enum State {
        OuterState(Outer),
        InnerState(Inner),
        NullState,
        UsizeState(usize),
        CharState(char),
        StringState(String),
        FieldState(&'static str),
        VecState(Vec<Inner>),
        MapState(HashMap<String, Option<char>>),
        OptionState(bool),
    }

    pub struct OuterDecoder {
        stack: Vec<State>,

    }

    impl OuterDecoder {
        #[inline]
        pub fn new(animal: Outer) -> OuterDecoder {
            OuterDecoder {
                stack: vec!(OuterState(animal)),
            }
        }
    }

    impl Decoder for OuterDecoder {
        type Error = Error;

        fn error(&mut self, msg: &str) -> Error {
            Error::OtherError(msg.to_string())
        }

        // Primitive types:
        #[inline]
        fn read_nil(&mut self) -> Result<(), Error> {
            match self.stack.pop() {
                Some(NullState) => Ok(()),
                _ => Err(Error::SyntaxError("NullState".to_string())),
            }
        }
        #[inline]
        fn read_usize(&mut self) -> Result<usize, Error> {
            match self.stack.pop() {
                Some(UsizeState(value)) => Ok(value),
                _ => Err(Error::SyntaxError("UintState".to_string())),
            }
        }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_u8(&mut self) -> Result<u8, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_isize(&mut self) -> Result<isize, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_i64(&mut self) -> Result<i64, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_i32(&mut self) -> Result<i32, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_i16(&mut self) -> Result<i16, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_i8(&mut self) -> Result<i8, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_bool(&mut self) -> Result<bool, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_f64(&mut self) -> Result<f64, Error> { Err(Error::SyntaxError("".to_string())) }
        fn read_f32(&mut self) -> Result<f32, Error> { Err(Error::SyntaxError("".to_string())) }
        #[inline]
        fn read_char(&mut self) -> Result<char, Error> {
            match self.stack.pop() {
                Some(CharState(c)) => Ok(c),
                _ => Err(Error::SyntaxError("".to_string())),
            }
        }
        #[inline]
        fn read_str(&mut self) -> Result<String, Error> {
            match self.stack.pop() {
                Some(StringState(value)) => Ok(value),
                _ => Err(Error::SyntaxError("".to_string())),
            }
        }

        // Compound types:
        fn read_enum<T, F>(&mut self, _name: &str, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError("".to_string()))
        }

        fn read_enum_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::SyntaxError("".to_string()))
        }

        fn read_enum_variant_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError("".to_string()))
        }

        fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::SyntaxError("".to_string()))
        }

        fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError("".to_string()))
        }

        #[inline]
        fn read_struct<T, F>(&mut self, s_name: &str, _len: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(OuterState(Outer { inner })) => {
                    if s_name == "Outer" {
                        self.stack.push(VecState(inner));
                        self.stack.push(FieldState("inner"));
                        f(self)
                    } else {
                        Err(Error::SyntaxError("expected Outer".to_string()))
                    }
                }
                Some(InnerState(Inner { a: (), b, c })) => {
                    if s_name == "Inner" {
                        self.stack.push(MapState(c));
                        self.stack.push(FieldState("c"));

                        self.stack.push(UsizeState(b));
                        self.stack.push(FieldState("b"));

                        self.stack.push(NullState);
                        self.stack.push(FieldState("a"));
                        f(self)
                    } else {
                        Err(Error::SyntaxError("expected Inner".to_string()))
                    }
                }
                _ => Err(Error::SyntaxError("expected InnerState or OuterState".to_string())),
            }
        }
        #[inline]
        fn read_struct_field<T, F>(&mut self, f_name: &str, _f_idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(FieldState(name)) => {
                    if f_name == name {
                        f(self)
                    } else {
                        Err(Error::SyntaxError("expected FieldState".to_string()))
                    }
                }
                _ => Err(Error::SyntaxError("expected FieldState".to_string()))
            }
        }

        fn read_tuple<T, F>(&mut self, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError("".to_string()))
        }

        fn read_tuple_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError("".to_string()))
        }

        fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError("".to_string()))
        }

        fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError("".to_string()))
        }

        // Specialized types:
        #[inline]
        fn read_option<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder, bool) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(OptionState(b)) => f(self, b),
                _ => Err(Error::SyntaxError("expected OptionState".to_string())),
            }
        }

        #[inline]
        fn read_seq<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder, usize) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(VecState(value)) => {
                    let len = value.len();
                    for inner in value.into_iter().rev() {
                        self.stack.push(InnerState(inner));
                    }
                    f(self, len)
                }
                _ => Err(Error::SyntaxError("expected VecState".to_string()))
            }
        }
        #[inline]
        fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            f(self)
        }

        #[inline]
        fn read_map<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder, usize) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(MapState(map)) => {
                    let len = map.len();
                    for (key, value) in map.into_iter() {
                        match value {
                            Some(c) => {
                                self.stack.push(CharState(c));
                                self.stack.push(OptionState(true));
                            }
                            None => {
                                self.stack.push(OptionState(false));
                            }
                        }
                        self.stack.push(StringState(key));
                    }
                    f(self, len)
                }
                _ => Err(Error::SyntaxError("expected MapState".to_string())),
            }
        }
        #[inline]
        fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            f(self)
        }

        #[inline]
        fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            f(self)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    use std::collections::HashMap;
    use super::{Outer, Inner};
    use super::Error;
    use serde::de;

    use self::State::{
        OuterState,
        InnerState,
        FieldState,
        NullState,
        UsizeState,
        CharState,
        StringState,
        OptionState,
        //TupleState(usize),
        VecState,
        MapState,
        EndState,
    };

    #[derive(Debug)]
    enum State {
        OuterState(Outer),
        InnerState(Inner),
        FieldState(&'static str),
        NullState,
        UsizeState(usize),
        CharState(char),
        StringState(String),
        OptionState(bool),
        //TupleState(uint),
        VecState(Vec<Inner>),
        MapState(HashMap<String, Option<char>>),
        EndState,
    }

    pub struct OuterDeserializer {
        stack: Vec<State>,
    }

    impl OuterDeserializer {
        #[inline]
        pub fn new(outer: Outer) -> OuterDeserializer {
            OuterDeserializer {
                stack: vec!(OuterState(outer)),
            }
        }
    }

    impl Iterator for OuterDeserializer {
        type Item = Result<de::Token, Error>;

        #[inline]
        fn next(&mut self) -> Option<Result<de::Token, Error>> {
            match self.stack.pop() {
                Some(OuterState(Outer { inner })) => {
                    self.stack.push(EndState);
                    self.stack.push(VecState(inner));
                    self.stack.push(FieldState("inner"));
                    Some(Ok(de::Token::StructStart("Outer", 1)))
                }
                Some(InnerState(Inner { a: (), b, c })) => {
                    self.stack.push(EndState);
                    self.stack.push(MapState(c));
                    self.stack.push(FieldState("c"));

                    self.stack.push(UsizeState(b));
                    self.stack.push(FieldState("b"));

                    self.stack.push(NullState);
                    self.stack.push(FieldState("a"));
                    Some(Ok(de::Token::StructStart("Inner", 3)))
                }
                Some(FieldState(name)) => Some(Ok(de::Token::Str(name))),
                Some(VecState(value)) => {
                    self.stack.push(EndState);
                    let len = value.len();
                    for inner in value.into_iter().rev() {
                        self.stack.push(InnerState(inner));
                    }
                    Some(Ok(de::Token::SeqStart(len)))
                }
                Some(MapState(value)) => {
                    self.stack.push(EndState);
                    let len = value.len();
                    for (key, value) in value.into_iter() {
                        match value {
                            Some(c) => {
                                self.stack.push(CharState(c));
                                self.stack.push(OptionState(true));
                            }
                            None => {
                                self.stack.push(OptionState(false));
                            }
                        }
                        self.stack.push(StringState(key));
                    }
                    Some(Ok(de::Token::MapStart(len)))
                }
                //Some(TupleState(len)) => Some(Ok(de::Token::TupleStart(len))),
                Some(NullState) => Some(Ok(de::Token::Null)),
                Some(UsizeState(x)) => Some(Ok(de::Token::Usize(x))),
                Some(CharState(x)) => Some(Ok(de::Token::Char(x))),
                Some(StringState(x)) => Some(Ok(de::Token::String(x))),
                Some(OptionState(x)) => Some(Ok(de::Token::Option(x))),
                Some(EndState) => {
                    Some(Ok(de::Token::End))
                }
                None => None,
            }
        }
    }

    impl de::Deserializer<Error> for OuterDeserializer {
        #[inline]
        fn end_of_stream_error(&mut self) -> Error {
            Error::EndOfStream
        }

        #[inline]
        fn syntax_error(&mut self, token: de::Token, expected: &[de::TokenKind]) -> Error {
            Error::SyntaxError(format!("expected {:?}, found {:?}", expected, token))
        }

        #[inline]
        fn unexpected_name_error(&mut self, token: de::Token) -> Error {
            Error::UnexpectedName(format!("found {:?}", token))
        }

        #[inline]
        fn conversion_error(&mut self, token: de::Token) -> Error {
            Error::UnexpectedName(format!("found {:?}", token))
        }

        #[inline]
        fn missing_field<
            T: de::Deserialize<OuterDeserializer, Error>
        >(&mut self, field: &'static str) -> Result<T, Error> {
            Err(Error::MissingField(field))
        }
    }
}

#[bench]
fn bench_decoder_0_0(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::new();
        map.insert("abc".to_string(), Some('c'));

        let outer = Outer {
            inner: vec!(),
        };

        let mut d = decoder::OuterDecoder::new(outer.clone());
        let value: Outer = Decodable::decode(&mut d).unwrap();

        assert_eq!(value, outer);
    })
}

#[bench]
fn bench_decoder_1_0(b: &mut Bencher) {
    b.iter(|| {
        let map = HashMap::new();

        let outer = Outer {
            inner: vec!(
                Inner {
                    a: (),
                    b: 5,
                    c: map,
                },
            )
        };

        let mut d = decoder::OuterDecoder::new(outer.clone());
        let value: Outer = Decodable::decode(&mut d).unwrap();

        assert_eq!(value, outer);
    })
}

#[bench]
fn bench_decoder_1_5(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::new();
        map.insert("1".to_string(), Some('a'));
        map.insert("2".to_string(), None);
        map.insert("3".to_string(), Some('b'));
        map.insert("4".to_string(), None);
        map.insert("5".to_string(), Some('c'));

        let outer = Outer {
            inner: vec!(
                Inner {
                    a: (),
                    b: 5,
                    c: map,
                },
            )
        };

        let mut d = decoder::OuterDecoder::new(outer.clone());
        let value: Outer = Decodable::decode(&mut d).unwrap();

        assert_eq!(value, outer);
    })
}

#[bench]
fn bench_deserializer_0_0(b: &mut Bencher) {
    b.iter(|| {
        let outer = Outer {
            inner: vec!(),
        };

        let mut d = deserializer::OuterDeserializer::new(outer.clone());
        let value: Outer = Deserialize::deserialize(&mut d).unwrap();

        assert_eq!(value, outer);
    })
}

#[bench]
fn bench_deserializer_1_0(b: &mut Bencher) {
    b.iter(|| {
        let map = HashMap::new();

        let outer = Outer {
            inner: vec!(
                Inner {
                    a: (),
                    b: 5,
                    c: map,
                },
            )
        };

        let mut d = deserializer::OuterDeserializer::new(outer.clone());
        let value: Outer = Deserialize::deserialize(&mut d).unwrap();

        assert_eq!(value, outer);
    })
}

#[bench]
fn bench_deserializer_1_5(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::new();
        map.insert("1".to_string(), Some('a'));
        map.insert("2".to_string(), None);
        map.insert("3".to_string(), Some('b'));
        map.insert("4".to_string(), None);
        map.insert("5".to_string(), Some('c'));

        let outer = Outer {
            inner: vec!(
                Inner {
                    a: (),
                    b: 5,
                    c: map,
                },
            )
        };

        let mut d = deserializer::OuterDeserializer::new(outer.clone());
        let value: Outer = Deserialize::deserialize(&mut d).unwrap();

        assert_eq!(value, outer);
    })
}
