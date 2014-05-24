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
    use serialize::Decoder;

    use super::{Error, EndOfStream, SyntaxError};

    pub struct IntsDecoder {
        len: uint,
        iter: vec::MoveItems<int>,
    }

    impl IntsDecoder {
        #[inline]
        pub fn new(values: Vec<int>) -> IntsDecoder {
            IntsDecoder {
                len: values.len(),
                iter: values.move_iter(),
            }
        }
    }

    impl Decoder<Error> for IntsDecoder {
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
        fn read_enum<T>(&mut self, _name: &str, _f: |&mut IntsDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_variant<T>(&mut self,
                                _names: &[&str],
                                _f: |&mut IntsDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_variant_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut IntsDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_struct_variant<T>(&mut self,
                                       _names: &[&str],
                                       _f: |&mut IntsDecoder, uint| -> Result<T, Error>)
                                       -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_struct_variant_field<T>(&mut self,
                                             _f_name: &str,
                                             _f_idx: uint,
                                             _f: |&mut IntsDecoder| -> Result<T, Error>)
                                             -> Result<T, Error> { Err(SyntaxError) }

        fn read_struct<T>(&mut self, _s_name: &str, _len: uint, _f: |&mut IntsDecoder| -> Result<T, Error>)
                          -> Result<T, Error> { Err(SyntaxError) }
        fn read_struct_field<T>(&mut self,
                                _f_name: &str,
                                _f_idx: uint,
                                _f: |&mut IntsDecoder| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple<T>(&mut self, _f: |&mut IntsDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_arg<T>(&mut self, _a_idx: uint, _f: |&mut IntsDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple_struct<T>(&mut self,
                                _s_name: &str,
                                _f: |&mut IntsDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_struct_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut IntsDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        // Specialized types:
        fn read_option<T>(&mut self, _f: |&mut IntsDecoder, bool| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        #[inline]
        fn read_seq<T>(&mut self, f: |&mut IntsDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            f(self, self.len)
        }
        #[inline]
        fn read_seq_elt<T>(&mut self, _idx: uint, f: |&mut IntsDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }

        fn read_map<T>(&mut self, _f: |&mut IntsDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_key<T>(&mut self, _idx: uint, _f: |&mut IntsDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_val<T>(&mut self, _idx: uint, _f: |&mut IntsDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    use std::num;
    use std::vec;

    use super::{Error, EndOfStream, SyntaxError};

    use de::Deserializer;
    use de::{Token, Int, SeqStart, Sep, End};

    #[deriving(Eq, Show)]
    enum State {
        StartState,
        SepOrEndState,
        EndState,
    }

    pub struct IntsDeserializer {
        state: State,
        len: uint,
        iter: vec::MoveItems<int>,
    }

    impl IntsDeserializer {
        #[inline]
        pub fn new(values: Vec<int>) -> IntsDeserializer {
            IntsDeserializer {
                state: StartState,
                len: values.len(),
                iter: values.move_iter(),
            }
        }
    }

    impl Iterator<Result<Token, Error>> for IntsDeserializer {
        #[inline]
        fn next(&mut self) -> Option<Result<Token, Error>> {
            match self.state {
                StartState => {
                    self.state = SepOrEndState;
                    Some(Ok(SeqStart(self.len)))
                }
                SepOrEndState => {
                    match self.iter.next() {
                        Some(value) => {
                            Some(Ok(Int(value)))
                        }
                        None => {
                            self.state = EndState;
                            Some(Ok(End))
                        }
                    }
                }
                EndState => {
                    None
                }
            }
        }
    }

    impl Deserializer<Error> for IntsDeserializer {
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
        run_decoder(decoder::IntsDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_vec_int_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = vec!();
        run_deserializer(deserializer::IntsDeserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_vec_int_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = vec!(1, 2, 3);
        run_decoder(decoder::IntsDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_vec_int_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = vec!(1, 2, 3);
        run_deserializer(deserializer::IntsDeserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_vec_int_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = range(0, 100).collect();
        run_decoder(decoder::IntsDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_vec_int_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<int> = range(0, 100).collect();
        run_deserializer(deserializer::IntsDeserializer::new(v.clone()), v)
    })
}
