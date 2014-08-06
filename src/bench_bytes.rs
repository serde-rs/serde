use test::Bencher;

use serialize::Decodable;

use de::{Deserializable};

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

    pub struct BytesDecoder {
        iter: vec::MoveItems<u8>,
    }

    impl BytesDecoder {
        #[inline]
        pub fn new(values: Vec<u8>) -> BytesDecoder {
            BytesDecoder {
                iter: values.move_iter()
            }
        }
    }

    impl Decoder<Error> for BytesDecoder {
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
        fn read_int(&mut self) -> Result<int, Error> { Err(SyntaxError) }
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
        fn read_enum<T>(&mut self, _name: &str, _f: |&mut BytesDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_variant<T>(&mut self,
                                _names: &[&str],
                                _f: |&mut BytesDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_variant_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut BytesDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_struct_variant<T>(&mut self,
                                       _names: &[&str],
                                       _f: |&mut BytesDecoder, uint| -> Result<T, Error>)
                                       -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_struct_variant_field<T>(&mut self,
                                             _f_name: &str,
                                             _f_idx: uint,
                                             _f: |&mut BytesDecoder| -> Result<T, Error>)
                                             -> Result<T, Error> { Err(SyntaxError) }

        fn read_struct<T>(&mut self, _s_name: &str, _len: uint, _f: |&mut BytesDecoder| -> Result<T, Error>)
                          -> Result<T, Error> { Err(SyntaxError) }
        fn read_struct_field<T>(&mut self,
                                _f_name: &str,
                                _f_idx: uint,
                                _f: |&mut BytesDecoder| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple<T>(&mut self, _f: |&mut BytesDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_arg<T>(&mut self, _a_idx: uint, _f: |&mut BytesDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple_struct<T>(&mut self,
                                _s_name: &str,
                                _f: |&mut BytesDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_struct_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut BytesDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        // Specialized types:
        fn read_option<T>(&mut self, _f: |&mut BytesDecoder, bool| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        #[inline]
        fn read_seq<T>(&mut self, f: |&mut BytesDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            f(self, 3)
        }
        #[inline]
        fn read_seq_elt<T>(&mut self, _idx: uint, f: |&mut BytesDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }

        fn read_map<T>(&mut self, _f: |&mut BytesDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_key<T>(&mut self, _idx: uint, _f: |&mut BytesDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_val<T>(&mut self, _idx: uint, _f: |&mut BytesDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    use std::num;
    use std::vec;

    use super::{Error, EndOfStream, SyntaxError};

    use de;

    #[deriving(Eq, Show)]
    enum State {
        StartState,
        SepOrEndState,
        EndState,
    }

    pub struct BytesDeserializer {
        state: State,
        len: uint,
        iter: vec::MoveItems<u8>,
    }

    impl BytesDeserializer {
        #[inline]
        pub fn new(values: Vec<int>) -> BytesDeserializer {
            BytesDeserializer {
                state: StartState,
                len: values.len(),
                iter: values.move_iter(),
            }
        }
    }

    impl Iterator<Result<de::Token, Error>> for BytesDeserializer {
        #[inline]
        fn next(&mut self) -> Option<Result<Token, Error>> {
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

    impl de::Deserializer<Error> for BytesDeserializer {
        #[inline]
        fn end_of_stream_error(&self) -> Error {
            EndOfStream
        }

        #[inline]
        fn syntax_error(&self, _token: de::Token) -> Error {
            SyntaxError
        }

        #[inline]
        fn missing_field_error(&mut self, _field: &'static str) -> Error {
            SyntaxError
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

fn run_bench_decoder(bytes: Vec<u8>) {
    let mut d = decoder::BytesDecoder::new(bytes.clone());
    let value: Vec<u8> = Decodable::decode(&mut d).unwrap();

    assert_eq!(value, bytes);
}

fn run_bench_deserializer(bytes: Vec<u8>) {
    let mut d = deserializer::BytesDeserializer::new(bytes.clone());
    let value: Vec<u8> = Deserializable::deserialize(&mut d).unwrap();

    assert_eq!(value, bytes);
}

#[bench]
fn bench_bytes_decoder_empty(b: &mut Bencher) {
    b.iter(|| {
        run_bench_decoder(vec!())
    })
}

#[bench]
fn bench_bytes_deserializer_empty(b: &mut Bencher) {
    b.iter(|| {
        run_bench_deserializer(vec!())
    })
}
