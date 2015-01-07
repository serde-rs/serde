#![feature(plugin)]

#[plugin]
extern crate serde_macros;

extern crate serde;
extern crate "rustc-serialize" as rustc_serialize;
extern crate test;

use std::fmt::Show;
use std::collections::HashMap;
use test::Bencher;

use rustc_serialize::{Decoder, Decodable};

use serde::de::{Deserializer, Deserialize};

//////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Show)]
pub enum Error {
    EndOfStream,
    SyntaxError,
    OtherError(String),
}

//////////////////////////////////////////////////////////////////////////////

mod decoder {
    use std::collections::HashMap;
    use std::collections::hash_map::IntoIter;
    use rustc_serialize;

    use super::Error;
    use super::Error::{EndOfStream, SyntaxError, OtherError};
    use self::Value::{StringValue, IntValue};

    enum Value {
        StringValue(String),
        IntValue(int),
    }

    pub struct IntDecoder {
        len: uint,
        iter: IntoIter<String, int>,
        stack: Vec<Value>,
    }

    impl IntDecoder {
        #[inline]
        pub fn new(values: HashMap<String, int>) -> IntDecoder {
            IntDecoder {
                len: values.len(),
                iter: values.into_iter(),
                stack: vec!(),
            }
        }
    }

    impl rustc_serialize::Decoder for IntDecoder {
        type Error = Error;

        fn error(&mut self, msg: &str) -> Error {
            OtherError(msg.to_string())
        }

        // Primitive types:
        fn read_nil(&mut self) -> Result<(), Error> { Err(SyntaxError) }
        fn read_uint(&mut self) -> Result<uint, Error> { Err(SyntaxError) }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(SyntaxError) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(SyntaxError) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(SyntaxError) }
        fn read_u8(&mut self) -> Result<u8, Error> { Err(SyntaxError) }
        #[inline]
        fn read_int(&mut self) -> Result<int, Error> {
            match self.stack.pop() {
                Some(IntValue(x)) => Ok(x),
                Some(_) => Err(SyntaxError),
                None => Err(EndOfStream),
            }
        }
        fn read_i64(&mut self) -> Result<i64, Error> { Err(SyntaxError) }
        fn read_i32(&mut self) -> Result<i32, Error> { Err(SyntaxError) }
        fn read_i16(&mut self) -> Result<i16, Error> { Err(SyntaxError) }
        fn read_i8(&mut self) -> Result<i8, Error> { Err(SyntaxError) }
        fn read_bool(&mut self) -> Result<bool, Error> { Err(SyntaxError) }
        fn read_f64(&mut self) -> Result<f64, Error> { Err(SyntaxError) }
        fn read_f32(&mut self) -> Result<f32, Error> { Err(SyntaxError) }
        fn read_char(&mut self) -> Result<char, Error> { Err(SyntaxError) }
        #[inline]
        fn read_str(&mut self) -> Result<String, Error> {
            match self.stack.pop() {
                Some(StringValue(x)) => Ok(x),
                Some(_) => Err(SyntaxError),
                None => Err(EndOfStream),
            }
        }

        // Compound types:
        fn read_enum<T, F>(&mut self, _name: &str, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder, uint) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_variant_arg<T, F>(&mut self, _a_idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder, uint) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_struct<T, F>(&mut self, _s_name: &str, _len: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_struct_field<T, F>(&mut self, _f_name: &str, _f_idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple<T, F>(&mut self, _len: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_arg<T, F>(&mut self, _a_idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        // Specialized types:
        fn read_option<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder, bool) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        fn read_seq<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder, uint) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        fn read_seq_elt<T, F>(&mut self, _idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        #[inline]
        fn read_map<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder, uint) -> Result<T, Error>,
        {
            let len = self.len;
            f(self, len)
        }
        #[inline]
        fn read_map_elt_key<T, F>(&mut self, _idx: uint, f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            match self.iter.next() {
                Some((key, value)) => {
                    self.stack.push(IntValue(value));
                    self.stack.push(StringValue(key));
                    f(self)
                }
                None => {
                    Err(SyntaxError)
                }
            }
        }

        #[inline]
        fn read_map_elt_val<T, F>(&mut self, _idx: uint, f: F) -> Result<T, Error> where
            F: FnOnce(&mut IntDecoder) -> Result<T, Error>,
        {
            f(self)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    use std::collections::HashMap;
    use std::collections::hash_map::IntoIter;

    use super::Error;
    use super::Error::{EndOfStream, SyntaxError};
    use self::State::{StartState, KeyOrEndState, ValueState, EndState};

    use serde::de;

    #[derive(PartialEq, Show)]
    enum State {
        StartState,
        KeyOrEndState,
        ValueState(int),
        EndState,
    }

    pub struct IntDeserializer {
        stack: Vec<State>,
        len: uint,
        iter: IntoIter<String, int>,
    }

    impl IntDeserializer {
        #[inline]
        pub fn new(values: HashMap<String, int>) -> IntDeserializer {
            IntDeserializer {
                stack: vec!(StartState),
                len: values.len(),
                iter: values.into_iter(),
            }
        }
    }

    impl Iterator for IntDeserializer {
        type Item = Result<de::Token, Error>;

        #[inline]
        fn next(&mut self) -> Option<Result<de::Token, Error>> {
            match self.stack.pop() {
                Some(StartState) => {
                    self.stack.push(KeyOrEndState);
                    Some(Ok(de::Token::MapStart(self.len)))
                }
                Some(KeyOrEndState) => {
                    match self.iter.next() {
                        Some((key, value)) => {
                            self.stack.push(ValueState(value));
                            Some(Ok(de::Token::String(key)))
                        }
                        None => {
                            self.stack.push(EndState);
                            Some(Ok(de::Token::End))
                        }
                    }
                }
                Some(ValueState(x)) => {
                    self.stack.push(KeyOrEndState);
                    Some(Ok(de::Token::Int(x)))
                }
                Some(EndState) => {
                    None
                }
                None => {
                    None
                }
            }
        }
    }

    impl de::Deserializer<Error> for IntDeserializer {
        #[inline]
        fn end_of_stream_error(&mut self) -> Error {
            EndOfStream
        }

        #[inline]
        fn syntax_error(&mut self, _token: de::Token, _expected: &[de::TokenKind]) -> Error {
            SyntaxError
        }

        #[inline]
        fn unexpected_name_error(&mut self, _token: de::Token) -> Error {
            SyntaxError
        }

        #[inline]
        fn conversion_error(&mut self, _token: de::Token) -> Error {
            SyntaxError
        }

        #[inline]
        fn missing_field<
            T: de::Deserialize<IntDeserializer, Error>
        >(&mut self, _field: &'static str) -> Result<T, Error> {
            Err(SyntaxError)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

fn run_decoder<
    D: Decoder<Error=Error>,
    T: Clone + PartialEq + Show + Decodable
>(mut d: D, value: T) {
    let v = Decodable::decode(&mut d);

    assert_eq!(Ok(value), v);
}

#[bench]
fn bench_decoder_000(b: &mut Bencher) {
    b.iter(|| {
        let m: HashMap<String, int> = HashMap::new();
        run_decoder(decoder::IntDecoder::new(m.clone()), m)
    })
}

#[bench]
fn bench_decoder_003(b: &mut Bencher) {
    b.iter(|| {
        let mut m: HashMap<String, int> = HashMap::new();
        for i in range(0i, 3) {
            m.insert(i.to_string(), i);
        }
        run_decoder(decoder::IntDecoder::new(m.clone()), m)
    })
}

#[bench]
fn bench_decoder_100(b: &mut Bencher) {
    b.iter(|| {
        let mut m: HashMap<String, int> = HashMap::new();
        for i in range(0i, 100) {
            m.insert(i.to_string(), i);
        }
        run_decoder(decoder::IntDecoder::new(m.clone()), m)
    })
}

fn run_deserializer<
    D: Deserializer<E>,
    E: Show,
    T: Clone + PartialEq + Show + Deserialize<D, E>
>(mut d: D, value: T) {
    let v: T = Deserialize::deserialize(&mut d).unwrap();

    assert_eq!(value, v);
}

#[bench]
fn bench_deserializer_000(b: &mut Bencher) {
    b.iter(|| {
        let m: HashMap<String, int> = HashMap::new();
        run_deserializer(deserializer::IntDeserializer::new(m.clone()), m)
    })
}

#[bench]
fn bench_deserializer_003(b: &mut Bencher) {
    b.iter(|| {
        let mut m: HashMap<String, int> = HashMap::new();
        for i in range(0i, 3) {
            m.insert(i.to_string(), i);
        }
        run_deserializer(deserializer::IntDeserializer::new(m.clone()), m)
    })
}

#[bench]
fn bench_deserializer_100(b: &mut Bencher) {
    b.iter(|| {
        let mut m: HashMap<String, int> = HashMap::new();
        for i in range(0i, 100) {
            m.insert(i.to_string(), i);
        }
        run_deserializer(deserializer::IntDeserializer::new(m.clone()), m)
    })
}
