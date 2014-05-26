use std::fmt::Show;
use test::Bencher;

use serialize::{Decoder, Decodable};

use de::{Deserializer, Deserializable};

//////////////////////////////////////////////////////////////////////////////

#[deriving(Show)]
enum Error {
    EndOfStream,
    SyntaxError,
}

//////////////////////////////////////////////////////////////////////////////

mod decoder {
    use std::vec;
    use serialize;

    use super::{Error, EndOfStream, SyntaxError};

    pub struct IntDecoder {
        len: uint,
        iter: vec::MoveItems<int>,
    }

    impl IntDecoder {
        #[inline]
        pub fn new(values: Vec<int>) -> IntDecoder {
            IntDecoder {
                len: values.len(),
                iter: values.move_iter(),
            }
        }
    }

    impl serialize::Decoder<Error> for IntDecoder {
        // Primitive types:
        fn read_nil(&mut self) -> Result<(), Error> { Err(SyntaxError) }
        fn read_uint(&mut self) -> Result<uint, Error> { Err(SyntaxError) }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(SyntaxError) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(SyntaxError) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(SyntaxError) }
        fn read_u8(&mut self) -> Result<u8, Error> { Err(SyntaxError) }
        #[inline]
        fn read_int(&mut self) -> Result<int, Error> {
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
        fn read_str(&mut self) -> Result<StrBuf, Error> { Err(SyntaxError) }

        // Compound types:
        fn read_enum<T>(&mut self, _name: &str, _f: |&mut IntDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_variant<T>(&mut self,
                                _names: &[&str],
                                _f: |&mut IntDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_variant_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut IntDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_struct_variant<T>(&mut self,
                                       _names: &[&str],
                                       _f: |&mut IntDecoder, uint| -> Result<T, Error>)
                                       -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_struct_variant_field<T>(&mut self,
                                             _f_name: &str,
                                             _f_idx: uint,
                                             _f: |&mut IntDecoder| -> Result<T, Error>)
                                             -> Result<T, Error> { Err(SyntaxError) }

        fn read_struct<T>(&mut self, _s_name: &str, _len: uint, _f: |&mut IntDecoder| -> Result<T, Error>)
                          -> Result<T, Error> { Err(SyntaxError) }
        fn read_struct_field<T>(&mut self,
                                _f_name: &str,
                                _f_idx: uint,
                                _f: |&mut IntDecoder| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple<T>(&mut self, _f: |&mut IntDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_arg<T>(&mut self, _a_idx: uint, _f: |&mut IntDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple_struct<T>(&mut self,
                                _s_name: &str,
                                _f: |&mut IntDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_struct_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut IntDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        // Specialized types:
        fn read_option<T>(&mut self, _f: |&mut IntDecoder, bool| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        #[inline]
        fn read_seq<T>(&mut self, f: |&mut IntDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            f(self, self.len)
        }
        #[inline]
        fn read_seq_elt<T>(&mut self, _idx: uint, f: |&mut IntDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }

        fn read_map<T>(&mut self, _f: |&mut IntDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_key<T>(&mut self, _idx: uint, _f: |&mut IntDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_val<T>(&mut self, _idx: uint, _f: |&mut IntDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
    }


    pub struct U8Decoder {
        len: uint,
        iter: vec::MoveItems<u8>,
    }

    impl U8Decoder {
        #[inline]
        pub fn new(values: Vec<u8>) -> U8Decoder {
            U8Decoder {
                len: values.len(),
                iter: values.move_iter(),
            }
        }
    }

    impl serialize::Decoder<Error> for U8Decoder {
        // Primitive types:
        fn read_nil(&mut self) -> Result<(), Error> { Err(SyntaxError) }
        fn read_uint(&mut self) -> Result<uint, Error> { Err(SyntaxError) }
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
        fn read_int(&mut self) -> Result<int, Error> { Err(SyntaxError) }
        fn read_i64(&mut self) -> Result<i64, Error> { Err(SyntaxError) }
        fn read_i32(&mut self) -> Result<i32, Error> { Err(SyntaxError) }
        fn read_i16(&mut self) -> Result<i16, Error> { Err(SyntaxError) }
        fn read_i8(&mut self) -> Result<i8, Error> { Err(SyntaxError) }
        fn read_bool(&mut self) -> Result<bool, Error> { Err(SyntaxError) }
        fn read_f64(&mut self) -> Result<f64, Error> { Err(SyntaxError) }
        fn read_f32(&mut self) -> Result<f32, Error> { Err(SyntaxError) }
        fn read_char(&mut self) -> Result<char, Error> { Err(SyntaxError) }
        fn read_str(&mut self) -> Result<StrBuf, Error> { Err(SyntaxError) }

        // Compound types:
        fn read_enum<T>(&mut self, _name: &str, _f: |&mut U8Decoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_variant<T>(&mut self,
                                _names: &[&str],
                                _f: |&mut U8Decoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_variant_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut U8Decoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_struct_variant<T>(&mut self,
                                       _names: &[&str],
                                       _f: |&mut U8Decoder, uint| -> Result<T, Error>)
                                       -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_struct_variant_field<T>(&mut self,
                                             _f_name: &str,
                                             _f_idx: uint,
                                             _f: |&mut U8Decoder| -> Result<T, Error>)
                                             -> Result<T, Error> { Err(SyntaxError) }

        fn read_struct<T>(&mut self, _s_name: &str, _len: uint, _f: |&mut U8Decoder| -> Result<T, Error>)
                          -> Result<T, Error> { Err(SyntaxError) }
        fn read_struct_field<T>(&mut self,
                                _f_name: &str,
                                _f_idx: uint,
                                _f: |&mut U8Decoder| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple<T>(&mut self, _f: |&mut U8Decoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_arg<T>(&mut self, _a_idx: uint, _f: |&mut U8Decoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple_struct<T>(&mut self,
                                _s_name: &str,
                                _f: |&mut U8Decoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_struct_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut U8Decoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        // Specialized types:
        fn read_option<T>(&mut self, _f: |&mut U8Decoder, bool| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        #[inline]
        fn read_seq<T>(&mut self, f: |&mut U8Decoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            f(self, self.len)
        }
        #[inline]
        fn read_seq_elt<T>(&mut self, _idx: uint, f: |&mut U8Decoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }

        fn read_map<T>(&mut self, _f: |&mut U8Decoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_key<T>(&mut self, _idx: uint, _f: |&mut U8Decoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_val<T>(&mut self, _idx: uint, _f: |&mut U8Decoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    //use std::num;
    use std::vec;

    use super::{Error, EndOfStream, SyntaxError};

    use de;

    #[deriving(Eq, Show)]
    enum State {
        StartState,
        SepOrEndState,
        EndState,
    }

    pub struct IntDeserializer {
        state: State,
        len: uint,
        iter: vec::MoveItems<int>,
    }

    impl IntDeserializer {
        #[inline]
        pub fn new(values: Vec<int>) -> IntDeserializer {
            IntDeserializer {
                state: StartState,
                len: values.len(),
                iter: values.move_iter(),
            }
        }
    }

    impl Iterator<Result<de::Token, Error>> for IntDeserializer {
        #[inline]
        fn next(&mut self) -> Option<Result<de::Token, Error>> {
            match self.state {
                StartState => {
                    self.state = SepOrEndState;
                    Some(Ok(de::SeqStart(self.len)))
                }
                SepOrEndState => {
                    match self.iter.next() {
                        Some(value) => {
                            Some(Ok(de::Int(value)))
                        }
                        None => {
                            self.state = EndState;
                            Some(Ok(de::End))
                        }
                    }
                }
                EndState => {
                    None
                }
            }
        }
    }

    impl de::Deserializer<Error> for IntDeserializer {
        #[inline]
        fn end_of_stream_error(&self) -> Error {
            EndOfStream
        }

        #[inline]
        fn syntax_error(&self) -> Error {
            SyntaxError
        }
    }

    pub struct U8Deserializer {
        state: State,
        len: uint,
        iter: vec::MoveItems<u8>,
    }

    impl U8Deserializer {
        #[inline]
        pub fn new(values: Vec<u8>) -> U8Deserializer {
            U8Deserializer {
                state: StartState,
                len: values.len(),
                iter: values.move_iter(),
            }
        }
    }

    impl Iterator<Result<de::Token, Error>> for U8Deserializer {
        #[inline]
        fn next(&mut self) -> Option<Result<de::Token, Error>> {
            match self.state {
                StartState => {
                    self.state = SepOrEndState;
                    Some(Ok(de::SeqStart(self.len)))
                }
                SepOrEndState => {
                    match self.iter.next() {
                        Some(value) => {
                            Some(Ok(de::U8(value)))
                        }
                        None => {
                            self.state = EndState;
                            Some(Ok(de::End))
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
        fn end_of_stream_error(&self) -> Error {
            EndOfStream
        }

        #[inline]
        fn syntax_error(&self) -> Error {
            SyntaxError
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

fn run_decoder<
    E: Show,
    D: Decoder<E>,
    T: Clone + Eq + Show + Decodable<D, E>
>(mut d: D, value: T) {
    let v: T = Decodable::decode(&mut d).unwrap();

    assert_eq!(value, v);
}

fn run_deserializer<
    E: Show,
    D: Deserializer<E>,
    T: Clone + Eq + Show + Deserializable<E, D>
>(mut d: D, value: T) {
    let v: T = Deserializable::deserialize(&mut d).unwrap();

    assert_eq!(value, v);
}

#[bench]
fn bench_decoder_vec_int_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = vec!();
        run_decoder(decoder::IntDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_vec_int_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = vec!(1, 2, 3);
        run_decoder(decoder::IntDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_vec_int_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = range(0, 100).collect();
        run_decoder(decoder::IntDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_vec_u8_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!();
        run_decoder(decoder::U8Decoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_vec_u8_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!(1, 2, 3);
        run_decoder(decoder::U8Decoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_vec_u8_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = range(0u8, 100).collect();
        run_decoder(decoder::U8Decoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_vec_int_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = vec!();
        run_deserializer(deserializer::IntDeserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_vec_int_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = vec!(1, 2, 3);
        run_deserializer(deserializer::IntDeserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_vec_int_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = range(0, 100).collect();
        run_deserializer(deserializer::IntDeserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_vec_u8_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!();
        run_deserializer(deserializer::U8Deserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_vec_u8_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!(1, 2, 3);
        run_deserializer(deserializer::U8Deserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_vec_u8_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = range(0u8, 100).collect();
        run_deserializer(deserializer::U8Deserializer::new(v.clone()), v)
    })
}
