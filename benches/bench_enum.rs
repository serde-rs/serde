#![feature(phase)]

#[phase(plugin)]
extern crate serde_macros;

extern crate serde;
extern crate serialize;
extern crate test;

use test::Bencher;

use serialize::{Decoder, Decodable};

use serde::de::{Deserializer, Deserialize};

use Animal::{Dog, Frog};

//////////////////////////////////////////////////////////////////////////////

#[deriving(Clone, PartialEq, Show, Decodable)]
#[deriving_deserialize]
enum Animal {
    Dog,
    Frog(String, int)
}

//////////////////////////////////////////////////////////////////////////////

#[deriving(Show)]
pub enum Error {
    EndOfStream,
    SyntaxError,
    OtherError(String),
}

//////////////////////////////////////////////////////////////////////////////

mod decoder {
    use serialize::Decoder;

    use super::{Animal, Error};
    use super::Animal::{Dog, Frog};
    use super::Error::{SyntaxError, OtherError};
    use self::State::{AnimalState, IntState, StringState};

    enum State {
        AnimalState(Animal),
        IntState(int),
        StringState(String),
    }

    pub struct AnimalDecoder {
        stack: Vec<State>,

    }

    impl AnimalDecoder {
        #[inline]
        pub fn new(animal: Animal) -> AnimalDecoder {
            AnimalDecoder {
                stack: vec!(AnimalState(animal)),
            }
        }
    }

    impl Decoder<Error> for AnimalDecoder {
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
                Some(IntState(x)) => Ok(x),
                _ => Err(SyntaxError),
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
                Some(StringState(x)) => Ok(x),
                _ => Err(SyntaxError),
            }
        }

        // Compound types:
        #[inline]
        fn read_enum<T, F>(&mut self, name: &str, f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(AnimalState(animal)) => {
                    self.stack.push(AnimalState(animal));
                    if name == "Animal" {
                        f(self)
                    } else {
                        Err(SyntaxError)
                    }
                }
                _ => Err(SyntaxError)
            }
        }

        #[inline]
        fn read_enum_variant<T, F>(&mut self, names: &[&str], f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder, uint) -> Result<T, Error>,
        {
            let name = match self.stack.pop() {
                Some(AnimalState(Dog)) => "Dog",
                Some(AnimalState(Frog(x0, x1))) => {
                    self.stack.push(IntState(x1));
                    self.stack.push(StringState(x0));
                    "Frog"
                }
                _ => { return Err(SyntaxError); }
            };

            let idx = match names.iter().position(|n| *n == name) {
                Some(idx) => idx,
                None => { return Err(SyntaxError); }
            };

            f(self, idx)
        }

        #[inline]
        fn read_enum_variant_arg<T, F>(&mut self, _a_idx: uint, f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            f(self)
        }

        fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder, uint) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_struct<T, F>(&mut self, _s_name: &str, _len: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_struct_field<T, F>(&mut self, _f_name: &str, _f_idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple<T, F>(&mut self, _len: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_arg<T, F>(&mut self, _a_idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::SyntaxError)
        }

        // Specialized types:
        fn read_option<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder, bool) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        #[inline]
        fn read_seq<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder, uint) -> Result<T, Error>,
        {
            f(self, 3)
        }

        #[inline]
        fn read_seq_elt<T, F>(&mut self, _idx: uint, f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            f(self)
        }

        fn read_map<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder, uint) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        fn read_map_elt_key<T, F>(&mut self, _idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }

        fn read_map_elt_val<T, F>(&mut self, _idx: uint, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(SyntaxError)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    use super::{Animal, Error};
    use super::Animal::{Dog, Frog};
    use super::Error::{EndOfStream, SyntaxError};
    use self::State::{AnimalState, IntState, StringState, EndState};

    use serde::de;

    enum State {
        AnimalState(Animal),
        IntState(int),
        StringState(String),
        EndState,

    }

    pub struct AnimalDeserializer {
        stack: Vec<State>,
    }

    impl AnimalDeserializer {
        #[inline]
        pub fn new(animal: Animal) -> AnimalDeserializer {
            AnimalDeserializer {
                stack: vec!(AnimalState(animal)),
            }
        }
    }

    impl Iterator<Result<de::Token, Error>> for AnimalDeserializer {
        #[inline]
        fn next(&mut self) -> Option<Result<de::Token, Error>> {
            match self.stack.pop() {
                Some(AnimalState(Dog)) => {
                    self.stack.push(EndState);
                    Some(Ok(de::Token::EnumStart("Animal", "Dog", 0)))
                }
                Some(AnimalState(Frog(x0, x1))) => {
                    self.stack.push(EndState);
                    self.stack.push(IntState(x1));
                    self.stack.push(StringState(x0));
                    Some(Ok(de::Token::EnumStart("Animal", "Frog", 2)))
                }
                Some(IntState(x)) => {
                    Some(Ok(de::Token::Int(x)))
                }
                Some(StringState(x)) => {
                    Some(Ok(de::Token::String(x)))
                }
                Some(EndState) => {
                    Some(Ok(de::Token::End))
                }
                None => None,
            }
        }
    }

    impl de::Deserializer<Error> for AnimalDeserializer {
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
            T: de::Deserialize<AnimalDeserializer, Error>
        >(&mut self, _field: &'static str) -> Result<T, Error> {
            Err(SyntaxError)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

#[bench]
fn bench_decoder_dog(b: &mut Bencher) {
    b.iter(|| {
        let animal = Dog;

        let mut d = decoder::AnimalDecoder::new(animal.clone());
        let value: Animal = Decodable::decode(&mut d).unwrap();

        assert_eq!(value, animal);
    })
}

#[bench]
fn bench_decoder_frog(b: &mut Bencher) {
    b.iter(|| {
        let animal = Frog("Henry".to_string(), 349);

        let mut d = decoder::AnimalDecoder::new(animal.clone());
        let value: Animal = Decodable::decode(&mut d).unwrap();

        assert_eq!(value, animal);
    })
}

#[bench]
fn bench_deserializer_dog(b: &mut Bencher) {
    b.iter(|| {
        let animal = Dog;

        let mut d = deserializer::AnimalDeserializer::new(animal.clone());
        let value: Animal = Deserialize::deserialize(&mut d).unwrap();

        assert_eq!(value, animal);
    })
}

#[bench]
fn bench_deserializer_frog(b: &mut Bencher) {
    b.iter(|| {
        let animal = Frog("Henry".to_string(), 349);

        let mut d = deserializer::AnimalDeserializer::new(animal.clone());
        let value: Animal = Deserialize::deserialize(&mut d).unwrap();

        assert_eq!(value, animal);
    })
}
