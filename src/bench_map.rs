use std::fmt::Show;
use std::collections::HashMap;
use test::Bencher;

use serialize::{Decoder, Decodable};

use de::{Deserializer, Deserializable};

//////////////////////////////////////////////////////////////////////////////

#[deriving(Show)]
enum Error {
    EndOfStream,
    SyntaxError,
    OtherError(String),
}

//////////////////////////////////////////////////////////////////////////////

mod decoder {
    use std::collections::HashMap;
    use std::collections::hashmap::MoveEntries;
    use serialize;

    use super::{Error, EndOfStream, SyntaxError, OtherError};

    enum Value {
        StringValue(String),
        IntValue(int),
    }

    pub struct IntDecoder {
        len: uint,
        iter: MoveEntries<String, int>,
        stack: Vec<Value>,
    }

    impl IntDecoder {
        #[inline]
        pub fn new(values: HashMap<String, int>) -> IntDecoder {
            IntDecoder {
                len: values.len(),
                iter: values.move_iter(),
                stack: vec!(),
            }
        }
    }

    impl serialize::Decoder<Error> for IntDecoder {
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

        fn read_seq<T>(&mut self, _f: |&mut IntDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_seq_elt<T>(&mut self, _idx: uint, _f: |&mut IntDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        #[inline]
        fn read_map<T>(&mut self, f: |&mut IntDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            let len = self.len;
            f(self, len)
        }
        #[inline]
        fn read_map_elt_key<T>(&mut self, _idx: uint, f: |&mut IntDecoder| -> Result<T, Error>) -> Result<T, Error> {
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
        fn read_map_elt_val<T>(&mut self, _idx: uint, f: |&mut IntDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    use std::collections::HashMap;
    use std::collections::hashmap::MoveEntries;

    use super::{Error, EndOfStream, SyntaxError};

    use de;

    #[deriving(PartialEq, Show)]
    enum State {
        StartState,
        KeyOrEndState,
        ValueState(int),
        EndState,
    }

    pub struct IntDeserializer {
        stack: Vec<State>,
        len: uint,
        iter: MoveEntries<String, int>,
    }

    impl IntDeserializer {
        #[inline]
        pub fn new(values: HashMap<String, int>) -> IntDeserializer {
            IntDeserializer {
                stack: vec!(StartState),
                len: values.len(),
                iter: values.move_iter(),
            }
        }
    }

    impl Iterator<Result<de::Token, Error>> for IntDeserializer {
        #[inline]
        fn next(&mut self) -> Option<Result<de::Token, Error>> {
            match self.stack.pop() {
                Some(StartState) => {
                    self.stack.push(KeyOrEndState);
                    Some(Ok(de::MapStart(self.len)))
                }
                Some(KeyOrEndState) => {
                    match self.iter.next() {
                        Some((key, value)) => {
                            self.stack.push(ValueState(value));
                            Some(Ok(de::String(key)))
                        }
                        None => {
                            self.stack.push(EndState);
                            Some(Ok(de::End))
                        }
                    }
                }
                Some(ValueState(x)) => {
                    self.stack.push(KeyOrEndState);
                    Some(Ok(de::Int(x)))
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
            T: de::Deserializable<IntDeserializer, Error>
        >(&mut self, _field: &'static str) -> Result<T, Error> {
            Err(SyntaxError)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

fn run_decoder<
    E: Show,
    D: Decoder<E>,
    T: Clone + PartialEq + Show + Decodable<D, E>
>(mut d: D, value: T) {
    let v: T = Decodable::decode(&mut d).unwrap();

    assert_eq!(value, v);
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
    T: Clone + PartialEq + Show + Deserializable<D, E>
>(mut d: D, value: T) {
    let v: T = Deserializable::deserialize(&mut d).unwrap();

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
