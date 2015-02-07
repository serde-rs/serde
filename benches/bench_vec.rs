#![feature(plugin)]

#[plugin]
extern crate serde_macros;

extern crate serde;
extern crate "rustc-serialize" as rustc_serialize;
extern crate test;

use std::fmt::Debug;
use test::Bencher;

use rustc_serialize::{Decoder, Decodable};

use serde::de::{Deserializer, Deserialize};

//////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Debug)]
pub enum Error {
    EndOfStream,
    SyntaxError,
    OtherError(String),
}

//////////////////////////////////////////////////////////////////////////////

mod decoder {
    use std::vec;
    use rustc_serialize;

    use super::Error;
    use super::Error::{EndOfStream, SyntaxError, OtherError};

    pub struct IsizeDecoder {
        len: usize,
        iter: vec::IntoIter<isize>,
    }

    impl IsizeDecoder {
        #[inline]
        pub fn new(values: Vec<isize>) -> IsizeDecoder {
            IsizeDecoder {
                len: values.len(),
                iter: values.into_iter(),
            }
        }
    }

    impl rustc_serialize::Decoder for IsizeDecoder {
        type Error = Error;

        fn error(&mut self, msg: &str) -> Error {
            OtherError(msg.to_string())
        }

        // Primitive types:
        fn read_nil(&mut self) -> Result<(), Error> { Err(SyntaxError) }
        fn read_usize(&mut self) -> Result<usize, Error> { Err(SyntaxError) }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(SyntaxError) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(SyntaxError) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(SyntaxError) }
        fn read_u8(&mut self) -> Result<u8, Error> { Err(SyntaxError) }
        #[inline]
        fn read_isize(&mut self) -> Result<isize, Error> {
            match self.iter.next() {
                Some(value) => Ok(value),
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
        fn read_str(&mut self) -> Result<String, Error> { Err(SyntaxError) }

        // Compound types:
        fn read_enum<T, F>(&mut self, _name: &str, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_variant_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_struct_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple<T, F>(&mut self, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        // Specialized types:
        fn read_option<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder, bool) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        #[inline]
        fn read_seq<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder, usize) -> Result<T, Error>,
        {
            let len = self.len;
            f(self, len)
        }
        #[inline]
        fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            f(self)
        }

        fn read_map<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder, usize) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        fn read_map_elt_key<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        fn read_map_elt_val<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }
    }


    pub struct U8Decoder {
        len: usize,
        iter: vec::IntoIter<u8>,
    }

    impl U8Decoder {
        #[inline]
        pub fn new(values: Vec<u8>) -> U8Decoder {
            U8Decoder {
                len: values.len(),
                iter: values.into_iter(),
            }
        }
    }

    impl rustc_serialize::Decoder for U8Decoder {
        type Error = Error;

        fn error(&mut self, msg: &str) -> Error {
            OtherError(msg.to_string())
        }

        // Primitive types:
        fn read_nil(&mut self) -> Result<(), Error> { Err(SyntaxError) }
        fn read_usize(&mut self) -> Result<usize, Error> { Err(SyntaxError) }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(SyntaxError) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(SyntaxError) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(SyntaxError) }
        #[inline]
        fn read_u8(&mut self) -> Result<u8, Error> {
            match self.iter.next() {
                Some(value) => Ok(value),
                None => Err(EndOfStream),
            }
        }
        #[inline]
        fn read_isize(&mut self) -> Result<isize, Error> { Err(SyntaxError) }
        fn read_i64(&mut self) -> Result<i64, Error> { Err(SyntaxError) }
        fn read_i32(&mut self) -> Result<i32, Error> { Err(SyntaxError) }
        fn read_i16(&mut self) -> Result<i16, Error> { Err(SyntaxError) }
        fn read_i8(&mut self) -> Result<i8, Error> { Err(SyntaxError) }
        fn read_bool(&mut self) -> Result<bool, Error> { Err(SyntaxError) }
        fn read_f64(&mut self) -> Result<f64, Error> { Err(SyntaxError) }
        fn read_f32(&mut self) -> Result<f32, Error> { Err(SyntaxError) }
        fn read_char(&mut self) -> Result<char, Error> { Err(SyntaxError) }
        fn read_str(&mut self) -> Result<String, Error> { Err(SyntaxError) }

        // Compound types:
        fn read_enum<T, F>(&mut self, _name: &str, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder, usize) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_variant_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder, usize) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_struct_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple<T, F>(&mut self, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        // Specialized types:
        fn read_option<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder, bool) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        #[inline]
        fn read_seq<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder, usize) -> Result<T, Error>,
        {
            let len = self.len;
            f(self, len)
        }
        #[inline]
        fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            f(self)
        }

        fn read_map<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder, usize) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        fn read_map_elt_key<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        fn read_map_elt_val<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    //use std::num;
    use std::vec;

    use super::Error;
    use super::Error::{EndOfStream, SyntaxError};
    use self::State::{StartState, SepOrEndState, EndState};

    use serde::de;

    #[derive(PartialEq, Debug)]
    enum State {
        StartState,
        SepOrEndState,
        EndState,
    }

    pub struct IsizeDeserializer {
        state: State,
        len: usize,
        iter: vec::IntoIter<isize>,
    }

    impl IsizeDeserializer {
        #[inline]
        pub fn new(values: Vec<isize>) -> IsizeDeserializer {
            IsizeDeserializer {
                state: StartState,
                len: values.len(),
                iter: values.into_iter(),
            }
        }
    }

    impl Iterator for IsizeDeserializer {
        type Item = Result<de::Token, Error>;

        #[inline]
        fn next(&mut self) -> Option<Result<de::Token, Error>> {
            match self.state {
                StartState => {
                    self.state = SepOrEndState;
                    Some(Ok(de::Token::SeqStart(self.len)))
                }
                SepOrEndState => {
                    match self.iter.next() {
                        Some(value) => {
                            Some(Ok(de::Token::Isize(value)))
                        }
                        None => {
                            self.state = EndState;
                            Some(Ok(de::Token::End))
                        }
                    }
                }
                EndState => {
                    None
                }
            }
        }
    }

    impl de::Deserializer<Error> for IsizeDeserializer {
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
            T: de::Deserialize<IsizeDeserializer, Error>
        >(&mut self, _field: &'static str) -> Result<T, Error> {
            Err(SyntaxError)
        }
    }

    pub struct U8Deserializer {
        state: State,
        len: usize,
        iter: vec::IntoIter<u8>,
    }

    impl U8Deserializer {
        #[inline]
        pub fn new(values: Vec<u8>) -> U8Deserializer {
            U8Deserializer {
                state: StartState,
                len: values.len(),
                iter: values.into_iter(),
            }
        }
    }

    impl Iterator for U8Deserializer {
        type Item = Result<de::Token, Error>;

        #[inline]
        fn next(&mut self) -> Option<Result<de::Token, Error>> {
            match self.state {
                StartState => {
                    self.state = SepOrEndState;
                    Some(Ok(de::Token::SeqStart(self.len)))
                }
                SepOrEndState => {
                    match self.iter.next() {
                        Some(value) => {
                            Some(Ok(de::Token::U8(value)))
                        }
                        None => {
                            self.state = EndState;
                            Some(Ok(de::Token::End))
                        }
                    }
                }
                EndState => {
                    None
                }
            }
        }
    }

    impl de::Deserializer<Error> for U8Deserializer {
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
            T: de::Deserialize<U8Deserializer, Error>
        >(&mut self, _field: &'static str) -> Result<T, Error> {
            Err(SyntaxError)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

fn run_decoder<
    D: Decoder<Error=Error>,
    T: Clone + PartialEq + Debug + Decodable
>(mut d: D, value: T) {
    let v = Decodable::decode(&mut d);

    assert_eq!(Ok(value), v);
}

fn run_deserializer<
    D: Deserializer<E>,
    E: Debug,
    T: Clone + PartialEq + Debug + Deserialize<D, E>
>(mut d: D, value: T) {
    let v: T = Deserialize::deserialize(&mut d).unwrap();

    assert_eq!(value, v);
}

#[bench]
fn bench_decoder_int_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<isize> = vec!();
        run_decoder(decoder::IsizeDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_int_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<isize> = vec!(1, 2, 3);
        run_decoder(decoder::IsizeDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_int_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<isize> = range(0is, 100).collect();
        run_decoder(decoder::IsizeDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_u8_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!();
        run_decoder(decoder::U8Decoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_u8_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!(1, 2, 3);
        run_decoder(decoder::U8Decoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_u8_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = range(0u8, 100).collect();
        run_decoder(decoder::U8Decoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_int_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<isize> = vec!();
        run_deserializer(deserializer::IsizeDeserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_int_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<isize> = vec!(1, 2, 3);
        run_deserializer(deserializer::IsizeDeserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_int_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<isize> = range(0is, 100).collect();
        run_deserializer(deserializer::IsizeDeserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_u8_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!();
        run_deserializer(deserializer::U8Deserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_u8_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!(1, 2, 3);
        run_deserializer(deserializer::U8Deserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_u8_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = range(0u8, 100).collect();
        run_deserializer(deserializer::U8Deserializer::new(v.clone()), v)
    })
}
