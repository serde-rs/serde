use collections::HashMap;
use test::Bencher;

use serialize::{Decoder, Decodable};

use de;
use de::{Token, Deserializer, Deserializable};

//////////////////////////////////////////////////////////////////////////////

#[deriving(Clone, Eq, Show, Decodable)]
struct Inner {
    a: (),
    b: uint,
    c: HashMap<String, Option<char>>,
}

impl<E, D: Deserializer<E>> Deserializable<E, D> for Inner {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Inner, E> {
        match token {
            de::StructStart("Inner", _) |
            de::MapStart(_) => {
                let mut a = None;
                let mut b = None;
                let mut c = None;

                loop {
                    match try!(d.expect_token()) {
                        de::End => { break; }
                        de::Str(name) => {
                            match name {
                                "a" => {
                                    a = Some(try!(de::Deserializable::deserialize(d)));
                                }
                                "b" => {
                                    b = Some(try!(de::Deserializable::deserialize(d)));
                                }
                                "c" => {
                                    c = Some(try!(de::Deserializable::deserialize(d)));
                                }
                                _ => { }
                            }
                        }
                        de::String(ref name) => {
                            match name.as_slice() {
                                "a" => {
                                    a = Some(try!(de::Deserializable::deserialize(d)));
                                }
                                "b" => {
                                    b = Some(try!(de::Deserializable::deserialize(d)));
                                }
                                "c" => {
                                    c = Some(try!(de::Deserializable::deserialize(d)));
                                }
                                _ => { }
                            }
                        }
                        _ => { return d.syntax_error(); }
                    }
                }

                match (a, b, c) {
                    (Some(a), Some(b), Some(c)) => {
                        Ok(Inner { a: a, b: b, c: c })
                    }
                    _ => d.syntax_error(),
                }
            }
            _ => d.syntax_error(),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

#[deriving(Clone, Eq, Show, Decodable)]
struct Outer {
    inner: Vec<Inner>,
}

impl<E, D: Deserializer<E>> Deserializable<E, D> for Outer {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Outer, E> {
        match token {
            de::StructStart("Outer", _) |
            de::MapStart(_) => {
                let mut inner = None;

                loop {
                    match try!(d.expect_token()) {
                        de::End => { break; }
                        de::Str(name) => {
                            match name {
                                "inner" => {
                                    inner = Some(try!(de::Deserializable::deserialize(d)));
                                }
                                _ => { }
                            }
                        }
                        de::String(ref name) => {
                            match name.as_slice() {
                                "inner" => {
                                    inner = Some(try!(de::Deserializable::deserialize(d)));
                                }
                                _ => { }
                            }
                        }
                        _ => { return d.syntax_error(); }
                    }
                }

                match inner {
                    Some(inner) => {
                        Ok(Outer { inner: inner })
                    }
                    _ => d.syntax_error(),
                }
            }
            _ => d.syntax_error(),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

#[deriving(Show)]
enum Error {
    EndOfStream,
    SyntaxError,
}

mod decoder {
    use collections::HashMap;
    use serialize::Decoder;

    use super::{Outer, Inner, Error, SyntaxError};

    #[deriving(Show)]
    enum State {
        OuterState(Outer),
        InnerState(Inner),
        NullState,
        UintState(uint),
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

    impl Decoder<Error> for OuterDecoder {
        // Primitive types:
        #[inline]
        fn read_nil(&mut self) -> Result<(), Error> {
            match self.stack.pop() {
                Some(NullState) => Ok(()),
                _ => Err(SyntaxError),
            }
        }
        #[inline]
        fn read_uint(&mut self) -> Result<uint, Error> {
            match self.stack.pop() {
                Some(UintState(value)) => Ok(value),
                _ => Err(SyntaxError),
            }
        }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(SyntaxError) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(SyntaxError) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(SyntaxError) }
        fn read_u8(&mut self) -> Result<u8, Error> { Err(SyntaxError) }
        fn read_int(&mut self) -> Result<int, Error> { Err(SyntaxError) }
        fn read_i64(&mut self) -> Result<i64, Error> { Err(SyntaxError) }
        fn read_i32(&mut self) -> Result<i32, Error> { Err(SyntaxError) }
        fn read_i16(&mut self) -> Result<i16, Error> { Err(SyntaxError) }
        fn read_i8(&mut self) -> Result<i8, Error> { Err(SyntaxError) }
        fn read_bool(&mut self) -> Result<bool, Error> { Err(SyntaxError) }
        fn read_f64(&mut self) -> Result<f64, Error> { Err(SyntaxError) }
        fn read_f32(&mut self) -> Result<f32, Error> { Err(SyntaxError) }
        #[inline]
        fn read_char(&mut self) -> Result<char, Error> {
            match self.stack.pop() {
                Some(CharState(c)) => Ok(c),
                _ => Err(SyntaxError),
            }
        }
        #[inline]
        fn read_str(&mut self) -> Result<String, Error> {
            match self.stack.pop() {
                Some(StringState(value)) => Ok(value),
                _ => Err(SyntaxError),
            }
        }

        // Compound types:
        fn read_enum<T>(&mut self, _name: &str, _f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_variant<T>(&mut self,
                                _names: &[&str],
                                _f: |&mut OuterDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_variant_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut OuterDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_struct_variant<T>(&mut self,
                                       _names: &[&str],
                                       _f: |&mut OuterDecoder, uint| -> Result<T, Error>)
                                       -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_struct_variant_field<T>(&mut self,
                                             _f_name: &str,
                                             _f_idx: uint,
                                             _f: |&mut OuterDecoder| -> Result<T, Error>)
                                             -> Result<T, Error> { Err(SyntaxError) }

        #[inline]
        fn read_struct<T>(&mut self, s_name: &str, _len: uint, f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(OuterState(Outer { inner: inner })) => {
                    if s_name == "Outer" {
                        self.stack.push(VecState(inner));
                        self.stack.push(FieldState("inner"));
                        f(self)
                    } else {
                        Err(SyntaxError)
                    }
                }
                Some(InnerState(Inner { a: (), b: b, c: c })) => {
                    if s_name == "Inner" {
                        self.stack.push(MapState(c));
                        self.stack.push(FieldState("c"));

                        self.stack.push(UintState(b));
                        self.stack.push(FieldState("b"));

                        self.stack.push(NullState);
                        self.stack.push(FieldState("a"));
                        f(self)
                    } else {
                        Err(SyntaxError)
                    }
                }
                _ => Err(SyntaxError),
            }
        }
        #[inline]
        fn read_struct_field<T>(&mut self, f_name: &str, _f_idx: uint, f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(FieldState(name)) => {
                    if f_name == name {
                        f(self)
                    } else {
                        Err(SyntaxError)
                    }
                }
                _ => Err(SyntaxError)
            }
        }

        fn read_tuple<T>(&mut self, _f: |&mut OuterDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_arg<T>(&mut self, _a_idx: uint, _f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple_struct<T>(&mut self,
                                _s_name: &str,
                                _f: |&mut OuterDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_struct_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut OuterDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        // Specialized types:
        #[inline]
        fn read_option<T>(&mut self, f: |&mut OuterDecoder, bool| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(OptionState(b)) => f(self, b),
                _ => Err(SyntaxError),
            }
        }

        #[inline]
        fn read_seq<T>(&mut self, f: |&mut OuterDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(VecState(value)) => {
                    let len = value.len();
                    for inner in value.move_iter().rev() {
                        self.stack.push(InnerState(inner));
                    }
                    f(self, len)
                }
                _ => Err(SyntaxError)
            }
        }
        #[inline]
        fn read_seq_elt<T>(&mut self, _idx: uint, f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }

        #[inline]
        fn read_map<T>(&mut self, f: |&mut OuterDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(MapState(map)) => {
                    let len = map.len();
                    for (key, value) in map.move_iter() {
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
                _ => Err(SyntaxError),
            }
        }
        #[inline]
        fn read_map_elt_key<T>(&mut self, _idx: uint, f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }
        #[inline]
        fn read_map_elt_val<T>(&mut self, _idx: uint, f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    use collections::HashMap;
    use super::{Outer, Inner, Error, EndOfStream, SyntaxError};
    use de::Deserializer;
    use de::{Token, Uint, Char, String, Null, TupleStart, StructStart, Str, SeqStart, MapStart, End, Option};

    enum State {
        OuterState(Outer),
        InnerState(Inner),
        FieldState(&'static str),
        NullState,
        UintState(uint),
        CharState(char),
        StringState(String),
        OptionState(bool),
        TupleState(uint),
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

    impl Iterator<Result<Token, Error>> for OuterDeserializer {
        #[inline]
        fn next(&mut self) -> Option<Result<Token, Error>> {
            match self.stack.pop() {
                Some(OuterState(Outer { inner })) => {
                    self.stack.push(EndState);
                    self.stack.push(VecState(inner));
                    self.stack.push(FieldState("inner"));
                    Some(Ok(StructStart("Outer", 1)))
                }
                Some(InnerState(Inner { a: (), b, c })) => {
                    self.stack.push(EndState);
                    self.stack.push(MapState(c));
                    self.stack.push(FieldState("c"));

                    self.stack.push(UintState(b));
                    self.stack.push(FieldState("b"));

                    self.stack.push(NullState);
                    self.stack.push(FieldState("a"));
                    Some(Ok(StructStart("Inner", 3)))
                }
                Some(FieldState(name)) => Some(Ok(Str(name))),
                Some(VecState(value)) => {
                    self.stack.push(EndState);
                    let len = value.len();
                    for inner in value.move_iter().rev() {
                        self.stack.push(InnerState(inner));
                    }
                    Some(Ok(SeqStart(len)))
                }
                Some(MapState(value)) => {
                    self.stack.push(EndState);
                    let len = value.len();
                    for (key, value) in value.move_iter() {
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
                    Some(Ok(MapStart(len)))
                }
                Some(TupleState(len)) => Some(Ok(TupleStart(len))),
                Some(NullState) => Some(Ok(Null)),
                Some(UintState(x)) => Some(Ok(Uint(x))),
                Some(CharState(x)) => Some(Ok(Char(x))),
                Some(StringState(x)) => Some(Ok(String(x))),
                Some(OptionState(x)) => Some(Ok(Option(x))),
                Some(EndState) => {
                    Some(Ok(End))
                }
                None => None,
            }
        }
    }

    impl Deserializer<Error> for OuterDeserializer {
        #[inline]
        fn end_of_stream_error<T>(&self) -> Result<T, Error> {
            Err(EndOfStream)
        }

        #[inline]
        fn syntax_error<T>(&self) -> Result<T, Error> {
            Err(SyntaxError)
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
        let mut map = HashMap::new();
        map.insert("abc".to_string(), Some('c'));

        let outer = Outer {
            inner: vec!(),
        };

        let mut d = deserializer::OuterDeserializer::new(outer.clone());
        let value: Outer = Deserializable::deserialize(&mut d).unwrap();

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
        let value: Outer = Deserializable::deserialize(&mut d).unwrap();

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
        let value: Outer = Deserializable::deserialize(&mut d).unwrap();

        assert_eq!(value, outer);
    })
}
